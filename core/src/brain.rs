//! Qué hace la mascota. Máquina de estados pensada para una TV con poca RAM: anima solo cuando pasa algo y se queda quieta
//! (sin pedir más fotogramas) en cuanto se duerme. Así el reposo cuesta 0 % de CPU.
//!
//! Dos vidas: la "de servicio" (saluda, reacciona a un chiste, bosteza y se duerme en su esquina) y la "rutina de
//! aburrimiento", que arranca desde fuera cuando la TV lleva un rato sin usarse: pasea por la pantalla, baila sin música,
//! señala lo que hay enfocado, saluda al vacío... y al acabar (o en cuanto alguien toca el mando) vuelve a su esquina.

use crate::moves;
use crate::pose::{gestures, Eyes, Moveset, Mouth, Pose, CHIBI};
use crate::render::{Particle, ParticleKind};

/// Milisegundos entre fotogramas mientras anima (~12 fps).
const FRAME_MS: u32 = 80;
const IDLE_MS: u64 = 7_000;
const GREET_MS: u64 = 1_600;
const REACT_MS: u64 = 1_400;
const YAWN_MS: u64 = 1_600;
const SIT_MS: u64 = 5_000;

/// Velocidades en fracciones de pantalla por segundo.
const WALK_SPEED: f32 = 0.085;
const RETURN_SPEED: f32 = 0.24;

// Lo que cuenta `take_event` (Kotlin lo traduce a una frase).
pub const EV_NONE: u8 = 0;
pub const EV_BORED: u8 = 1;
pub const EV_WALK: u8 = 2;
pub const EV_DANCE: u8 = 3;
pub const EV_POINT: u8 = 4;
pub const EV_WAVE: u8 = 5;
pub const EV_REST: u8 = 6;
pub const EV_LOOK: u8 = 7;
pub const EV_STRETCH: u8 = 8;
pub const EV_TIRED: u8 = 9;
pub const EV_BACK: u8 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Greet,
    React,
    Idle,
    Yawn,
    Sit,
    Sleep,
    /// Camina hacia `target`; al llegar sigue la rutina (o señala, si iba a eso).
    Walk,
    Dance,
    Point,
    Wave,
    Look,
    Rest,
    Stretch,
    /// Vuelve a su esquina a paso rápido.
    Return,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum After {
    /// Tras volver: saluda (alguien usó la TV).
    Greet,
    /// Tras volver: se sienta y se duerme (acabó su rutina).
    Sleep,
}

pub struct Brain {
    mode: Mode,
    since: u64,
    started: u64,
    last_tick: u64,
    talking_until: u64,
    gaze: f32,
    pub pose: Pose,
    pub particles: Vec<Particle>,
    seed: u32,
    moves: &'static Moveset,

    // Posición horizontal del centro de la mascota (fracción del ancho de pantalla).
    x: f32,
    home: f32,
    lo: f32,
    hi: f32,
    target: f32,
    dir: f32,
    focus_x: Option<f32>,

    // Rutina de aburrimiento.
    routine_end: u64,
    act_len: u64,
    dance_idx: usize,
    last_act: Mode,
    point_after_walk: bool,
    after_return: After,
    event: u8,
}

impl Brain {
    pub fn new(now: u64) -> Brain {
        Brain {
            mode: Mode::Sleep,
            since: now,
            started: now,
            last_tick: now,
            talking_until: 0,
            gaze: 1.0,
            pose: gestures::sleep(0.0),
            particles: Vec::new(),
            seed: (now as u32) | 1,
            moves: &CHIBI,
            x: 0.9,
            home: 0.9,
            lo: 0.06,
            hi: 0.94,
            target: 0.9,
            dir: 1.0,
            focus_x: None,
            routine_end: 0,
            act_len: 0,
            dance_idx: 0,
            last_act: Mode::Sleep,
            point_after_walk: false,
            after_return: After::Sleep,
            event: EV_NONE,
        }
    }

    /// Elige el repertorio de gestos y bailes (según el modelo). Es barato: solo cambia un puntero.
    pub fn set_moveset(&mut self, m: &'static Moveset) {
        self.moves = m;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn in_routine(&self) -> bool {
        self.routine_end != 0
    }

    /// Lo último que empezó a hacer y todavía no se ha contado (una sola vez).
    pub fn take_event(&mut self) -> u8 {
        std::mem::replace(&mut self.event, EV_NONE)
    }

    fn rand(&mut self) -> f32 {
        // xorshift32: basta para variar los gestos.
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }

    fn enter(&mut self, mode: Mode, now: u64) {
        self.mode = mode;
        self.since = now;
        if mode != Mode::Sleep {
            self.particles.retain(|p| p.kind != ParticleKind::Zzz);
        }
    }

    /// Dónde está su esquina y entre qué límites puede pasear (fracciones del ancho de la pantalla).
    pub fn set_world(&mut self, home: f32, lo: f32, hi: f32, now: u64) {
        self.lo = lo.min(hi);
        self.hi = hi.max(lo);
        let home = home.clamp(self.lo, self.hi);
        let first = (self.x - self.home).abs() < 1e-4 && self.mode == Mode::Sleep;
        self.home = home;
        if first || self.mode == Mode::Sleep && !self.in_routine() {
            // Dormida o recién creada: simplemente está en su esquina.
            self.x = home;
        } else if !self.in_routine() && (self.x - home).abs() > 0.005 {
            // Si la cambian de esquina estando despierta, camina hasta allí.
            self.target = home;
            self.after_return = After::Greet;
            self.enter(Mode::Return, now);
        }
    }

    /// Hacia dónde está lo enfocado (para ir a señalarlo), o `None` si no se sabe.
    pub fn set_focus(&mut self, x: Option<f32>) {
        self.focus_x = x.map(|v| v.clamp(self.lo, self.hi));
    }

    /// La pantalla cambió (otra app, otro menú): se despierta y saluda.
    pub fn wake(&mut self, now: u64) {
        self.gaze = if self.rand() > 0.5 { 1.0 } else { -1.0 };
        if self.in_routine() || self.mode == Mode::Return {
            self.interrupt(now);
            return;
        }
        match self.mode {
            Mode::Sleep | Mode::Sit | Mode::Yawn => {
                self.enter(Mode::Greet, now);
                self.burst(ParticleKind::Heart, 1);
            }
            Mode::Idle => self.since = now,
            _ => {}
        }
    }

    /// Alguien volvió a usar la TV: se corta la rutina y vuelve a su esquina. Barato si no hay rutina.
    pub fn interrupt(&mut self, now: u64) {
        if !self.in_routine() && self.mode != Mode::Return {
            return;
        }
        let was_routine = self.in_routine();
        self.routine_end = 0;
        self.point_after_walk = false;
        self.after_return = After::Greet;
        if was_routine {
            self.event = EV_BACK;
        }
        if (self.x - self.home).abs() > 0.01 {
            self.target = self.home;
            self.enter(Mode::Return, now);
        } else {
            self.enter(Mode::Greet, now);
        }
    }

    /// Acaba de decir algo: reacciona (salta contenta) y mueve la boca mientras dura el bocadillo.
    pub fn talk(&mut self, now: u64, ms: u32) {
        self.talking_until = now + ms as u64;
        self.burst(ParticleKind::Spark, 3);
        // En plena rutina o de camino no se interrumpe lo que hacía: solo mueve la boca.
        if !self.in_routine() && self.mode != Mode::Return {
            self.enter(Mode::React, now);
        }
    }

    /// Empieza la rutina de aburrimiento durante `ms` milisegundos. No hace nada si ya está en una.
    pub fn begin_routine(&mut self, now: u64, ms: u64) {
        if self.in_routine() || self.mode == Mode::Return {
            return;
        }
        self.routine_end = now + ms;
        self.event = EV_BORED;
        self.last_act = Mode::Sleep;
        self.start_activity(Mode::Stretch, now);
    }

    fn burst(&mut self, kind: ParticleKind, n: usize) {
        for i in 0..n {
            let a = self.rand();
            let b = self.rand();
            let side = if i % 2 == 0 { -1.0 } else { 1.0 };
            self.particles.push(Particle {
                kind,
                x: side * (1.5 + 2.0 * a),
                y: 9.0 + 2.0 * b,
                vx: side * (0.4 + 0.8 * a),
                vy: 1.8 + 1.2 * b,
                life: 1.1,
                age: 0.0,
            });
        }
        // Que no se acumulen si se despierta muchas veces seguidas.
        if self.particles.len() > 12 {
            let extra = self.particles.len() - 12;
            self.particles.drain(0..extra);
        }
    }

    // ---- rutina ------------------------------------------------------------------------------

    fn start_activity(&mut self, mode: Mode, now: u64) {
        self.last_act = mode;
        let (len, event) = match mode {
            Mode::Dance => {
                let count = self.moves.dance_count;
                self.dance_idx = (self.rand() * count as f32) as usize % count;
                (6_000 + (self.rand() * 4_000.0) as u64, EV_DANCE)
            }
            Mode::Point => (3_800, EV_POINT),
            Mode::Wave => (2_600, EV_WAVE),
            Mode::Look => (3_600, EV_LOOK),
            Mode::Rest => (4_200, EV_REST),
            Mode::Stretch => (1_800, EV_STRETCH),
            _ => (0, EV_NONE),
        };
        self.act_len = len;
        // El primer movimiento del aburrimiento ya anuncia "me aburro": no se pisa con otro aviso.
        if event != EV_NONE && self.event == EV_NONE {
            self.event = event;
        }
        self.enter(mode, now);
    }

    fn start_walk(&mut self, target: f32, then_point: bool, now: u64) {
        self.target = target.clamp(self.lo, self.hi);
        self.point_after_walk = then_point;
        if !then_point && self.event == EV_NONE {
            self.event = EV_WALK;
        }
        self.dir = if self.target >= self.x { 1.0 } else { -1.0 };
        self.last_act = Mode::Walk;
        self.enter(Mode::Walk, now);
    }

    /// Elige la siguiente cosa que hacer, sin repetir la anterior; al acabar el tiempo de la rutina vuelve a casa cansada.
    fn next_activity(&mut self, now: u64) {
        if now >= self.routine_end {
            self.routine_end = 0;
            self.event = EV_TIRED;
            self.after_return = After::Sleep;
            if (self.x - self.home).abs() > 0.01 {
                self.target = self.home;
                self.enter(Mode::Return, now);
            } else {
                self.enter(Mode::Sit, now);
            }
            return;
        }
        // (actividad, peso). Señalar solo si se sabe qué está enfocado y queda lejos de donde estoy.
        let can_point = self.focus_x.is_some_and(|f| self.last_act != Mode::Point && (f - self.x).abs() > 0.05);
        let options: [(Mode, f32); 6] = [
            (Mode::Walk, 3.0),
            (Mode::Dance, 3.0),
            (Mode::Point, if can_point { 2.5 } else { 0.0 }),
            (Mode::Wave, 1.5),
            (Mode::Look, 1.5),
            (Mode::Rest, 1.5),
        ];
        let total: f32 = options.iter().filter(|(m, w)| *m != self.last_act && *w > 0.0).map(|(_, w)| w).sum();
        let mut pick = self.rand() * total;
        let mut chosen = Mode::Wave;
        for (m, w) in options {
            if m == self.last_act || w <= 0.0 {
                continue;
            }
            if pick < w {
                chosen = m;
                break;
            }
            pick -= w;
        }
        match chosen {
            Mode::Walk => {
                // Un destino lejos de donde está: media pantalla para que el paseo se note.
                let span = self.hi - self.lo;
                let mut t = self.lo + self.rand() * span;
                if (t - self.x).abs() < span * 0.25 {
                    t = if self.x > (self.lo + self.hi) / 2.0 { self.lo + self.rand() * span * 0.35 } else { self.hi - self.rand() * span * 0.35 };
                }
                self.start_walk(t, false, now);
            }
            Mode::Point => {
                let f = self.focus_x.unwrap_or(self.x);
                // Se queda a un lado del elemento, no justo debajo, para verlo.
                let off = if f > 0.5 { -0.07 } else { 0.07 };
                self.start_walk(f + off, true, now);
            }
            other => self.start_activity(other, now),
        }
    }

    fn move_towards(&mut self, target: f32, speed: f32, dt: f32) -> bool {
        let step = speed * dt;
        let d = target - self.x;
        if d.abs() <= step {
            self.x = target;
            true
        } else {
            self.x += step * d.signum();
            false
        }
    }

    /// Avanza el estado. Devuelve (milisegundos hasta el siguiente fotograma, o 0 si ya no hace falta ninguno; ¿hay que
    /// volver a dibujar?).
    pub fn tick(&mut self, now: u64) -> (u32, bool) {
        let dt = (now.saturating_sub(self.last_tick)) as f32 / 1000.0;
        self.last_tick = now;
        let t = (now.saturating_sub(self.started)) as f32 / 1000.0;
        let waited = now.saturating_sub(self.since);

        // Transiciones por tiempo o por llegar a un sitio.
        match self.mode {
            Mode::Greet if waited >= GREET_MS => self.enter(Mode::Idle, now),
            Mode::React if waited >= REACT_MS => self.enter(Mode::Idle, now),
            Mode::Idle if waited >= IDLE_MS && now >= self.talking_until => self.enter(Mode::Yawn, now),
            Mode::Yawn if waited >= YAWN_MS => self.enter(Mode::Sit, now),
            Mode::Sit if waited >= SIT_MS => {
                self.enter(Mode::Sleep, now);
                // Dos "z" fijas: en reposo no se redibuja, así que no se mueven.
                self.particles.clear();
                self.particles.push(Particle { kind: ParticleKind::Zzz, x: 2.6, y: 10.2, vx: 0.0, vy: 0.0, life: 1.0, age: 0.3 });
                self.particles.push(Particle { kind: ParticleKind::Zzz, x: 3.6, y: 11.6, vx: 0.0, vy: 0.0, life: 1.0, age: 0.6 });
            }
            Mode::Walk => {
                let target = self.target;
                if self.move_towards(target, WALK_SPEED, dt) {
                    if self.point_after_walk {
                        self.point_after_walk = false;
                        self.start_activity(Mode::Point, now);
                    } else {
                        self.next_activity(now);
                    }
                }
            }
            Mode::Return => {
                let target = self.target;
                let dirn = if target >= self.x { 1.0 } else { -1.0 };
                self.dir = dirn;
                if self.move_towards(target, RETURN_SPEED, dt) {
                    match self.after_return {
                        After::Greet => {
                            self.enter(Mode::Greet, now);
                            self.burst(ParticleKind::Heart, 1);
                        }
                        After::Sleep => self.enter(Mode::Sit, now),
                    }
                }
            }
            Mode::Dance | Mode::Point | Mode::Wave | Mode::Look | Mode::Rest | Mode::Stretch if waited >= self.act_len => {
                self.next_activity(now)
            }
            _ => {}
        }
        let age_ms = now.saturating_sub(self.since);
        let age = age_ms as f32 / 1000.0;

        let m = self.moves;
        let mut pose = match self.mode {
            Mode::Greet => Pose::lerp(&(m.wave)(t), &(m.idle)(t), (age_ms as f32 / GREET_MS as f32 - 0.7).max(0.0) / 0.3),
            Mode::React => (m.react)(age),
            Mode::Idle => {
                // Mira un lado y otro: parece que lee el menú.
                let look = (t * 0.5).sin().signum() * self.gaze;
                Pose::lerp(&(m.idle)(t), &(m.look)(t, look), 0.7)
            }
            Mode::Yawn | Mode::Stretch => (m.yawn)(age),
            Mode::Sit | Mode::Rest => (m.sit)(t, self.mode == Mode::Rest),
            Mode::Sleep => (m.sleep)(0.0),
            Mode::Walk => (m.walk)(t, 1.0, self.dir),
            Mode::Return => (m.walk)(t, 2.4, self.dir),
            Mode::Dance => (m.dance)(self.dance_idx, age / moves::BEAT_SECS, 0.9),
            Mode::Point => {
                let side = if self.focus_x.unwrap_or(self.x) >= self.x { 1.0 } else { -1.0 };
                (m.point)(t, side)
            }
            Mode::Wave => (m.wave)(t),
            Mode::Look => {
                let look = (t * 0.7).sin().signum();
                (m.look)(t, look)
            }
        };
        if now < self.talking_until && self.mode != Mode::Sleep {
            // Habla: la boca se abre y se cierra al ritmo de las sílabas.
            pose.mouth = if (now / 140).is_multiple_of(2) { Mouth::Open } else { Mouth::Small };
            if pose.eyes == Eyes::Open {
                pose.eyes = Eyes::Happy;
            }
        }
        self.pose = pose;

        // Partículas flotantes (las "z" del sueño no se mueven).
        for p in &mut self.particles {
            if p.kind != ParticleKind::Zzz {
                p.x += p.vx * dt;
                p.y += p.vy * dt;
                p.age += dt;
            }
        }
        self.particles.retain(|p| p.kind == ParticleKind::Zzz || p.age < p.life);

        if self.mode == Mode::Sleep {
            // Último fotograma: la mascota duerme y no se vuelve a pedir ninguno hasta el próximo `wake`.
            (0, true)
        } else {
            (FRAME_MS, true)
        }
    }

    /// Segundos transcurridos desde que existe (para detalles pequeños: pelo, brillo).
    pub fn time(&self, now: u64) -> f32 {
        now.saturating_sub(self.started) as f32 / 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_until_stop(b: &mut Brain, mut now: u64) -> u64 {
        for _ in 0..4000 {
            let (next, _) = b.tick(now);
            if next == 0 {
                return now;
            }
            now += next as u64;
        }
        panic!("la mascota nunca se duerme");
    }

    #[test]
    fn falls_asleep_and_stops_asking_for_frames() {
        let mut b = Brain::new(0);
        b.wake(100);
        assert_eq!(b.mode(), Mode::Greet);
        let end = run_until_stop(&mut b, 100);
        assert_eq!(b.mode(), Mode::Sleep);
        // Tarda lo de saludar + reposo + bostezo + sentada, no menos.
        assert!(end >= 100 + IDLE_MS, "se durmió demasiado pronto: {end}");
        // Dormida: sigue sin pedir fotogramas hasta que algo la despierte.
        assert_eq!(b.tick(end + 60_000).0, 0);
    }

    #[test]
    fn talking_keeps_it_awake_and_wakes_again() {
        let mut b = Brain::new(0);
        run_until_stop(&mut b, 0);
        b.talk(50_000, 5_000);
        assert_eq!(b.mode(), Mode::React);
        // Mientras habla no se va a bostezar aunque pase el tiempo de reposo.
        let mut now = 50_000;
        for _ in 0..40 {
            now += 100;
            b.tick(now);
        }
        assert_ne!(b.mode(), Mode::Yawn);
        let end = run_until_stop(&mut b, now);
        assert!(end > 50_000 + 5_000);
    }

    #[test]
    fn particles_are_bounded() {
        let mut b = Brain::new(0);
        for i in 0..100 {
            b.talk(i * 10, 100);
        }
        assert!(b.particles.len() <= 12);
    }

    fn world() -> Brain {
        let mut b = Brain::new(0);
        b.set_world(0.9, 0.08, 0.92, 0);
        b
    }

    #[test]
    fn routine_roams_stays_in_bounds_and_comes_home_to_sleep() {
        let mut b = world();
        b.set_focus(Some(0.3));
        b.begin_routine(1_000, 40_000);
        assert!(b.in_routine());
        assert_eq!(b.take_event(), EV_BORED);
        let (mut now, mut min_x, mut max_x) = (1_000u64, 1.0f32, 0.0f32);
        let mut events = Vec::new();
        for _ in 0..6000 {
            let (next, _) = b.tick(now);
            min_x = min_x.min(b.x());
            max_x = max_x.max(b.x());
            let e = b.take_event();
            if e != EV_NONE {
                events.push(e);
            }
            if next == 0 {
                break;
            }
            now += next as u64;
        }
        assert_eq!(b.mode(), Mode::Sleep, "al acabar la rutina se duerme");
        assert!(!b.in_routine());
        assert!((b.x() - 0.9).abs() < 0.01, "duerme en su esquina: {}", b.x());
        assert!(min_x >= 0.08 - 1e-4 && max_x <= 0.92 + 1e-4, "se salió de los límites: {min_x}..{max_x}");
        assert!(min_x < 0.7, "debería haber paseado, no quedarse en la esquina: {min_x}");
        assert!(events.contains(&EV_TIRED), "tiene que avisar de que se cansa: {events:?}");
        assert!(events.iter().filter(|e| **e == EV_WALK || **e == EV_DANCE || **e == EV_POINT).count() >= 2, "{events:?}");
        assert!(now > 40_000, "la rutina dura lo pedido");
    }

    #[test]
    fn user_activity_interrupts_and_sends_it_home_greeting() {
        let mut b = world();
        b.begin_routine(0, 120_000);
        let mut now = 0;
        // Se aleja de casa.
        for _ in 0..400 {
            now += 80;
            b.tick(now);
            if (b.x() - 0.9).abs() > 0.2 {
                break;
            }
        }
        assert!((b.x() - 0.9).abs() > 0.2, "debería haberse alejado");
        b.interrupt(now);
        assert!(!b.in_routine());
        assert_eq!(b.mode(), Mode::Return);
        assert_eq!(b.take_event(), EV_BACK);
        for _ in 0..200 {
            now += 80;
            b.tick(now);
            if b.mode() == Mode::Greet {
                break;
            }
        }
        assert_eq!(b.mode(), Mode::Greet);
        assert!((b.x() - 0.9).abs() < 0.01);
        // Interrumpir sin rutina no hace nada.
        b.interrupt(now + 10);
        assert_eq!(b.mode(), Mode::Greet);
    }

    #[test]
    fn speaking_does_not_cut_the_routine() {
        let mut b = world();
        b.begin_routine(0, 60_000);
        let mode = b.mode();
        b.talk(100, 3_000);
        assert_eq!(b.mode(), mode);
        assert!(b.in_routine());
    }

    #[test]
    fn point_walks_next_to_the_focus_and_points() {
        let mut b = world();
        b.set_focus(Some(0.2));
        b.begin_routine(0, 600_000);
        let (mut now, mut pointed) = (0u64, false);
        'outer: for _ in 0..20_000 {
            b.tick(now);
            if b.mode() == Mode::Point {
                pointed = true;
                assert!((b.x() - 0.2).abs() < 0.12, "señala cerca de lo enfocado: {}", b.x());
                break 'outer;
            }
            now += 80;
        }
        assert!(pointed, "con algo enfocado lejos acaba señalándolo");
    }

    #[test]
    fn moving_the_corner_while_awake_walks_there() {
        let mut b = world();
        b.wake(0);
        b.tick(100);
        b.set_world(0.1, 0.08, 0.92, 200);
        assert_eq!(b.mode(), Mode::Return);
        let mut now = 200;
        for _ in 0..300 {
            now += 80;
            b.tick(now);
            if b.mode() != Mode::Return {
                break;
            }
        }
        assert!((b.x() - 0.1).abs() < 0.01);
    }
}
