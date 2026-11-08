//! Coreografías para cuando la mascota baila sin música (port de `Moves.kt` del reproductor). `t` va en pulsos (el 0 es un
//! golpe) y `a` es la amplitud 0..1; cada baile es periódico en `beats` pulsos, así que se encadena sin saltos.

use crate::pose::{Eyes, Mouth, Pose};
use std::f32::consts::PI;

const TAU: f32 = 2.0 * PI;

fn frac(t: f32) -> f32 {
    t - t.floor()
}

/// 0 en el golpe, 1 a mitad del pulso: un salto que aterriza justo en el golpe.
fn hop(t: f32) -> f32 {
    (PI * frac(t)).sin()
}

/// 1 en el golpe, cae rápido: el aplastamiento al aterrizar.
fn land(t: f32) -> f32 {
    let c = 0.5 + 0.5 * (TAU * t).cos();
    c * c * c
}

fn wave(t: f32, period: f32) -> f32 {
    (TAU * t / period).sin()
}

pub const COUNT: usize = 8;

/// Segundos por pulso (120 ppm).
pub const BEAT_SECS: f32 = 0.5;

pub fn name(idx: usize) -> &'static str {
    ["balanceo", "olas", "paso lateral", "palmas", "cadera", "ola de brazos", "salto", "giro"][idx % COUNT]
}

pub fn dance(idx: usize, t: f32, a: f32) -> Pose {
    let base = Pose::default();
    match idx % COUNT {
        0 => {
            let w = wave(t, 2.0);
            Pose {
                shift: 0.55 * a * w,
                lean: 0.10 * a * w,
                head_tilt: -0.14 * a * w,
                squash: 0.12 * a * land(t),
                arm_l: 0.3 + 0.28 * a * w,
                bend_l: 0.3,
                arm_r: 0.3 - 0.28 * a * w,
                bend_r: 0.3,
                leg_l: 0.1 + 0.1 * w.max(0.0),
                knee_l: 0.35 * w.max(0.0),
                knee_r: 0.35 * (-w).max(0.0),
                hair: 0.5 * a * w,
                eyes: Eyes::Happy,
                mouth: Mouth::Smile,
                ..base
            }
        }
        1 => {
            let w = wave(t, 2.0);
            Pose {
                shift: 0.3 * wave(t, 4.0),
                lean: 0.06 * w,
                squash: 0.1 * a * land(t),
                arm_l: 1.9 + 0.8 * a * w,
                bend_l: 0.3,
                arm_r: 1.9 - 0.8 * a * w,
                bend_r: 0.3,
                hair: 0.4 * w,
                eyes: Eyes::Closed,
                mouth: Mouth::Small,
                ..base
            }
        }
        2 => {
            let sway = (PI * t / 2.0).sin();
            let s = (PI * t).sin();
            Pose {
                shift: 1.4 * a * sway,
                lift: 0.25 * a * hop(t),
                lean: 0.1 * sway,
                knee_l: 0.8 * s.max(0.0),
                knee_r: 0.8 * (-s).max(0.0),
                arm_l: 1.0 + 0.4 * a * s,
                bend_l: 1.8,
                arm_r: 1.0 - 0.4 * a * s,
                bend_r: 1.8,
                hair: 0.8 * sway,
                eyes: Eyes::Open,
                mouth: Mouth::Smile,
                ..base
            }
        }
        3 => {
            let clap = land(t);
            Pose {
                lift: 0.35 * a * hop(t),
                squash: 0.25 * a * clap,
                arm_l: 0.35,
                bend_l: -1.3 - 0.6 * clap,
                arm_r: 0.35,
                bend_r: -1.3 - 0.6 * clap,
                knee_l: 0.5 * hop(t),
                knee_r: 0.5 * hop(t),
                hair: 0.5 * wave(t, 2.0),
                eyes: Eyes::Happy,
                mouth: if clap > 0.5 { Mouth::Open } else { Mouth::Smile },
                ..base
            }
        }
        4 => {
            let sway = (PI * t / 2.0).sin();
            Pose {
                shift: 0.8 * a * sway,
                lean: 0.14 * a * sway,
                head_tilt: -0.12 * sway,
                squash: 0.15 * a * land(t),
                arm_l: 0.75,
                bend_l: -1.5,
                arm_r: 0.75,
                bend_r: -1.5,
                knee_l: 0.5 * sway.max(0.0),
                knee_r: 0.5 * (-sway).max(0.0),
                hair: 1.0 * sway,
                eyes: Eyes::Happy,
                mouth: Mouth::Smile,
                blush: 0.8,
                ..base
            }
        }
        5 => {
            let s = (PI * t).sin();
            Pose {
                lift: 0.5 * a * hop(t),
                squash: 0.2 * a * land(t),
                arm_l: 1.6 + 0.9 * a * s,
                bend_l: 0.4,
                arm_r: 1.6 - 0.9 * a * s,
                bend_r: 0.4,
                knee_l: 0.6 * hop(t),
                knee_r: 0.6 * hop(t),
                hair: 0.7 * s,
                eyes: Eyes::Open,
                mouth: Mouth::Open,
                ..base
            }
        }
        6 => {
            let h = hop(t);
            Pose {
                lift: 2.3 * a * h,
                squash: 0.5 * a * land(t),
                arm_l: 2.7 + 0.3 * (TAU * t).sin(),
                bend_l: 0.1,
                arm_r: 2.7 - 0.3 * (TAU * t).sin(),
                bend_r: 0.1,
                leg_l: 0.1 + 0.3 * h,
                leg_r: 0.1 + 0.3 * h,
                knee_l: h,
                knee_r: h,
                hair: 0.9 * h,
                eyes: Eyes::Happy,
                mouth: Mouth::Open,
                blush: 0.8,
                ..base
            }
        }
        _ => {
            let h = hop(t);
            Pose {
                spin: TAU * t / 2.0,
                lift: 1.0 * a * h,
                squash: 0.2 * a * land(t),
                arm_l: 1.5,
                bend_l: 0.1,
                arm_r: 1.5,
                bend_r: 0.1,
                knee_r: h,
                hair: 1.3,
                eyes: Eyes::Happy,
                mouth: Mouth::Open,
                ..base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_dance_is_finite_and_bounded() {
        for idx in 0..COUNT {
            for i in 0..80 {
                let p = dance(idx, i as f32 * 0.1, 1.0);
                for v in [p.lift, p.shift, p.squash, p.lean, p.arm_l, p.arm_r, p.knee_l, p.hair, p.spin] {
                    assert!(v.is_finite(), "{} produjo un valor no finito", name(idx));
                }
                assert!(p.lift <= 2.5, "{} salta demasiado", name(idx));
            }
        }
    }

    #[test]
    fn loops_are_continuous_at_the_period() {
        // El giro y el salto cierran el ciclo en su periodo (2 y 1 pulsos): sin tirones al encadenar.
        let a = dance(6, 0.0, 1.0);
        let b = dance(6, 1.0, 1.0);
        assert!((a.lift - b.lift).abs() < 1e-3 && (a.arm_l - b.arm_l).abs() < 1e-3);
    }
}
