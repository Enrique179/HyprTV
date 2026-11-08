//! Cómo se mueve el Slayer. Mismo esqueleto que las demás mascotas, pero nada de saltitos ni sonrisas: planta ancha, puños apretados, pasos de
//! tanque y en vez de bailar hace lo suyo (cabezazos al ritmo, puñetazos, motosierra, recarga de escopeta, flexión, patada, doble salto y
//! «dash»). Las expresiones no se ven (hay casco): `Eyes::Closed` apaga el visor, `Wide` lo enciende y `Happy` es «está hablando».

use crate::pose::{Eyes, Mouth, Moveset, Pose};
use std::f32::consts::PI;

const TAU: f32 = 2.0 * PI;

pub const SLAYER: Moveset = Moveset {
    idle: gestures::idle,
    sit: gestures::sit,
    sleep: gestures::sleep,
    react: gestures::react,
    yawn: gestures::stretch,
    wave: gestures::salute,
    walk: gestures::march,
    point: gestures::point,
    look: gestures::look,
    dance,
    dance_count: COUNT,
};

pub const COUNT: usize = 8;

pub fn name(idx: usize) -> &'static str {
    ["cabezazos", "puñetazos", "motosierra", "recarga", "músculos", "patada", "doble salto", "dash"][idx % COUNT]
}

fn frac(t: f32) -> f32 {
    t - t.floor()
}

/// 0 en el golpe, 1 a mitad del pulso.
fn hop(t: f32) -> f32 {
    (PI * frac(t)).sin()
}

/// 1 en el golpe y cae rápido: el impacto de una pisada o de un cabezazo.
fn land(t: f32) -> f32 {
    let c = 0.5 + 0.5 * (TAU * t).cos();
    c * c * c
}

/// Base de pie: las piernas abiertas, como las de la figurita.
fn stance() -> Pose {
    Pose { leg_l: 0.22, leg_r: 0.22, arm_l: 0.32, arm_r: 0.32, bend_l: 0.5, bend_r: 0.5, eyes: Eyes::Open, mouth: Mouth::Pout, blush: 0.0, ..Pose::default() }
}

pub mod gestures {
    use super::*;

    /// En guardia: pecho que sube y baja despacio y la cabeza girando como quien busca demonios.
    pub fn idle(time: f32) -> Pose {
        Pose {
            squash: 0.05 * (0.5 + 0.5 * (time * 1.6).sin()),
            head_tilt: 0.05 * (time * 0.5).sin(),
            head_nod: 0.08,
            arm_l: 0.32 + 0.03 * (time * 1.1).sin(),
            arm_r: 0.32 + 0.03 * (time * 1.1 + 1.0).sin(),
            ..stance()
        }
    }

    /// Sentado con los puños sobre las rodillas. Descansando respira más hondo.
    pub fn sit(time: f32, resting: bool) -> Pose {
        let breath = if resting { 0.07 } else { 0.03 };
        Pose {
            sit: 1.0,
            head_nod: 0.15,
            squash: breath * (time * 1.5).sin(),
            leg_l: 1.3,
            leg_r: 1.3,
            knee_l: 0.1,
            knee_r: 0.1,
            arm_l: 0.7,
            bend_l: 0.7,
            arm_r: 0.7,
            bend_r: 0.7,
            eyes: Eyes::Open,
            mouth: Mouth::Pout,
            blush: 0.0,
            ..Pose::default()
        }
    }

    /// Duerme sentado, con la cabeza caída y los brazos cruzados (como en el sarcófago).
    pub fn sleep(time: f32) -> Pose {
        Pose {
            sit: 1.0,
            head_nod: 0.9,
            lean: 0.05,
            squash: 0.04 * (time * 1.2).sin(),
            leg_l: 1.3,
            leg_r: 1.3,
            arm_l: 0.8,
            bend_l: -1.5,
            arm_r: 0.8,
            bend_r: -1.5,
            eyes: Eyes::Closed,
            mouth: Mouth::Pout,
            blush: 0.0,
            ..Pose::default()
        }
    }

    /// Tras un comentario: un rugido corto. Se echa hacia atrás con los puños en alto, da un pisotón y se queda en guardia.
    pub fn react(age: f32) -> Pose {
        let roar = (PI * (age / 0.5).clamp(0.0, 1.0)).sin();
        let stomp = if age < 0.5 { 0.0 } else { 0.45 * land((age - 0.5) * 1.3) };
        Pose {
            lift: 0.8 * roar,
            squash: stomp,
            lean: -0.12 * roar,
            head_nod: -0.5 * roar,
            arm_l: 0.4 + 1.1 * roar,
            bend_l: 0.5 + 0.9 * roar,
            arm_r: 0.4 + 1.1 * roar,
            bend_r: 0.5 + 0.9 * roar,
            knee_l: 0.4 * roar,
            knee_r: 0.4 * roar,
            eyes: if age < 0.5 { Eyes::Wide } else { Eyes::Open },
            ..stance()
        }
    }

    /// Crujir el cuello y los nudillos: giro de cabeza, brazos cruzados delante y vuelta a la guardia.
    pub fn stretch(age: f32) -> Pose {
        let k = (PI * (age / 1.6).clamp(0.0, 1.0)).sin();
        Pose {
            head_tilt: 0.3 * k * (TAU * age / 0.8).sin(),
            lean: -0.04 * k,
            arm_l: 0.35 + 0.55 * k,
            bend_l: 0.5 - 2.2 * k,
            arm_r: 0.35 + 0.55 * k,
            bend_r: 0.5 - 2.2 * k,
            squash: 0.1 * k,
            ..stance()
        }
    }

    /// Saludo del Slayer: puño en alto, firme, con un asentimiento seco de vez en cuando. Nada de manitas.
    pub fn salute(time: f32) -> Pose {
        Pose {
            arm_r: 1.6 + 0.04 * (time * 5.0).sin(),
            bend_r: 1.5,
            head_nod: 0.35 * (time * 2.5).sin().max(0.0),
            lift: 0.0,
            ..stance()
        }
    }

    /// Marcha de tanque: pasos pesados y lentos, cuerpo ligeramente inclinado hacia delante. A `speed` alta echa a correr.
    pub fn march(time: f32, speed: f32, dir: f32) -> Pose {
        let run = speed > 1.5;
        let s = (time * if run { 8.0 } else { 6.0 } * speed.min(1.3)).sin();
        Pose {
            lift: if run { 0.3 } else { 0.1 } * s.abs(),
            squash: 0.12 * s.abs(),
            lean: (if run { 0.22 } else { 0.1 }) * dir + 0.03 * s,
            knee_l: 0.7 * s.max(0.0),
            knee_r: 0.7 * (-s).max(0.0),
            arm_l: 0.4 + (if run { 0.7 } else { 0.22 }) * s,
            arm_r: 0.4 - (if run { 0.7 } else { 0.22 }) * s,
            bend_l: 0.7,
            bend_r: 0.7,
            head_nod: 0.1,
            ..stance()
        }
    }

    /// Señala con el brazo estirado (el dedo en vez de la manita) y el otro puño en la cadera.
    pub fn point(time: f32, side: f32) -> Pose {
        let up = 2.2 + 0.05 * (time * 4.0).sin();
        let left = side < 0.0;
        Pose {
            lean: 0.1 * side,
            head_tilt: 0.08 * side,
            arm_l: if left { up } else { 0.75 },
            bend_l: if left { 0.1 } else { -1.5 },
            arm_r: if left { 0.75 } else { up },
            bend_r: if left { -1.5 } else { 0.1 },
            eyes: Eyes::Wide,
            ..stance()
        }
    }

    /// Vigila: la cabeza gira hacia un lado con los puños listos y el visor se apaga un instante, como un parpadeo.
    pub fn look(time: f32, side: f32) -> Pose {
        Pose {
            head_tilt: 0.2 * side,
            lean: 0.04 * side,
            squash: 0.04 * (time * 2.0).sin(),
            eyes: if frac(time * 0.4) > 0.92 { Eyes::Closed } else { Eyes::Open },
            ..stance()
        }
    }
}

/// Las 8 coreografías, en pulsos (`t`) y con amplitud `a`. Todas cierran el ciclo en 1, 2 o 4 pulsos, así que se encadenan sin saltos.
pub fn dance(idx: usize, t: f32, a: f32) -> Pose {
    let base = stance();
    match idx % COUNT {
        // Cabezazos al ritmo: la cabeza cae en cada golpe y el cuerpo se hunde con ella.
        0 => {
            let hit = land(t);
            Pose {
                head_nod: 0.95 * hit - 0.15,
                lean: 0.1 * a * hit,
                squash: 0.22 * a * hit,
                lift: 0.12 * a * hop(t),
                knee_l: 0.35 * hit,
                knee_r: 0.35 * hit,
                arm_l: 0.45 + 0.1 * hit,
                arm_r: 0.45 + 0.1 * hit,
                leg_l: 0.3,
                leg_r: 0.3,
                ..base
            }
        }
        // Puñetazos alternos hacia arriba, con un giro de cadera.
        1 => {
            let s = (PI * t).sin();
            let (pl, pr) = (s.max(0.0), (-s).max(0.0));
            Pose {
                shift: 0.3 * a * s,
                lean: 0.14 * a * s,
                squash: 0.15 * a * land(t),
                arm_l: 0.5 + 1.9 * a * pl,
                bend_l: 0.2 + 0.6 * (1.0 - pl),
                arm_r: 0.5 + 1.9 * a * pr,
                bend_r: 0.2 + 0.6 * (1.0 - pr),
                knee_l: 0.3 * pl,
                knee_r: 0.3 * pr,
                eyes: if pl.max(pr) > 0.8 { Eyes::Wide } else { Eyes::Open },
                ..base
            }
        }
        // Motosierra: los dos puños al frente, vibrando, y un barrido de lado a lado con todo el cuerpo.
        2 => {
            let sweep = (PI * t / 2.0).sin();
            let jitter = 0.05 * (TAU * 6.0 * t).sin();
            Pose {
                shift: 0.8 * a * sweep,
                lean: 0.18 * a * sweep,
                squash: 0.1 + 0.05 * (TAU * 6.0 * t).sin().abs(),
                arm_l: 0.85 + jitter,
                bend_l: -1.0,
                arm_r: 0.85 - jitter,
                bend_r: -1.0,
                knee_l: 0.3,
                knee_r: 0.3,
                head_nod: 0.2,
                eyes: Eyes::Wide,
                ..base
            }
        }
        // Recarga de escopeta: una mano sostiene, la otra bombea dos veces y la cabeza asiente al cerrar.
        3 => {
            let pump = (TAU * t).sin();
            Pose {
                arm_l: 0.9,
                bend_l: -0.9,
                arm_r: 0.8 + 0.45 * a * pump,
                bend_r: -0.9,
                head_nod: 0.3 * pump.max(0.0),
                squash: 0.08 * land(t),
                lean: -0.04,
                ..base
            }
        }
        // Músculos: los dos brazos en jarra hacia arriba, el pecho se hincha con cada pulso y la cabeza se alza.
        4 => {
            let beat = land(t);
            Pose {
                arm_l: 1.45 + 0.12 * beat,
                bend_l: 1.5,
                arm_r: 1.45 + 0.12 * beat,
                bend_r: 1.5,
                squash: -0.06 * beat,
                head_nod: -0.25,
                leg_l: 0.38,
                leg_r: 0.38,
                knee_l: 0.15 * beat,
                knee_r: 0.15 * beat,
                eyes: if beat > 0.6 { Eyes::Wide } else { Eyes::Open },
                ..base
            }
        }
        // Patadas laterales alternas, con los brazos de contrapeso.
        5 => {
            let s = (PI * t).sin();
            let (kl, kr) = (s.max(0.0), (-s).max(0.0));
            Pose {
                lift: 0.3 * a * s.abs(),
                lean: 0.12 * a * s,
                shift: -0.4 * a * s,
                leg_l: 0.22 + 1.2 * a * kl,
                knee_l: 0.3 * kl,
                leg_r: 0.22 + 1.2 * a * kr,
                knee_r: 0.3 * kr,
                arm_l: 1.0 + 0.5 * kr,
                bend_l: 0.5,
                arm_r: 1.0 + 0.5 * kl,
                bend_r: 0.5,
                ..base
            }
        }
        // Doble salto: dos saltos por ciclo, el puño en alto en cada uno y un aterrizaje que se nota.
        6 => {
            let h = hop(t);
            let first = frac(t / 2.0) < 0.5;
            Pose {
                lift: 2.1 * a * h,
                squash: 0.5 * a * land(t),
                arm_l: if first { 2.9 } else { 0.5 },
                bend_l: 0.1,
                arm_r: if first { 0.5 } else { 2.9 },
                bend_r: 0.1,
                leg_l: 0.22 + 0.25 * h,
                leg_r: 0.22 + 0.25 * h,
                knee_l: 0.9 * h,
                knee_r: 0.9 * h,
                eyes: if h > 0.8 { Eyes::Wide } else { Eyes::Open },
                ..base
            }
        }
        // Dash: un giro rápido de brazos extendidos, inclinado, como cuando se lanza contra un demonio.
        _ => {
            let h = hop(t);
            Pose {
                spin: TAU * t / 2.0,
                lift: 0.7 * a * h,
                lean: 0.2,
                squash: 0.2 * a * land(t),
                arm_l: 1.55,
                bend_l: 0.05,
                arm_r: 1.55,
                bend_r: 0.05,
                knee_r: 0.8 * h,
                eyes: Eyes::Wide,
                ..base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_dance_is_finite_bounded_and_loops_in_four_beats() {
        for idx in 0..COUNT {
            for i in 0..80 {
                let t = i as f32 * 0.1;
                let p = dance(idx, t, 0.9);
                for v in [p.lift, p.shift, p.squash, p.lean, p.arm_l, p.arm_r, p.knee_l, p.leg_l, p.head_nod, p.spin] {
                    assert!(v.is_finite(), "{} produjo un valor no finito", name(idx));
                }
                assert!(p.lift <= 2.5 && p.lift >= 0.0, "{} sale de rango", name(idx));
                // Todas cierran el ciclo en 4 pulsos como mucho.
                let q = dance(idx, t + 4.0, 0.9);
                assert!((p.lift - q.lift).abs() < 1e-2 && (p.arm_l - q.arm_l).abs() < 1e-2 && (p.shift - q.shift).abs() < 1e-2, "{} no cierra el ciclo", name(idx));
                assert!((p.spin.cos() - q.spin.cos()).abs() < 1e-2, "{} no cierra el giro", name(idx));
            }
        }
    }

    #[test]
    fn gestures_are_finite_and_the_visor_is_the_only_expression() {
        for i in 0..60 {
            let t = i as f32 * 0.13;
            let all = [
                gestures::idle(t),
                gestures::sit(t, true),
                gestures::sleep(t),
                gestures::react(t / 4.0),
                gestures::stretch(t / 4.0),
                gestures::salute(t),
                gestures::march(t, 1.0, 1.0),
                gestures::march(t, 2.4, -1.0),
                gestures::point(t, 1.0),
                gestures::look(t, -1.0),
            ];
            for p in all {
                assert!(p.lift.is_finite() && p.arm_l.is_finite() && p.lean.is_finite() && p.squash.is_finite());
                assert!(p.eyes != Eyes::Happy && p.eyes != Eyes::Wink, "el Slayer no sonríe ni guiña");
            }
        }
    }
}
