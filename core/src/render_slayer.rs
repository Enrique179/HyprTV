//! Dibujo del Slayer con los sprites que dibujó el usuario (casco, peto, dos brazos y dos botas; `assets/slayer.bin`, hechos a partir de
//! `assets/sprites_png`). Comparte esqueleto (`Pose`), sombra y búfer con `render.rs`, pero en vez de trazos mueve imágenes: cada pieza
//! gira sobre su articulación (hombro, codo, rodilla, cuello). Faltan los muslos (no hay dibujo): se pintan con el mismo verde, gris y
//! contorno negro. La versión vectorial anterior está en `core/backup/render_slayer_vectorial.rs.txt`.
//!
//! Los sprites se guardan como índices de paleta con RLE (44 KB; la TV no lleva decodificador PNG) y se reducen con un filtro de caja al
//! tamaño al que se verán, una vez por tamaño de búfer: así no hay que reescalar en cada fotograma y los bordes salen suaves.

use crate::look::SLAYER;
use crate::pose::{Eyes, Pose};
use crate::render::{build, Cv, Particle, ParticleKind, RAD_TO_DEG};
use std::sync::{Arc, Mutex, OnceLock};
use tiny_skia::{IntSize, LineCap, Pixmap, Transform};

static DATA: &[u8] = include_bytes!("../assets/slayer.bin");

/// Unidades de dibujo por píxel de sprite: el Slayer mide ~800 px de la bota al casco y ~10,3 unidades.
const K: f32 = 0.013;

// Orden de las piezas en `slayer.bin`.
const BOTA_IZQ: usize = 0;
const BOTA_DER: usize = 1;
const CASCO: usize = 6;
const TORSO: usize = 7;

// Índices de la paleta del archivo: los que se cambian o se aíslan.
const VISOR: u8 = 3;
const ROJO: u8 = 4;
const VERDE: u8 = 1;
const TRANSPARENTE: u8 = 255;

// Colores de los muslos: los mismos del dibujo (negro del contorno, verde y gris de las placas).
const NEGRO: u32 = 0xFF030602;
const PLACA: u32 = 0xFF236F09;
const GRIS: u32 = 0xFF4F4F4F;

// Geometría en píxeles de sprite, con y hacia arriba y los pies en 0.
const CADERA: (f32, f32) = (40.0, 280.0); // x de cada cadera (±) y su altura
const MUSLO: f32 = 105.0;
/// Los muslos abren en V: las botas miden 157 de ancho y juntas se pisarían. Se cierra al sentarse.
const ABRE: f32 = 0.28;
const TORSO_ARRIBA: f32 = 554.0; // y del borde superior del peto
const TORSO_PIVOTE: (f32, f32) = (115.0, 0.0); // centro del borde superior, en el sprite
const HOMBRO: (f32, f32) = (105.0, 459.0); // ± x e y de cada articulación del hombro
/// Flexión del codo en `Pose::default()`: con ella el brazo queda tal cual está dibujado.
const REST_BEND: f32 = 0.25;
const CUELLO: f32 = 509.0; // y donde apoya el casco
const CASCO_PIVOTE: (f32, f32) = (131.0, 285.0); // barbilla/cuello, en el sprite
const BOTA_PIVOTE: [(f32, f32); 2] = [(77.0, 45.0), (80.0, 45.0)]; // rodilla, en cada bota
/// Radio del disco gris que tapa la junta del codo.
const CODO_R: f32 = 35.0;

/// Un brazo cortado por el codo (los dos trozos se solapan unos píxeles; el disco del codo tapa la costura al doblarlo). Todo en píxeles de sprite,
/// medido en cada trozo ya recortado.
struct Brazo {
    alto: usize,
    ante: usize,
    hombro: (f32, f32),
    codo: (f32, f32),
    codo2: (f32, f32),
}

const BRAZOS: [Brazo; 2] = [
    Brazo { alto: 2, ante: 3, hombro: (125.0, 85.0), codo: (112.0, 175.0), codo2: (112.0, 11.0) },
    Brazo { alto: 4, ante: 5, hombro: (105.0, 185.0), codo: (115.0, 270.0), codo2: (115.0, 6.0) },
];

/// Una pieza tal como está en el archivo: índices de paleta, 255 = transparente.
struct Pieza {
    w: usize,
    h: usize,
    idx: Vec<u8>,
}

struct Crudo {
    paleta: Vec<[u8; 3]>,
    piezas: Vec<Pieza>,
}

fn decodifica(d: &[u8]) -> Option<Crudo> {
    if d.get(..4)? != b"HSPR" {
        return None;
    }
    let n = *d.get(4)? as usize;
    let mut pos = 5;
    let mut paleta = Vec::with_capacity(n);
    for _ in 0..n {
        paleta.push(d.get(pos..pos + 3)?.try_into().ok()?);
        pos += 3;
    }
    let count = *d.get(pos)? as usize;
    pos += 1;
    let mut piezas = Vec::with_capacity(count);
    for _ in 0..count {
        let w = u16::from_le_bytes(d.get(pos..pos + 2)?.try_into().ok()?) as usize;
        let h = u16::from_le_bytes(d.get(pos + 2..pos + 4)?.try_into().ok()?) as usize;
        let len = u32::from_le_bytes(d.get(pos + 4..pos + 8)?.try_into().ok()?) as usize;
        pos += 8;
        let mut idx = Vec::with_capacity(w * h);
        for &[racha, indice] in d.get(pos..pos + len)?.as_chunks::<2>().0 {
            idx.extend(std::iter::repeat_n(indice, racha as usize));
        }
        pos += len;
        if idx.len() != w * h {
            return None;
        }
        piezas.push(Pieza { w, h, idx });
    }
    Some(Crudo { paleta, piezas })
}

fn crudo() -> &'static Crudo {
    static C: OnceLock<Crudo> = OnceLock::new();
    C.get_or_init(|| decodifica(DATA).expect("assets/slayer.bin corrupto"))
}

/// Una imagen ya reducida y lo que se encogió respecto al sprite (para convertir píxeles del sprite a los de esta imagen).
struct Img {
    pm: Pixmap,
    fx: f32,
    fy: f32,
}

/// Reduce una pieza por `f` (≤ 1) con un filtro de caja (media de los píxeles de origen que caen en cada uno de destino).
/// `color` da el RGB de cada índice, o `None` si no se pinta.
fn reduce(p: &Pieza, f: f32, color: &dyn Fn(u8) -> Option<[u8; 3]>) -> Img {
    let (dw, dh) = (((p.w as f32 * f).ceil() as usize).max(1), ((p.h as f32 * f).ceil() as usize).max(1));
    let mut out = vec![0u8; dw * dh * 4];
    for dy in 0..dh {
        let (y0, y1) = (dy * p.h / dh, ((dy + 1) * p.h / dh).max(dy * p.h / dh + 1));
        for dx in 0..dw {
            let (x0, x1) = (dx * p.w / dw, ((dx + 1) * p.w / dw).max(dx * p.w / dw + 1));
            let (mut r, mut g, mut b, mut a, mut n) = (0u32, 0u32, 0u32, 0u32, 0u32);
            for y in y0..y1 {
                for x in x0..x1 {
                    n += 1;
                    if let Some(c) = color(p.idx[y * p.w + x]) {
                        (r, g, b, a) = (r + c[0] as u32, g + c[1] as u32, b + c[2] as u32, a + 255);
                    }
                }
            }
            // Todo píxel de origen es opaco o transparente: la media ya sale con el alfa premultiplicado.
            let o = (dy * dw + dx) * 4;
            out[o..o + 4].copy_from_slice(&[(r / n) as u8, (g / n) as u8, (b / n) as u8, (a / n) as u8]);
        }
    }
    let size = IntSize::from_wh(dw as u32, dh as u32).expect("pieza con tamaño 0");
    Img { pm: Pixmap::from_vec(out, size).expect("búfer del tamaño justo"), fx: dw as f32 / p.w as f32, fy: dh as f32 / p.h as f32 }
}

/// Todo lo que se dibuja, reducido para un alto de búfer.
struct Juego {
    frente: Vec<Img>,
    casco_espalda: Img,
    torso_espalda: Img,
    visor_luz: Img,
    visor_oscuro: Img,
}

fn construye(alto: u32) -> Juego {
    let c = crudo();
    // Píxeles de pantalla por píxel de sprite: se reduce a eso (nunca se agranda, solo se suaviza al dibujar).
    let f = (alto as f32 / 15.0 * K).min(1.0);
    let pal = |i: u8| if i == TRANSPARENTE { None } else { c.paleta.get(i as usize).copied() };
    let cambia = |de: u8, a: u8| move |i: u8| pal(if i == de { a } else { i });
    let mascara = |rgb: u32| move |i: u8| (i == VISOR).then_some([(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8]);
    Juego {
        frente: c.piezas.iter().map(|p| reduce(p, f, &pal)).collect(),
        // De espaldas no hay visor ni luz del pecho: esas zonas se pintan del verde del casco y del peto.
        casco_espalda: reduce(&c.piezas[CASCO], f, &cambia(VISOR, VERDE)),
        torso_espalda: reduce(&c.piezas[TORSO], f, &cambia(ROJO, VERDE)),
        visor_luz: reduce(&c.piezas[CASCO], f, &mascara(SLAYER.iris_light & 0xFFFFFF)),
        visor_oscuro: reduce(&c.piezas[CASCO], f, &mascara(0)),
    }
}

/// El juego de imágenes para este alto. Se guardan los dos últimos (el ajuste de tamaño y la vista previa de los ajustes).
fn juego(alto: u32) -> Arc<Juego> {
    static CACHE: Mutex<Vec<(u32, Arc<Juego>)>> = Mutex::new(Vec::new());
    let mut c = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, j)) = c.iter().find(|(h, _)| *h == alto) {
        return j.clone();
    }
    let j = Arc::new(construye(alto));
    if c.len() >= 2 {
        c.remove(0);
    }
    c.push((alto, j.clone()));
    j
}

/// Lleva los píxeles de `img` a unidades: el punto `piv` (en píxeles del sprite) queda en `(x, y)` y la imagen gira `deg` grados
/// (positivo = hacia la izquierda de la pantalla).
fn pon(t: Transform, img: &Img, x: f32, y: f32, deg: f32, piv: (f32, f32)) -> Transform {
    t.pre_translate(x, y).pre_rotate(deg).pre_scale(K / img.fx, -K / img.fy).pre_translate(-piv.0 * img.fx, -piv.1 * img.fy)
}

/// Un fotograma del Slayer. Mismas transformaciones que `girl` (salto, aplastamiento, sentarse, giro, inclinación, cabeceo).
pub(crate) fn figure(cv: &mut Cv, p: &Pose, time: f32) {
    let j = juego(cv.height());
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
            leg(cv, &j, -1.0, p.leg_l, p.knee_l, p.sit);
            leg(cv, &j, 1.0, p.leg_r, p.knee_r, p.sit);
            cv.with(
                |t| t.pre_rotate_at(-lean * RAD_TO_DEG, 0.0, CADERA.1 * K),
                |cv| {
                    let torso = if front { &j.frente[TORSO] } else { &j.torso_espalda };
                    cv.image(&torso.pm, 1.0, |t| pon(t, torso, 0.0, TORSO_ARRIBA * K, 0.0, TORSO_PIVOTE));
                    arm(cv, &j, -1.0, p.arm_l, p.bend_l);
                    arm(cv, &j, 1.0, p.arm_r, p.bend_r);
                    cv.with(
                        |t| t.pre_translate(0.0, -nod * 0.35).pre_rotate_at(-tilt * RAD_TO_DEG, 0.0, CUELLO * K),
                        |cv| helmet(cv, &j, p.eyes, front, time),
                    );
                },
            );
        },
    );
}

/// Muslo pintado (no hay sprite) y bota con la rodilla en su extremo. La bota se mantiene vertical con la pierna en reposo.
fn leg(cv: &mut Cv, j: &Juego, s: f32, hip: f32, knee: f32, sit: f32) {
    let (hx, hy) = (s * CADERA.0 * K, CADERA.1 * K);
    let a = hip + ABRE * (1.0 - sit);
    let (kx, ky) = (hx + s * a.sin() * MUSLO * K, hy - a.cos() * MUSLO * K);
    cv.line(NEGRO, 1.0, hx, hy, kx, ky, 94.0 * K, LineCap::Round);
    cv.line(PLACA, 1.0, hx, hy, kx, ky, 82.0 * K, LineCap::Round);
    cv.line(GRIS, 1.0, hx, hy, kx, ky, 28.0 * K, LineCap::Round);
    let side = usize::from(s > 0.0);
    let boot = &j.frente[[BOTA_IZQ, BOTA_DER][side]];
    let shin = hip - crate::pose::REST_LEG - knee;
    cv.image(&boot.pm, 1.0, |t| pon(t, boot, kx, ky, s * shin * RAD_TO_DEG, BOTA_PIVOTE[side]));
}

/// Brazo en dos trozos: el de arriba gira en el hombro y el antebrazo en el codo. `shoulder` y `bend` como en `Pose` (0 = colgando,
/// positivo = hacia fuera); los sprites traen su propia inclinación, que se resta para que 0 sea de verdad colgando.
fn arm(cv: &mut Cv, j: &Juego, s: f32, shoulder: f32, bend: f32) {
    let side = usize::from(s > 0.0);
    let b = &BRAZOS[side];
    let (alto, ante) = (&j.frente[b.alto], &j.frente[b.ante]);
    let (sx, sy) = (s * HOMBRO.0 * K, HOMBRO.1 * K);
    // Giro del trozo de arriba (antihorario, como `pre_rotate`): lo que pide la pose menos la inclinación con la que viene dibujado.
    let nativa = (s * (b.codo.0 - b.hombro.0)).atan2(b.codo.1 - b.hombro.1);
    let giro = s * (shoulder - nativa);
    // El codo es el punto del sprite donde se corta el brazo, llevado a donde cae tras girar el trozo de arriba.
    let (vx, vy) = ((b.codo.0 - b.hombro.0) * K, -(b.codo.1 - b.hombro.1) * K);
    let (ex, ey) = (sx + vx * giro.cos() - vy * giro.sin(), sy + vx * giro.sin() + vy * giro.cos());
    // Con la flexión de reposo el antebrazo sigue al brazo tal como se dibujó; solo lo que se aparta de ella lo dobla.
    let doblez = bend - REST_BEND;
    cv.image(&ante.pm, 1.0, |t| pon(t, ante, ex, ey, (giro + s * doblez) * RAD_TO_DEG, b.codo2));
    cv.image(&alto.pm, 1.0, |t| pon(t, alto, sx, sy, giro * RAD_TO_DEG, b.hombro));
    // Disco de la junta: solo cuando el codo se dobla, para tapar el hueco que se abre en la costura
    let r = CODO_R * (doblez.abs() / 0.5).min(1.0);
    if doblez.abs() > 0.05 {
        cv.circle(NEGRO, 1.0, r * K + 0.05, ex, ey);
        cv.circle(GRIS, 1.0, r * K, ex, ey);
    }
}

/// El casco: lo que más pesa en la silueta. Los ojos de la pose mandan sobre el visor (no hay cara): `Closed` lo apaga, `Wide` lo enciende
/// y `Happy` (la mascota está hablando) lo hace latir.
fn helmet(cv: &mut Cv, j: &Juego, kind: Eyes, front: bool, time: f32) {
    let casco = if front { &j.frente[CASCO] } else { &j.casco_espalda };
    cv.image(&casco.pm, 1.0, |t| pon(t, casco, 0.0, CUELLO * K, 0.0, CASCO_PIVOTE));
    if !front {
        return;
    }
    let (capa, alfa) = match kind {
        Eyes::Closed => (&j.visor_oscuro, 0.5),
        Eyes::Wide => (&j.visor_luz, 0.35 + 0.15 * (time * 6.0).sin()),
        Eyes::Happy => (&j.visor_luz, 0.14 + 0.12 * (time * 14.0).sin()),
        Eyes::Open | Eyes::Wink => return,
    };
    cv.image(&capa.pm, alfa, |t| pon(t, capa, 0.0, CUELLO * K, 0.0, CASCO_PIVOTE));
}

/// Las partículas del Slayer: nada de corazones ni chispitas. Brasas, esquirlas de disparo y humo.
pub(crate) fn particle(cv: &mut Cv, pt: &Particle) {
    let fade = (1.0 - pt.age / pt.life).clamp(0.0, 1.0);
    let (cx, cy) = (pt.x, pt.y);
    match pt.kind {
        ParticleKind::Heart | ParticleKind::Note | ParticleKind::Sweat => {
            // Brasa: una llama pequeña
            let flame = build(|pb| {
                pb.move_to(cx, cy + 0.7);
                pb.cubic_to(cx + 0.7, cy + 0.1, cx + 0.5, cy - 0.5, cx, cy - 0.5);
                pb.cubic_to(cx - 0.5, cy - 0.5, cx - 0.7, cy + 0.1, cx, cy + 0.7);
                pb.close();
            });
            cv.fill(flame, 0xFFFF6A1A, fade);
            cv.circle(0xFFFFD23F, fade, 0.2, cx, cy - 0.15);
        }
        ParticleKind::Spark => {
            // Esquirla de disparo: un aspa afilada en diagonal
            cv.line(0xFFFFB020, fade, cx - 0.4, cy - 0.4, cx + 0.4, cy + 0.4, 0.18, LineCap::Butt);
            cv.line(0xFFFFB020, fade, cx - 0.4, cy + 0.4, cx + 0.4, cy - 0.4, 0.18, LineCap::Butt);
            cv.circle(0xFFFFF1B0, fade, 0.1, cx, cy);
        }
        ParticleKind::Zzz => {
            // Roncar en armadura: un hilo de humo gris en vez de «z» violetas
            let k = (pt.age / pt.life).clamp(0.0, 1.0);
            cv.circle(0xFF8A8A92, fade * 0.8, 0.25 + 0.3 * k, cx, cy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::render;

    #[test]
    fn el_archivo_se_decodifica_entero() {
        let c = crudo();
        assert_eq!(c.paleta.len(), 6);
        assert_eq!(c.piezas.len(), 8);
        assert!(c.piezas.iter().all(|p| p.w > 50 && p.h > 50));
        // Cada pieza tiene algo opaco y algo transparente (fondo quitado)
        for p in &c.piezas {
            assert!(p.idx.contains(&TRANSPARENTE) && p.idx.iter().any(|&i| i != TRANSPARENTE));
        }
        assert!(decodifica(&DATA[..DATA.len() - 5]).is_none(), "un archivo cortado no debe decodificarse");
    }

    #[test]
    fn se_reduce_al_tamano_pedido() {
        let pequeno = construye(112);
        let grande = construye(448);
        assert!(pequeno.frente[CASCO].pm.width() < grande.frente[CASCO].pm.width());
        assert!(pequeno.frente[CASCO].pm.width() >= 10);
        // Nunca se agranda: a un tamaño enorme se queda en el original
        assert_eq!(construye(4000).frente[CASCO].pm.width() as usize, crudo().piezas[CASCO].w);
    }

    #[test]
    fn dibuja_el_slayer_dentro_del_lienzo_en_todas_las_posturas() {
        use crate::pose::{gestures, Pose};
        let (w, h) = (224u32, 224u32);
        for pose in [
            Pose::default(),
            gestures::sit(0.5, false),
            gestures::sleep(0.2),
            gestures::wave(0.3),
            gestures::point(0.2, 1.0),
            Pose { spin: std::f32::consts::PI, ..Pose::default() },
        ] {
            let mut buf = vec![0u8; (w * h * 4) as usize];
            render(&mut buf, w, h, &SLAYER, &pose, &[], 0.5);
            let opacos = buf.as_chunks::<4>().0.iter().filter(|p| p[3] == 255).count();
            assert!(opacos > 2500, "se ve muy poco Slayer: {opacos} píxeles opacos");
            // El borde izquierdo y el superior quedan vacíos: no se corta contra el lienzo
            assert!((0..h).all(|y| buf[(y * w * 4 + 3) as usize] == 0));
            assert!((0..w).all(|x| buf[(x * 4 + 3) as usize] == 0));
        }
    }
}
