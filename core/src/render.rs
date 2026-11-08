//! Dibujo de la mascota con trazos vectoriales (port de `PetRenderer.kt` a tiny-skia). Todo se dibuja en "unidades" con el
//! eje y hacia arriba y el origen en los pies: la figura mide ~10,6 unidades y la unidad es `alto / 14`.

use crate::look::{Extra, HairStyle, Look, TopStyle};
use crate::pose::{Eyes, Mouth, Pose};
use crate::render_slayer;
use tiny_skia::{FillRule, FilterQuality, LineCap, Paint, Path, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Rect, Stroke, Transform};

pub(crate) const RAD_TO_DEG: f32 = 57.29578;
const BUCKLE: u32 = 0xFFE2C275;
const LASH: u32 = 0xFF1A1722;
const MOUTH: u32 = 0xFF9D3B4B;
const MOUTH_IN: u32 = 0xFF8A1F3A;
const TONGUE: u32 = 0xFFFF8FA0;
const VIOLET: u32 = 0xFFB794F6;
const PINK: u32 = 0xFFE879F9;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ParticleKind {
    Heart,
    Spark,
    Zzz,
    Sweat,
    Note,
}

/// Algo que flota junto a la mascota. `x`/`y` en unidades de dibujo relativas a sus pies (y hacia arriba).
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub kind: ParticleKind,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub life: f32,
    pub age: f32,
}

pub(crate) struct Cv<'a> {
    pm: PixmapMut<'a>,
    ts: Transform,
}

fn paint(color: u32, alpha: f32) -> Paint<'static> {
    let a = ((color >> 24) & 0xFF) as f32 * alpha.clamp(0.0, 1.0);
    let mut p = Paint::default();
    p.set_color_rgba8((color >> 16) as u8, (color >> 8) as u8, color as u8, a.round() as u8);
    p.anti_alias = true;
    p
}

pub(crate) fn build(f: impl FnOnce(&mut PathBuilder)) -> Option<Path> {
    let mut pb = PathBuilder::new();
    f(&mut pb);
    pb.finish()
}

impl Cv<'_> {
    pub(crate) fn with(&mut self, t: impl FnOnce(Transform) -> Transform, f: impl FnOnce(&mut Self)) {
        let saved = self.ts;
        self.ts = t(saved);
        f(self);
        self.ts = saved;
    }

    /// Alto del búfer en píxeles: los sprites se reducen de antemano al tamaño al que se van a ver.
    pub(crate) fn height(&self) -> u32 {
        self.pm.height()
    }

    /// Pinta una imagen. `t` lleva los píxeles de la imagen a unidades de dibujo (se aplica sobre la transformación actual).
    pub(crate) fn image(&mut self, img: &Pixmap, alpha: f32, t: impl FnOnce(Transform) -> Transform) {
        let paint = PixmapPaint { opacity: alpha.clamp(0.0, 1.0), quality: FilterQuality::Bilinear, ..PixmapPaint::default() };
        self.pm.draw_pixmap(0, 0, img.as_ref(), &paint, t(self.ts), None);
    }

    pub(crate) fn fill(&mut self, path: Option<Path>, color: u32, alpha: f32) {
        if let Some(p) = path {
            self.pm.fill_path(&p, &paint(color, alpha), FillRule::Winding, self.ts, None);
        }
    }

    pub(crate) fn stroke(&mut self, path: Option<Path>, color: u32, alpha: f32, width: f32, cap: LineCap) {
        if let Some(p) = path {
            let s = Stroke { width, line_cap: cap, ..Stroke::default() };
            self.pm.stroke_path(&p, &paint(color, alpha), &s, self.ts, None);
        }
    }

    pub(crate) fn oval(&mut self, color: u32, alpha: f32, x: f32, y: f32, w: f32, h: f32) {
        let path = Rect::from_xywh(x, y, w, h).and_then(PathBuilder::from_oval);
        self.fill(path, color, alpha);
    }

    pub(crate) fn circle(&mut self, color: u32, alpha: f32, r: f32, cx: f32, cy: f32) {
        self.fill(PathBuilder::from_circle(cx, cy, r), color, alpha);
    }

    pub(crate) fn circle_stroke(&mut self, color: u32, alpha: f32, r: f32, cx: f32, cy: f32, width: f32) {
        self.stroke(PathBuilder::from_circle(cx, cy, r), color, alpha, width, LineCap::Butt);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn line(&mut self, color: u32, alpha: f32, x0: f32, y0: f32, x1: f32, y1: f32, width: f32, cap: LineCap) {
        let path = build(|pb| {
            pb.move_to(x0, y0);
            pb.line_to(x1, y1);
        });
        self.stroke(path, color, alpha, width, cap);
    }

    pub(crate) fn rrect(&mut self, color: u32, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let r = r.min(w / 2.0).min(h / 2.0);
        let k = r * 0.5523;
        let path = build(|pb| {
            pb.move_to(x + r, y);
            pb.line_to(x + w - r, y);
            pb.cubic_to(x + w - r + k, y, x + w, y + r - k, x + w, y + r);
            pb.line_to(x + w, y + h - r);
            pb.cubic_to(x + w, y + h - r + k, x + w - r + k, y + h, x + w - r, y + h);
            pb.line_to(x + r, y + h);
            pb.cubic_to(x + r - k, y + h, x, y + h - r + k, x, y + h - r);
            pb.line_to(x, y + r);
            pb.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
            pb.close();
        });
        self.fill(path, color, 1.0);
    }
}

/// Dibuja un fotograma en `buf` (RGBA con alfa premultiplicado, `w*h*4` bytes). Lo deja transparente antes de dibujar.
pub fn render(buf: &mut [u8], w: u32, h: u32, look: &Look, pose: &Pose, particles: &[Particle], time: f32) {
    buf.fill(0);
    let Some(pm) = PixmapMut::from_bytes(buf, w, h) else { return };
    // 15 unidades de alto: la figura (10,6) más el hueco de los saltos (hasta ~2,3) caben sin cortarse.
    let u = h as f32 / 15.0;
    let ts = Transform::from_translate(w as f32 / 2.0, h as f32 * 0.94).pre_scale(u, -u);
    let mut cv = Cv { pm, ts };

    // Sombra en el suelo: se encoge y aclara cuando salta.
    let air = (pose.lift / 4.0).clamp(0.0, 1.0);
    let sw = 4.4 * (1.0 - air * 0.4);
    cv.oval(0xFF000000, 0.28 * (1.0 - air * 0.6), -sw / 2.0, -0.35, sw, 0.7);

    // El Slayer tiene su propio dibujo (otras proporciones y otras piezas), pero el mismo esqueleto y la misma sombra.
    let slayer = look.hair_style == HairStyle::Helmet;
    if slayer {
        render_slayer::figure(&mut cv, pose, time);
    } else {
        girl(&mut cv, look, pose, time);
    }
    for p in particles {
        if slayer {
            render_slayer::particle(&mut cv, p);
        } else {
            shape(&mut cv, p);
        }
    }
}

fn girl(cv: &mut Cv, look: &Look, p: &Pose, time: f32) {
    let front = p.spin.cos() >= 0.0;
    let sx = p.spin.cos().abs().max(0.14);
    let sit_drop = 1.9 * p.sit;
    let (shift, lift) = (p.shift, p.lift);
    let (squash, lean, nod, tilt) = (p.squash, p.lean, p.head_nod, p.head_tilt);
    cv.with(
        |t| {
            t.pre_translate(shift, lift - sit_drop)
                .pre_translate(0.0, sit_drop)
                .pre_scale(sx * (1.0 + 0.07 * squash), 1.0 - 0.10 * squash)
                .pre_translate(0.0, -sit_drop)
        },
        |cv| {
            let sway = p.hair * 0.5 + 0.12 * (time * 3.0).sin();
            leg(cv, look, -1.0, p.leg_l, p.knee_l);
            leg(cv, look, 1.0, p.leg_r, p.knee_r);
            waist(cv, look, sway);
            // El cuerpo de la cintura para arriba se inclina junto.
            cv.with(
                |t| t.pre_rotate_at(-lean * RAD_TO_DEG, 0.0, 2.8),
                |cv| {
                    torso(cv, look, front);
                    arm(cv, look, -1.0, p.arm_l, p.bend_l);
                    arm(cv, look, 1.0, p.arm_r, p.bend_r);
                    cv.with(
                        |t| t.pre_translate(0.0, -nod * 0.35).pre_rotate_at(-tilt * RAD_TO_DEG, 0.0, 5.4),
                        |cv| head(cv, look, p, front, sway, time),
                    );
                    if look.extra == Extra::Scarf {
                        scarf(cv, look, sway);
                    }
                },
            );
        },
    );
}

fn leg(cv: &mut Cv, look: &Look, s: f32, hip: f32, knee: f32) {
    let (hx, hy) = (s * 0.85, 2.3);
    let (kx, ky) = (hx + s * hip.sin() * 0.95, hy - hip.cos() * 0.95);
    let shin = hip - knee;
    let (fx, fy) = (kx + s * shin.sin() * 0.95, ky - shin.cos() * 0.95);
    cv.line(look.pants, 1.0, hx, hy, kx, ky, 1.25, LineCap::Round);
    cv.line(look.pants, 1.0, kx, ky, fx, fy, 1.2, LineCap::Round);
    cv.oval(look.boot, 1.0, fx - 0.85, fy - 0.55, 1.7, 0.95);
}

/// Lo que cubre la cintura: pantalón, o la falda del vestido (que se mece con el baile).
fn waist(cv: &mut Cv, look: &Look, sway: f32) {
    if look.top_style != TopStyle::Dress {
        cv.rrect(look.pants, -1.6, 1.9, 3.2, 1.0, 0.5);
        return;
    }
    let k = sway * 0.3;
    let skirt = build(|pb| {
        pb.move_to(-1.6, 3.0);
        pb.line_to(1.6, 3.0);
        pb.cubic_to(2.0, 2.5, 2.5 + k, 2.1, 2.7 + k, 1.75);
        pb.quad_to(0.0, 1.3, -2.7 + k, 1.75);
        pb.cubic_to(-2.5 + k, 2.1, -2.0, 2.5, -1.6, 3.0);
        pb.close();
    });
    cv.fill(skirt, look.top, 1.0);
    let hem = build(|pb| {
        pb.move_to(-2.65 + k, 1.85);
        pb.quad_to(0.0, 1.4, 2.65 + k, 1.85);
    });
    cv.stroke(hem, look.top_dark, 1.0, 0.2, LineCap::Round);
}

fn torso(cv: &mut Cv, look: &Look, front: bool) {
    let body = build(|pb| {
        pb.move_to(-1.55, 4.8);
        pb.line_to(1.55, 4.8);
        pb.cubic_to(2.3, 4.2, 2.0, 3.0, 1.55, 2.3);
        pb.line_to(-1.55, 2.3);
        pb.cubic_to(-2.0, 3.0, -2.3, 4.2, -1.55, 4.8);
        pb.close();
    });
    cv.fill(body, look.top, 1.0);
    if !front {
        return;
    }
    let d = look.top_dark;
    match look.top_style {
        TopStyle::Jacket => {
            // Correas cruzadas y cinturón, como el equipo de maniobras
            cv.line(d, 1.0, -1.2, 4.6, 1.0, 2.8, 0.22, LineCap::Round);
            cv.line(d, 1.0, 1.2, 4.6, -1.0, 2.8, 0.22, LineCap::Round);
            cv.line(d, 1.0, -1.7, 2.55, 1.7, 2.55, 0.32, LineCap::Butt);
            cv.rrect(BUCKLE, -0.25, 2.35, 0.5, 0.4, 0.08);
        }
        TopStyle::Hoodie => {
            // Cuello de capucha, cordones y bolsillo canguro
            cv.rrect(d, -1.7, 4.35, 3.4, 0.6, 0.3);
            cv.line(d, 1.0, -0.45, 4.3, -0.55, 3.5, 0.12, LineCap::Round);
            cv.line(d, 1.0, 0.45, 4.3, 0.55, 3.5, 0.12, LineCap::Round);
            cv.rrect(d, -1.0, 2.5, 2.0, 0.85, 0.3);
        }
        TopStyle::Dress => {
            // Cuello redondo y lazo en la cintura
            let neck = build(|pb| {
                pb.move_to(-0.9, 4.8);
                pb.quad_to(0.0, 4.0, 0.9, 4.8);
            });
            cv.stroke(neck, look.skin, 1.0, 0.5, LineCap::Round);
            cv.line(d, 1.0, -1.8, 3.1, 1.8, 3.1, 0.28, LineCap::Butt);
            cv.circle(look.accent, 1.0, 0.3, 0.0, 3.1);
        }
        TopStyle::BandTee => {
            // Una nota de grupo de rock en el pecho
            cv.oval(look.accent, 1.0, -0.85, 2.95, 0.85, 0.6);
            cv.line(look.accent, 1.0, -0.1, 3.2, -0.1, 4.4, 0.17, LineCap::Round);
            cv.line(look.accent, 1.0, -0.1, 4.4, 0.65, 3.95, 0.2, LineCap::Round);
        }
        // El Slayer se dibuja entero en `render_slayer`
        TopStyle::Armor => {}
    }
}

fn arm(cv: &mut Cv, look: &Look, s: f32, shoulder: f32, bend: f32) {
    let (sx, sy) = (s * 1.45, 4.5);
    let (ex, ey) = (sx + s * shoulder.sin() * 1.15, sy - shoulder.cos() * 1.15);
    let fore = shoulder + bend;
    let (hx, hy) = (ex + s * fore.sin() * 1.25, ey - fore.cos() * 1.25);
    cv.line(look.top, 1.0, sx, sy, ex, ey, 1.05, LineCap::Round);
    cv.line(if look.bare_forearm() { look.skin } else { look.top }, 1.0, ex, ey, hx, hy, 0.95, LineCap::Round);
    cv.circle(look.skin, 1.0, 0.58, hx, hy);
}

/// La bufanda: lo que más distingue a la chica de la chaqueta. La punta cuelga y se mece con el baile.
fn scarf(cv: &mut Cv, look: &Look, sway: f32) {
    cv.rrect(look.accent, -2.0, 4.0, 4.0, 1.05, 0.52);
    let tail = build(|pb| {
        pb.move_to(0.8, 4.3);
        pb.line_to(2.0, 4.2);
        pb.cubic_to(2.4 + sway * 0.4, 3.5, 2.7 + sway, 3.0, 2.8 + sway, 2.4);
        pb.line_to(1.8 + sway, 2.3);
        pb.cubic_to(1.6 + sway * 0.5, 3.0, 1.2, 3.6, 0.8, 4.3);
        pb.close();
    });
    cv.fill(tail, look.accent, 1.0);
    cv.circle(look.accent_dark, 1.0, 0.5, 1.35, 4.5);
    cv.line(look.accent_dark, 1.0, -1.7, 4.35, 0.7, 4.35, 0.12, LineCap::Round);
}

fn head(cv: &mut Cv, look: &Look, p: &Pose, front: bool, sway: f32, time: f32) {
    back_hair(cv, look, sway);
    if !front {
        cv.oval(look.hair, 1.0, -3.1, 4.9, 6.2, 5.6);
        let shine = build(|pb| {
            pb.move_to(-1.8, 9.6);
            pb.quad_to(0.0, 10.4, 1.8, 9.6);
        });
        cv.stroke(shine, look.hair_light, 1.0, 0.25, LineCap::Round);
        headwear(cv, look, false);
        return;
    }
    // Cara de mochi: redonda y con carrillos
    cv.oval(look.skin, 1.0, -3.0, 4.9, 6.0, 5.2);
    cv.oval(look.skin, 1.0, -2.85, 4.75, 5.7, 3.0);
    eyes(cv, look, p.eyes, time);
    let blush_a = (0.5 * p.blush * 2.0).clamp(0.0, 0.9);
    cv.oval(0xFFFF8A9E, blush_a, -2.6, 6.05, 1.35, 0.8);
    cv.oval(0xFFFF8A9E, blush_a, 1.25, 6.05, 1.35, 0.8);
    mouth(cv, p.mouth);
    // Flequillo con puntas y un mechón a cada lado de la cara
    let fringe = build(|pb| {
        pb.move_to(-3.2, 8.7);
        pb.cubic_to(-3.7, 11.4, 3.7, 11.4, 3.2, 8.7);
        pb.line_to(2.4, 8.1);
        pb.line_to(1.7, 8.95);
        pb.line_to(0.7, 8.15);
        pb.line_to(-0.3, 9.0);
        pb.line_to(-1.3, 8.25);
        pb.line_to(-2.3, 9.0);
        pb.close();
    });
    cv.fill(fringe, look.hair, 1.0);
    for s in [-1.0f32, 1.0] {
        let lock = build(|pb| {
            pb.move_to(s * 3.1, 8.5);
            pb.quad_to(s * 3.5, 6.8, s * 3.0, 5.6);
            pb.quad_to(s * 2.55, 6.8, s * 2.3, 8.3);
            pb.close();
        });
        cv.fill(lock, look.hair, 1.0);
    }
    let shine = build(|pb| {
        pb.move_to(-1.8, 10.2);
        pb.quad_to(-0.3, 10.9, 1.4, 10.4);
    });
    cv.stroke(shine, look.hair_light, 1.0, 0.25, LineCap::Round);
    headwear(cv, look, true);
}

/// El pelo que va detrás de la cabeza: cambia mucho de un modelo a otro (melena, coletas...).
fn back_hair(cv: &mut Cv, look: &Look, sway: f32) {
    match look.hair_style {
        // Media melena a la altura de la barbilla
        HairStyle::Bob => {
            let path = build(|pb| {
                pb.move_to(-3.3, 8.8);
                pb.cubic_to(-4.0 + sway * 0.2, 7.0, -3.9 + sway * 0.4, 5.4, -2.7 + sway * 0.5, 4.8);
                pb.line_to(2.7 + sway * 0.5, 4.8);
                pb.cubic_to(3.9 + sway * 0.4, 5.4, 4.0 + sway * 0.2, 7.0, 3.3, 8.8);
                pb.cubic_to(2.5, 11.4, -2.5, 11.4, -3.3, 8.8);
                pb.close();
            });
            cv.fill(path, look.hair, 1.0);
        }
        HairStyle::Short => cv.oval(look.hair, 1.0, -3.15, 6.2, 6.3, 4.7),
        // El casco se dibuja entero en `helmet`
        HairStyle::Helmet => {}
        // Melena larga hasta los hombros, por encima del cuerpo
        HairStyle::Long => {
            let path = build(|pb| {
                pb.move_to(-3.3, 8.8);
                pb.cubic_to(-4.2 + sway * 0.2, 6.8, -4.1 + sway * 0.4, 4.6, -3.3 + sway * 0.6, 3.6);
                pb.line_to(3.3 + sway * 0.6, 3.6);
                pb.cubic_to(4.1 + sway * 0.4, 4.6, 4.2 + sway * 0.2, 6.8, 3.3, 8.8);
                pb.cubic_to(2.5, 11.4, -2.5, 11.4, -3.3, 8.8);
                pb.close();
            });
            cv.fill(path, look.hair, 1.0);
        }
        // Dos coletas que cuelgan a los lados y se mecen
        HairStyle::Pigtails => {
            cv.oval(look.hair, 1.0, -3.2, 6.4, 6.4, 4.4);
            for s in [-1.0f32, 1.0] {
                let tail = build(|pb| {
                    pb.move_to(s * 3.0, 8.6);
                    pb.cubic_to(s * (4.6 + sway * 0.3), 8.2, s * (4.9 + sway * 0.6), 6.0, s * (4.2 + sway * 0.8), 4.6);
                    pb.cubic_to(s * (3.6 + sway * 0.6), 5.6, s * 3.1, 7.0, s * 2.9, 8.0);
                    pb.close();
                });
                cv.fill(tail, look.hair, 1.0);
            }
        }
    }
}

/// Lo que va encima del pelo: gorro, lazos de las coletas o gafas (la bufanda va en el cuello).
fn headwear(cv: &mut Cv, look: &Look, front: bool) {
    match look.extra {
        Extra::Scarf | Extra::Visor => {}
        Extra::Beanie => {
            let cap = build(|pb| {
                pb.move_to(-3.3, 8.5);
                pb.cubic_to(-3.9, 12.3, 3.9, 12.3, 3.3, 8.5);
                pb.close();
            });
            cv.fill(cap, look.accent, 1.0);
            cv.rrect(look.accent_dark, -3.4, 8.5, 6.8, 0.95, 0.45);
            cv.circle(look.accent, 1.0, 0.6, 0.0, 11.35);
        }
        Extra::Bows => {
            for s in [-1.0f32, 1.0] {
                let (cx, cy) = (s * 3.1, 8.4);
                cv.oval(look.accent, 1.0, cx - 0.75, cy - 0.35, 0.75, 0.7);
                cv.oval(look.accent, 1.0, cx, cy - 0.35, 0.75, 0.7);
                cv.circle(look.accent_dark, 1.0, 0.25, cx, cy);
            }
        }
        Extra::Glasses => {
            if front {
                for s in [-1.0f32, 1.0] {
                    cv.circle_stroke(look.top_dark, 1.0, 1.0, s * 1.3, 7.2, 0.17);
                    cv.circle(0xFFFFFFFF, 0.12, 0.95, s * 1.3, 7.2);
                }
                cv.line(look.top_dark, 1.0, -0.3, 7.3, 0.3, 7.3, 0.15, LineCap::Round);
            }
        }
    }
}

fn eyes(cv: &mut Cv, look: &Look, kind: Eyes, time: f32) {
    let cy = 7.25;
    for side in [-1i32, 1] {
        let x = side as f32 * 1.3;
        let shape = if kind == Eyes::Wink {
            if side < 0 { Eyes::Open } else { Eyes::Happy }
        } else {
            kind
        };
        match shape {
            Eyes::Closed => {
                let path = build(|pb| {
                    pb.move_to(x - 0.65, cy);
                    pb.quad_to(x, cy - 0.5, x + 0.65, cy);
                });
                cv.stroke(path, LASH, 1.0, 0.22, LineCap::Round);
            }
            Eyes::Happy => {
                let path = build(|pb| {
                    pb.move_to(x - 0.65, cy - 0.2);
                    pb.quad_to(x, cy + 0.6, x + 0.65, cy - 0.2);
                });
                cv.stroke(path, LASH, 1.0, 0.24, LineCap::Round);
            }
            Eyes::Open | Eyes::Wide | Eyes::Wink => {
                let wide = shape == Eyes::Wide;
                let rx = if wide { 0.68 } else { 0.58 };
                let ry = if wide { 0.92 } else { 0.8 };
                cv.oval(look.iris, 1.0, x - rx, cy - ry, rx * 2.0, ry * 2.0);
                cv.oval(look.iris_light, 1.0, x - rx * 0.78, cy - ry * 0.92, rx * 1.56, ry * 0.95);
                cv.circle(LASH, 1.0, rx * 0.4, x, cy - 0.05);
                cv.circle(0xFFFFFFFF, 1.0, rx * 0.34, x - rx * 0.32, cy + ry * 0.38);
                cv.circle(0xFFFFFFFF, 1.0, rx * 0.17, x + rx * 0.35, cy - ry * 0.35);
                if wide {
                    cv.circle(0xFFFFFFFF, 0.7 + 0.3 * (time * 6.0).sin(), rx * 0.13, x + rx * 0.5, cy + ry * 0.55);
                }
                // Pestaña superior
                let lash = build(|pb| {
                    pb.move_to(x - rx - 0.1, cy + ry * 0.35);
                    pb.quad_to(x, cy + ry * 1.25, x + rx + 0.1, cy + ry * 0.35);
                });
                cv.stroke(lash, LASH, 1.0, 0.22, LineCap::Round);
            }
        }
    }
}

fn mouth(cv: &mut Cv, kind: Mouth) {
    let my = 5.85;
    match kind {
        Mouth::Smile => {
            let path = build(|pb| {
                pb.move_to(-0.5, my + 0.1);
                pb.quad_to(0.0, my - 0.4, 0.5, my + 0.1);
            });
            cv.stroke(path, MOUTH, 1.0, 0.15, LineCap::Round);
        }
        Mouth::Small => {
            let path = build(|pb| {
                pb.move_to(-0.28, my + 0.05);
                pb.quad_to(0.0, my - 0.15, 0.28, my + 0.05);
            });
            cv.stroke(path, MOUTH, 1.0, 0.14, LineCap::Round);
        }
        Mouth::Pout => cv.line(MOUTH, 1.0, -0.3, my, 0.3, my, 0.14, LineCap::Round),
        Mouth::Open => {
            cv.oval(MOUTH_IN, 1.0, -0.42, my - 0.35, 0.84, 0.7);
            cv.oval(TONGUE, 1.0, -0.22, my - 0.35, 0.44, 0.3);
        }
        Mouth::O => cv.oval(MOUTH_IN, 1.0, -0.25, my - 0.3, 0.5, 0.55),
    }
}

fn shape(cv: &mut Cv, pt: &Particle) {
    let fade = (1.0 - pt.age / pt.life).clamp(0.0, 1.0);
    let (cx, cy) = (pt.x, pt.y);
    match pt.kind {
        ParticleKind::Note => {
            cv.oval(PINK, fade, cx - 0.35, cy - 0.25, 0.7, 0.5);
            cv.line(PINK, fade, cx + 0.3, cy - 0.05, cx + 0.3, cy + 1.1, 0.12, LineCap::Round);
            cv.line(PINK, fade, cx + 0.3, cy + 1.1, cx + 0.75, cy + 0.7, 0.14, LineCap::Round);
        }
        ParticleKind::Heart => {
            let path = build(|pb| {
                pb.move_to(cx, cy - 0.4);
                pb.cubic_to(cx - 0.9, cy + 0.2, cx - 0.5, cy + 0.9, cx, cy + 0.45);
                pb.cubic_to(cx + 0.5, cy + 0.9, cx + 0.9, cy + 0.2, cx, cy - 0.4);
                pb.close();
            });
            cv.fill(path, 0xFFFF6B9D, fade);
        }
        ParticleKind::Sweat => {
            let path = build(|pb| {
                pb.move_to(cx, cy + 0.5);
                pb.quad_to(cx + 0.45, cy - 0.1, cx, cy - 0.35);
                pb.quad_to(cx - 0.45, cy - 0.1, cx, cy + 0.5);
            });
            cv.fill(path, 0xFF7DD3FC, fade);
        }
        ParticleKind::Spark => {
            cv.line(0xFFFDE68A, fade, cx - 0.35, cy, cx + 0.35, cy, 0.14, LineCap::Round);
            cv.line(0xFFFDE68A, fade, cx, cy - 0.35, cx, cy + 0.35, 0.14, LineCap::Round);
        }
        ParticleKind::Zzz => {
            // Una "z" dibujada con trazos (así no hace falta ninguna fuente): crece y se desvanece al subir.
            let k = (pt.age / pt.life).clamp(0.0, 1.0);
            let s = 0.35 + 0.35 * k;
            let path = build(|pb| {
                pb.move_to(cx - s, cy + s);
                pb.line_to(cx + s, cy + s);
                pb.line_to(cx - s, cy - s);
                pb.line_to(cx + s, cy - s);
            });
            cv.stroke(path, VIOLET, fade, 0.16, LineCap::Round);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::look::{ALL, SCARF};
    use crate::pose::gestures;

    fn opaque_pixels(buf: &[u8]) -> usize {
        buf.iter().skip(3).step_by(4).filter(|a| **a > 0).count()
    }

    #[test]
    fn every_look_draws_something_inside_the_buffer() {
        for look in ALL {
            let mut buf = vec![0u8; 144 * 144 * 4];
            render(&mut buf, 144, 144, &look, &gestures::idle(0.0), &[], 0.0);
            let n = opaque_pixels(&buf);
            assert!(n > 2000, "{} dibujó {n} píxeles", look.id);
            assert!(n < 144 * 144, "{} llena todo el buffer", look.id);
        }
    }

    #[test]
    fn transparent_before_drawing() {
        let mut buf = vec![255u8; 64 * 64 * 4];
        render(&mut buf, 64, 64, &SCARF, &gestures::sleep(0.0), &[], 0.0);
        // Las esquinas de arriba nunca se dibujan: la mascota está abajo y centrada.
        assert_eq!(buf[3], 0);
    }

    #[test]
    fn helmet_draws_every_expression_and_the_back() {
        use crate::look::SLAYER;
        for eyes in [Eyes::Open, Eyes::Closed, Eyes::Happy, Eyes::Wide, Eyes::Wink] {
            for spin in [0.0, std::f32::consts::PI] {
                let mut buf = vec![0u8; 120 * 120 * 4];
                let pose = Pose { eyes, spin, ..gestures::idle(0.0) };
                render(&mut buf, 120, 120, &SLAYER, &pose, &[], 0.3);
                assert!(opaque_pixels(&buf) > 2000, "{eyes:?} spin {spin}");
            }
        }
    }

    #[test]
    fn back_view_and_particles_do_not_panic() {
        let mut buf = vec![0u8; 100 * 100 * 4];
        let mut pose = gestures::react(0.3);
        pose.spin = std::f32::consts::PI;
        let parts = [
            Particle { kind: ParticleKind::Heart, x: 1.0, y: 5.0, vx: 0.0, vy: 0.0, life: 1.0, age: 0.2 },
            Particle { kind: ParticleKind::Zzz, x: 2.0, y: 9.0, vx: 0.0, vy: 0.0, life: 1.0, age: 0.5 },
            Particle { kind: ParticleKind::Spark, x: -2.0, y: 7.0, vx: 0.0, vy: 0.0, life: 1.0, age: 0.1 },
        ];
        render(&mut buf, 100, 100, &SCARF, &pose, &parts, 1.0);
        assert!(opaque_pixels(&buf) > 500);
    }
}
