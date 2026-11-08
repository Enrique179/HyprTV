//! Dibuja una hoja con los modelos en varias posturas para compararlos a ojo con los del reproductor. Cada fila usa el repertorio
//! de gestos de su modelo (las chicas, el suyo; el Slayer, el suyo).
//! `cargo run -p hyprcore --example preview -- <salida.png> [lado de cada casilla]`

use hyprcore::look::{HairStyle, ALL};
use hyprcore::pose::{Eyes, Moveset, Mouth, Pose, CHIBI};
use hyprcore::render::{render, Particle, ParticleKind};
use hyprcore::slayer_moves::SLAYER;
use tiny_skia::{Color, IntSize, Pixmap, PixmapPaint, Transform};

fn poses(m: &Moveset) -> Vec<(&'static str, Pose)> {
    vec![
        ("idle", (m.idle)(0.4)),
        ("saludo", (m.wave)(0.3)),
        ("salto", (m.react)(0.3)),
        ("sentada", (m.sit)(1.0, false)),
        ("dormida", (m.sleep)(0.0)),
        ("de espaldas", Pose { spin: std::f32::consts::PI, ..(m.idle)(0.0) }),
        ("habla", Pose { mouth: Mouth::Open, eyes: Eyes::Happy, ..(m.idle)(0.8) }),
        ("camina", (m.walk)(0.12, 1.0, 1.0)),
        ("señala", (m.point)(0.3, 1.0)),
        ("baila 1", (m.dance)(2, 1.3, 0.9)),
        ("baila 2", (m.dance)(3, 0.2, 0.9)),
        ("baila 3", (m.dance)(5, 0.4, 0.9)),
    ]
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "preview.png".into());
    // Segundo argumento opcional: lado de cada casilla en píxeles (en la TV la mascota mide ~224).
    let side = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(144u32);
    let (tw, th) = (side, side);
    let cols = poses(&CHIBI).len() as u32;
    let rows = ALL.len() as u32;
    let mut sheet = Pixmap::new(cols * tw, rows * th).unwrap();
    sheet.fill(Color::from_rgba8(0x0D, 0x0B, 0x16, 255));

    let parts = [
        Particle { kind: ParticleKind::Heart, x: 3.0, y: 9.5, vx: 0.0, vy: 0.0, life: 1.0, age: 0.2 },
        Particle { kind: ParticleKind::Spark, x: -3.2, y: 10.0, vx: 0.0, vy: 0.0, life: 1.0, age: 0.1 },
    ];
    for (r, look) in ALL.iter().enumerate() {
        let moves = if look.hair_style == HairStyle::Helmet { &SLAYER } else { &CHIBI };
        for (c, (name, pose)) in poses(moves).iter().enumerate() {
            let mut buf = vec![0u8; (tw * th * 4) as usize];
            let extra: &[Particle] = if *name == "salto" { &parts } else { &[] };
            render(&mut buf, tw, th, look, pose, extra, 0.5);
            let tile = Pixmap::from_vec(buf, IntSize::from_wh(tw, th).unwrap()).unwrap();
            sheet.draw_pixmap((c as u32 * tw) as i32, (r as u32 * th) as i32, tile.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        }
    }
    sheet.save_png(&out).unwrap();
    println!("guardado {out}");
}
