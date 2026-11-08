//! Postura del esqueleto de la mascota y los gestos que no son baile. Port de `Pose.kt` y `Gestures` del reproductor.
//! Unidades de dibujo: la figura mide ~10,6 de alto; ángulos en radianes. Brazos y piernas se miden desde "colgando hacia
//! abajo" (0 = abajo, π/2 = horizontal, π = vertical hacia arriba); el codo/rodilla se suma al ángulo del tramo anterior.

use std::f32::consts::PI;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Eyes {
    Open,
    Closed,
    Happy,
    Wide,
    Wink,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mouth {
    Smile,
    Open,
    Small,
    Pout,
    O,
}

pub const REST_ARM: f32 = 0.22;
pub const REST_LEG: f32 = 0.1;

#[derive(Clone, Copy, Debug)]
pub struct Pose {
    /// Altura de los pies sobre el suelo (saltos).
    pub lift: f32,
    pub shift: f32,
    /// 0..1: aplastamiento al caer o agacharse.
    pub squash: f32,
    /// Inclinación del tronco; positivo = hacia la derecha de la pantalla.
    pub lean: f32,
    pub head_tilt: f32,
    /// -1..1: cabeceo; positivo = agacha la cabeza.
    pub head_nod: f32,
    pub arm_l: f32,
    pub bend_l: f32,
    pub arm_r: f32,
    pub bend_r: f32,
    pub leg_l: f32,
    pub knee_l: f32,
    pub leg_r: f32,
    pub knee_r: f32,
    /// 0..1: sentada en el suelo.
    pub sit: f32,
    /// Empuje lateral de las coletas y la falda.
    pub hair: f32,
    /// Giro sobre sí misma (cos < 0 = de espaldas).
    pub spin: f32,
    pub eyes: Eyes,
    pub mouth: Mouth,
    pub blush: f32,
}

impl Default for Pose {
    fn default() -> Self {
        Pose {
            lift: 0.0,
            shift: 0.0,
            squash: 0.0,
            lean: 0.0,
            head_tilt: 0.0,
            head_nod: 0.0,
            arm_l: REST_ARM,
            bend_l: 0.25,
            arm_r: REST_ARM,
            bend_r: 0.25,
            leg_l: REST_LEG,
            knee_l: 0.0,
            leg_r: REST_LEG,
            knee_r: 0.0,
            sit: 0.0,
            hair: 0.0,
            spin: 0.0,
            eyes: Eyes::Open,
            mouth: Mouth::Smile,
            blush: 0.5,
        }
    }
}

impl Pose {
    pub fn lerp(a: &Pose, b: &Pose, t: f32) -> Pose {
        let k = t.clamp(0.0, 1.0);
        let f = |x: f32, y: f32| x + (y - x) * k;
        Pose {
            lift: f(a.lift, b.lift),
            shift: f(a.shift, b.shift),
            squash: f(a.squash, b.squash),
            lean: f(a.lean, b.lean),
            head_tilt: f(a.head_tilt, b.head_tilt),
            head_nod: f(a.head_nod, b.head_nod),
            arm_l: f(a.arm_l, b.arm_l),
            bend_l: f(a.bend_l, b.bend_l),
            arm_r: f(a.arm_r, b.arm_r),
            bend_r: f(a.bend_r, b.bend_r),
            leg_l: f(a.leg_l, b.leg_l),
            knee_l: f(a.knee_l, b.knee_l),
            leg_r: f(a.leg_r, b.leg_r),
            knee_r: f(a.knee_r, b.knee_r),
            sit: f(a.sit, b.sit),
            hair: f(a.hair, b.hair),
            spin: f(a.spin, b.spin),
            blush: f(a.blush, b.blush),
            // Lo discreto cambia a mitad del fundido.
            eyes: if k < 0.5 { a.eyes } else { b.eyes },
            mouth: if k < 0.5 { a.mouth } else { b.mouth },
        }
    }
}

/// Todo lo que sabe hacer una mascota: los gestos de siempre y su repertorio de bailes. Las chicas usan [`CHIBI`] y el Slayer el suyo
/// (`slayer_moves::SLAYER`); `Brain` solo elige cuál según el modelo y no sabe de qué se trata.
pub struct Moveset {
    pub idle: fn(f32) -> Pose,
    pub sit: fn(f32, bool) -> Pose,
    pub sleep: fn(f32) -> Pose,
    pub react: fn(f32) -> Pose,
    pub yawn: fn(f32) -> Pose,
    pub wave: fn(f32) -> Pose,
    pub walk: fn(f32, f32, f32) -> Pose,
    pub point: fn(f32, f32) -> Pose,
    pub look: fn(f32, f32) -> Pose,
    pub dance: fn(usize, f32, f32) -> Pose,
    pub dance_count: usize,
}

pub const CHIBI: Moveset = Moveset {
    idle: gestures::idle,
    sit: gestures::sit,
    sleep: gestures::sleep,
    react: gestures::react,
    yawn: gestures::yawn,
    wave: gestures::wave,
    walk: gestures::walk,
    point: gestures::point,
    look: gestures::look,
    dance: crate::moves::dance,
    dance_count: crate::moves::COUNT,
};

fn land(t: f32) -> f32 {
    let c = 0.5 + 0.5 * (2.0 * PI * t).cos();
    c * c * c
}

fn frac(t: f32) -> f32 {
    t - t.floor()
}

/// Posturas de reposo y reacciones. `time` y `age` en segundos.
pub mod gestures {
    use super::*;

    pub fn idle(time: f32) -> Pose {
        Pose {
            squash: 0.05 * (0.5 + 0.5 * (time * 2.2).sin()),
            head_tilt: 0.08 * (time * 0.7).sin(),
            arm_l: REST_ARM + 0.05 * (time * 1.1).sin(),
            arm_r: REST_ARM + 0.05 * (time * 1.1 + 1.0).sin(),
            hair: 0.15 * (time * 1.3).sin(),
            ..Pose::default()
        }
    }

    pub fn sit(time: f32, happy: bool) -> Pose {
        let swing = (time * 3.0).sin();
        Pose {
            sit: 1.0,
            head_tilt: 0.1 * (time * 0.8).sin(),
            squash: 0.03 * (time * 2.0).sin(),
            leg_l: 1.25 + 0.15 * swing,
            leg_r: 1.25 - 0.15 * swing,
            knee_l: 0.2,
            knee_r: 0.2,
            arm_l: 0.5,
            bend_l: 0.0,
            arm_r: 0.5,
            bend_r: 0.0,
            hair: 0.2 * (time * 1.5).sin(),
            eyes: if happy { Eyes::Happy } else { Eyes::Open },
            ..Pose::default()
        }
    }

    pub fn sleep(time: f32) -> Pose {
        Pose {
            sit: 1.0,
            head_nod: 0.8,
            lean: 0.06,
            squash: 0.04 * (time * 1.4).sin(),
            leg_l: 1.3,
            leg_r: 1.3,
            arm_l: 0.6,
            bend_l: 0.0,
            arm_r: 0.6,
            bend_r: 0.0,
            eyes: Eyes::Closed,
            mouth: if (time * 1.4).sin() > 0.2 { Mouth::O } else { Mouth::Small },
            blush: 0.7,
            ..Pose::default()
        }
    }

    /// Tras un chiste: salta sorprendida y se pone contenta.
    pub fn react(age: f32) -> Pose {
        let up = (PI * (age / 0.6).clamp(0.0, 1.0)).sin();
        Pose {
            lift: 1.8 * up,
            squash: if age < 0.6 { 0.0 } else { 0.4 * land(age * 1.6) },
            arm_l: 2.6,
            bend_l: 0.2,
            arm_r: 2.6,
            bend_r: 0.2,
            knee_l: 0.8 * up,
            knee_r: 0.8 * up,
            hair: 1.0 * up,
            eyes: if age < 0.6 { Eyes::Wide } else { Eyes::Happy },
            mouth: if age < 0.6 { Mouth::O } else { Mouth::Open },
            blush: 1.0,
            ..Pose::default()
        }
    }

    pub fn yawn(age: f32) -> Pose {
        let k = (PI * (age / 1.6).clamp(0.0, 1.0)).sin();
        Pose {
            lean: -0.05 * k,
            arm_l: REST_ARM + 2.4 * k,
            bend_l: 0.2,
            arm_r: REST_ARM + 2.4 * k,
            bend_r: 0.2,
            squash: -0.05 * k,
            eyes: if k > 0.2 { Eyes::Closed } else { Eyes::Open },
            mouth: if k > 0.2 { Mouth::O } else { Mouth::Smile },
            ..Pose::default()
        }
    }

    pub fn wave(time: f32) -> Pose {
        let s = (time * 8.0).sin();
        Pose {
            arm_r: 2.5 + 0.3 * s,
            bend_r: 0.4 * s,
            lift: 0.1,
            hair: 0.3 * s,
            eyes: Eyes::Happy,
            mouth: Mouth::Open,
            blush: 0.9,
            ..Pose::default()
        }
    }

    /// Andando de lado a lado. `dir` -1 izquierda / 1 derecha: se inclina hacia donde va y el pelo se queda atrás.
    pub fn walk(time: f32, speed: f32, dir: f32) -> Pose {
        let s = (time * 9.0 * speed).sin();
        Pose {
            lift: 0.18 * s.abs(),
            lean: 0.05 * s + 0.07 * dir,
            knee_l: 0.9 * s.max(0.0),
            knee_r: 0.9 * (-s).max(0.0),
            arm_l: 0.3 + 0.3 * s,
            arm_r: 0.3 - 0.3 * s,
            hair: 0.4 * s - 0.35 * dir,
            ..Pose::default()
        }
    }

    /// Señala hacia arriba con el brazo del lado de `side` (-1 izquierda / 1 derecha), como diciendo «ese de ahí».
    pub fn point(time: f32, side: f32) -> Pose {
        let up = 2.55 + 0.12 * (time * 5.0).sin();
        let left = side < 0.0;
        Pose {
            lean: 0.08 * side,
            head_tilt: 0.1 * side,
            arm_l: if left { up } else { 0.75 },
            bend_l: if left { 0.1 } else { -1.5 },
            arm_r: if left { 0.75 } else { up },
            bend_r: if left { -1.5 } else { 0.1 },
            eyes: Eyes::Wide,
            mouth: Mouth::Small,
            blush: 0.7,
            ..Pose::default()
        }
    }

    /// Mira hacia un lado (`side` -1 izquierda / 1 derecha) con curiosidad, como quien lee el menú.
    pub fn look(time: f32, side: f32) -> Pose {
        Pose {
            head_tilt: 0.16 * side,
            lean: 0.05 * side,
            arm_l: REST_ARM + 0.05 * (time * 1.1).sin(),
            eyes: if frac(time * 0.4) > 0.9 { Eyes::Closed } else { Eyes::Wide },
            mouth: Mouth::Small,
            ..Pose::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_endpoints_and_discrete_switch() {
        let a = Pose { lift: 0.0, eyes: Eyes::Open, ..Pose::default() };
        let b = Pose { lift: 2.0, eyes: Eyes::Closed, ..Pose::default() };
        assert_eq!(Pose::lerp(&a, &b, 0.0).lift, 0.0);
        assert_eq!(Pose::lerp(&a, &b, 1.0).lift, 2.0);
        assert!((Pose::lerp(&a, &b, 0.5).lift - 1.0).abs() < 1e-6);
        assert_eq!(Pose::lerp(&a, &b, 0.4).eyes, Eyes::Open);
        assert_eq!(Pose::lerp(&a, &b, 0.6).eyes, Eyes::Closed);
        // Fuera de rango se recorta.
        assert_eq!(Pose::lerp(&a, &b, 5.0).lift, 2.0);
    }

    #[test]
    fn gestures_stay_finite() {
        for i in 0..200 {
            let t = i as f32 * 0.05;
            for p in [
                gestures::idle(t),
                gestures::sit(t, true),
                gestures::sleep(t),
                gestures::react(t),
                gestures::yawn(t),
                gestures::wave(t),
                gestures::look(t, 1.0),
                gestures::walk(t, 1.0, -1.0),
                gestures::point(t, 1.0),
            ] {
                assert!(p.lift.is_finite() && p.squash.is_finite() && p.arm_l.is_finite());
            }
        }
    }
}
