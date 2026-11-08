//! Núcleo de HyprTV: configuración y temas, la mascota (cerebro + dibujo), los chistes y los atajos del mando.
//! Kotlin solo hace de pegamento con Android (ventanas, accesibilidad, lanzador); aquí vive la lógica, que se prueba en el PC.

pub mod brain;
pub mod config;
pub mod jokes;
pub mod keys;
pub mod look;
pub mod slayer;
pub mod moves;
pub mod pose;
pub mod render;
pub mod render_slayer;
pub mod slayer_moves;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Mutex, OnceLock};

use brain::Brain;
use config::Config;
use jni::objects::{JByteBuffer, JClass, JString};
use jni::sys::{jint, jstring};
use jni::JNIEnv;
use jokes::{Jokes, Policy, Trigger};
use keys::{Action, Keys};

/// Bit que `pet_tick` activa en su resultado cuando hay un fotograma nuevo en el búfer.
pub const DIRTY: i32 = 1 << 30;
/// Lo que devuelve `pet_tick` en los 16 bits bajos: milisegundos hasta el siguiente fotograma (0 = no pedir más)...
pub const DELAY_MASK: i32 = 0xFFFF;
/// ...y en los 4 siguientes, lo que acaba de empezar a hacer (`brain::EV_*`), para que Kotlin le ponga una frase.
pub const EVENT_SHIFT: i32 = 16;

struct Core {
    cfg: Config,
    keys: Keys,
    jokes: Jokes,
    brain: Option<Brain>,
    size: (u32, u32),
}

fn combos_of(cfg: &Config) -> Vec<(Action, Vec<i32>)> {
    vec![
        (Action::TogglePet, cfg.ints("combo.pet")),
        (Action::ToggleTheme, cfg.ints("combo.theme")),
        (Action::ToggleRing, cfg.ints("combo.ring")),
        (Action::PetCorner, cfg.ints("combo.corner")),
        (Action::RingThisApp, cfg.ints("combo.ringapp")),
    ]
}

impl Core {
    fn new() -> Core {
        let cfg = Config::defaults();
        let mut keys = Keys::default();
        keys.set_combos(combos_of(&cfg));
        Core { cfg, keys, jokes: Jokes::default(), brain: None, size: (0, 0) }
    }

    fn set_config(&mut self, cfg: Config) {
        self.keys.set_combos(combos_of(&cfg));
        self.cfg = cfg;
    }

    fn policy(&self) -> Policy {
        Policy {
            chatter: self.cfg.get_i32("pet.chatter"),
            quiet: self.cfg.get_bool("pet.quiet"),
            quiet_start: self.cfg.get_i32("pet.quiet.start"),
            quiet_end: self.cfg.get_i32("pet.quiet.end"),
            muted: self.cfg.get("pet.muted").split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
            slayer: self.cfg.get("pet.look") == "slayer",
        }
    }
}

fn core() -> std::sync::MutexGuard<'static, Core> {
    static CORE: OnceLock<Mutex<Core>> = OnceLock::new();
    CORE.get_or_init(|| Mutex::new(Core::new())).lock().unwrap_or_else(|e| e.into_inner())
}

// ------------------------------------------------------------------------------------------------
// API en Rust puro (lo que prueban los tests); las funciones JNI de abajo solo traducen tipos.
// ------------------------------------------------------------------------------------------------

/// Lee `path` (si no existe usa los valores por defecto), lo valida y lo deja como configuración activa.
pub fn load_config(path: &str) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let cfg = Config::parse(&text);
    let dump = cfg.dump();
    core().set_config(cfg);
    dump
}

/// Valida `dump`, lo guarda en `path` y lo deja como configuración activa. Devuelve el texto normalizado.
pub fn save_config(path: &str, dump: &str) -> String {
    let cfg = Config::parse(dump);
    let out = cfg.dump();
    // Se escribe en un temporal y se renombra: un corte de luz no deja el archivo a medias.
    let tmp = format!("{path}.tmp");
    if std::fs::write(&tmp, &out).is_ok() {
        let _ = std::fs::rename(&tmp, path);
    }
    core().set_config(cfg);
    out
}

pub fn apply_preset(dump: &str, name: &str) -> String {
    let mut cfg = Config::parse(dump);
    cfg.apply_preset(name);
    cfg.dump()
}

pub fn pet_create(w: u32, h: u32, now: u64) {
    let mut c = core();
    c.size = (w, h);
    c.brain = Some(Brain::new(now));
}

pub fn pet_wake(now: u64) {
    if let Some(b) = core().brain.as_mut() {
        b.wake(now);
    }
}

pub fn pet_talk(now: u64, ms: u32) {
    if let Some(b) = core().brain.as_mut() {
        b.talk(now, ms);
    }
}

/// Dónde está su esquina y por dónde puede pasear, como fracciones del ancho de la pantalla.
pub fn pet_world(home: f32, lo: f32, hi: f32, now: u64) {
    if let Some(b) = core().brain.as_mut() {
        b.set_world(home, lo, hi, now);
    }
}

/// Hacia dónde está lo enfocado (fracción del ancho), o un valor negativo si no se sabe.
pub fn pet_focus(x: f32) {
    if let Some(b) = core().brain.as_mut() {
        b.set_focus(if x >= 0.0 { Some(x) } else { None });
    }
}

/// Empieza la rutina de aburrimiento durante `ms` milisegundos.
pub fn pet_routine(now: u64, ms: u64) {
    if let Some(b) = core().brain.as_mut() {
        b.begin_routine(now, ms);
    }
}

/// Alguien volvió a usar la TV: corta la rutina y manda a la mascota a su esquina.
pub fn pet_interrupt(now: u64) {
    if let Some(b) = core().brain.as_mut() {
        b.interrupt(now);
    }
}

/// Posición horizontal actual de la mascota (centro, fracción del ancho de la pantalla).
pub fn pet_x() -> f32 {
    core().brain.as_ref().map_or(0.5, |b| b.x())
}

/// ¿Está dentro de las horas de silencio? (La rutina de aburrimiento no arranca de noche.)
pub fn is_quiet(hour: i32) -> bool {
    core().policy().is_quiet(hour)
}

/// Avanza la mascota y, si hay algo nuevo, la dibuja en `buf`. Devuelve en los 16 bits bajos los ms hasta el próximo
/// fotograma (0 = no pedir más hasta el siguiente `pet_wake`/`pet_talk`/`pet_routine`), en los 4 siguientes lo que acaba
/// de empezar a hacer (`brain::EV_*`) y el bit [`DIRTY`] si `buf` cambió.
pub fn pet_tick(now: u64, buf: &mut [u8]) -> i32 {
    let mut c = core();
    let (w, h) = c.size;
    let look = look::of(c.cfg.get("pet.look"));
    let Some(brain) = c.brain.as_mut() else { return 0 };
    // Cada modelo con su forma de moverse: el Slayer no baila como las chicas.
    brain.set_moveset(if look.hair_style == look::HairStyle::Helmet { &slayer_moves::SLAYER } else { &pose::CHIBI });
    if buf.len() < (w * h * 4) as usize {
        return 0;
    }
    let (next, dirty) = brain.tick(now);
    if dirty {
        render::render(&mut buf[..(w * h * 4) as usize], w, h, &look, &brain.pose, &brain.particles, brain.time(now));
    }
    let event = brain.take_event() as i32 & 0xF;
    (next as i32).min(DELAY_MASK) | event << EVENT_SHIFT | if dirty { DIRTY } else { 0 }
}

pub fn decide(trigger: i32, now: u64, hour: i32, pkg: &str, text: &str, title: &str) -> Option<String> {
    let mut c = core();
    let policy = c.policy();
    c.jokes.decide(Trigger::from_i32(trigger), now, hour, pkg, text, title, &policy)
}

/// 0 = nada; si no, el número de [`Action`]. Solo hay que consumir la tecla cuando devuelve algo distinto de 0.
pub fn key_press(code: i32, now: u64) -> i32 {
    core().keys.press(code, now).map_or(0, |a| a as i32)
}

// ------------------------------------------------------------------------------------------------
// JNI: `com.hyprtv.Native`
// ------------------------------------------------------------------------------------------------

fn to_jstring(env: &mut JNIEnv, s: Option<String>) -> jstring {
    match s {
        Some(s) => env.new_string(s).map(|j| j.into_raw()).unwrap_or(std::ptr::null_mut()),
        None => std::ptr::null_mut(),
    }
}

fn read(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(|j| j.into()).unwrap_or_default()
}

macro_rules! guarded {
    ($default:expr, $body:expr) => {
        catch_unwind(AssertUnwindSafe(|| $body)).unwrap_or($default)
    };
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_loadConfig<'l>(mut env: JNIEnv<'l>, _c: JClass<'l>, path: JString<'l>) -> jstring {
    let path = read(&mut env, &path);
    let out = guarded!(None, Some(load_config(&path)));
    to_jstring(&mut env, out)
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_saveConfig<'l>(
    mut env: JNIEnv<'l>,
    _c: JClass<'l>,
    path: JString<'l>,
    dump: JString<'l>,
) -> jstring {
    let (path, dump) = (read(&mut env, &path), read(&mut env, &dump));
    let out = guarded!(None, Some(save_config(&path, &dump)));
    to_jstring(&mut env, out)
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_applyPreset<'l>(
    mut env: JNIEnv<'l>,
    _c: JClass<'l>,
    dump: JString<'l>,
    name: JString<'l>,
) -> jstring {
    let (dump, name) = (read(&mut env, &dump), read(&mut env, &name));
    let out = guarded!(None, Some(apply_preset(&dump, &name)));
    to_jstring(&mut env, out)
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_presetNames<'l>(mut env: JNIEnv<'l>, _c: JClass<'l>) -> jstring {
    to_jstring(&mut env, Some(config::preset_names().join(",")))
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petCreate<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, w: jint, h: jint, now: i64) {
    guarded!((), pet_create(w.max(16) as u32, h.max(16) as u32, now.max(0) as u64));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petWake<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, now: i64) {
    guarded!((), pet_wake(now.max(0) as u64));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petTalk<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, now: i64, ms: jint) {
    guarded!((), pet_talk(now.max(0) as u64, ms.max(0) as u32));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petWorld<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, home: f32, lo: f32, hi: f32, now: i64) {
    guarded!((), pet_world(home, lo, hi, now.max(0) as u64));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petFocus<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, x: f32) {
    guarded!((), pet_focus(x));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petRoutine<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, now: i64, ms: i64) {
    guarded!((), pet_routine(now.max(0) as u64, ms.max(0) as u64));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petInterrupt<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, now: i64) {
    guarded!((), pet_interrupt(now.max(0) as u64));
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petX<'l>(_env: JNIEnv<'l>, _c: JClass<'l>) -> f32 {
    guarded!(0.5, pet_x())
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_isQuiet<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, hour: jint) -> jni::sys::jboolean {
    guarded!(false, is_quiet(hour)) as jni::sys::jboolean
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_petTick<'l>(env: JNIEnv<'l>, _c: JClass<'l>, now: i64, buf: JByteBuffer<'l>) -> jint {
    let (Ok(ptr), Ok(len)) = (env.get_direct_buffer_address(&buf), env.get_direct_buffer_capacity(&buf)) else { return 0 };
    if ptr.is_null() {
        return 0;
    }
    // SAFETY: Kotlin reserva el ByteBuffer directo y no lo toca mientras dura esta llamada (mismo hilo).
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    guarded!(0, pet_tick(now.max(0) as u64, slice))
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_decide<'l>(
    mut env: JNIEnv<'l>,
    _c: JClass<'l>,
    trigger: jint,
    now: i64,
    hour: jint,
    pkg: JString<'l>,
    text: JString<'l>,
    title: JString<'l>,
) -> jstring {
    let (pkg, text, title) = (read(&mut env, &pkg), read(&mut env, &text), read(&mut env, &title));
    let out = guarded!(None, decide(trigger, now.max(0) as u64, hour, &pkg, &text, &title));
    to_jstring(&mut env, out)
}

#[no_mangle]
pub extern "system" fn Java_com_hyprtv_Native_keyPress<'l>(_env: JNIEnv<'l>, _c: JClass<'l>, code: jint, now: i64) -> jint {
    guarded!(0, key_press(code, now.max(0) as u64))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Los tests comparten el estado global del núcleo: se agrupan en uno solo para que no se pisen.
    #[test]
    fn end_to_end_through_the_global_core() {
        let dir = std::env::temp_dir().join(format!("hyprtv-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.ini");
        let path = path.to_str().unwrap();

        // Sin archivo: valores por defecto.
        let d = load_config(path);
        assert!(d.contains("pet.chatter=2"));

        // Guardar normaliza y persiste.
        let saved = save_config(path, "pet.chatter=3\ngap=999\ncombo.pet=5,5\npet.look=gafas");
        assert!(saved.contains("gap=40") && saved.contains("pet.chatter=3"));
        assert!(std::fs::read_to_string(path).unwrap().contains("combo.pet=5,5"));
        assert_eq!(load_config(path), saved);

        // Los atajos salen de la configuración activa.
        assert_eq!(key_press(5, 0), 0);
        assert_eq!(key_press(5, 100), Action::TogglePet as i32);

        // Chistes con la política activa.
        let said = decide(Trigger::Manual as i32, 1_000, 15, "com.hyprtv", "", "");
        assert!(said.is_some());

        // La mascota: nace dormida, se despierta, se dibuja y acaba pidiendo 0.
        let (w, h) = (96u32, 96u32);
        pet_create(w, h, 0);
        let mut buf = vec![0u8; (w * h * 4) as usize];
        pet_wake(10);
        let r = pet_tick(100, &mut buf);
        assert!(r & DIRTY != 0 && (r & DELAY_MASK) > 0);
        assert!(buf.iter().skip(3).step_by(4).any(|a| *a > 0), "debería haber dibujado algo");
        let mut now = 100;
        let mut last = r;
        for _ in 0..1000 {
            if last & DELAY_MASK == 0 {
                break;
            }
            now += (last & DELAY_MASK) as u64;
            last = pet_tick(now, &mut buf);
        }
        assert_eq!(last & DELAY_MASK, 0, "tiene que acabar durmiéndose");

        // Rutina de aburrimiento: arranca avisando, se mueve por la pantalla y vuelve a casa a dormir.
        pet_world(0.9, 0.08, 0.92, now);
        pet_focus(0.25);
        pet_routine(now, 30_000);
        let (mut events, mut min_x) = (Vec::new(), 1.0f32);
        for _ in 0..4000 {
            now += 80;
            let r = pet_tick(now, &mut buf);
            min_x = min_x.min(pet_x());
            let e = (r >> EVENT_SHIFT) & 0xF;
            if e != 0 {
                events.push(e);
            }
            if r & DELAY_MASK == 0 {
                break;
            }
        }
        assert_eq!(events.first().copied(), Some(brain::EV_BORED as i32), "{events:?}");
        assert!(events.contains(&(brain::EV_TIRED as i32)), "{events:?}");
        assert!(min_x < 0.7, "debería haber paseado: {min_x}");
        assert!((pet_x() - 0.9).abs() < 0.01);
        // Con alguien usando la TV no hay rutina que valga.
        pet_routine(now, 30_000);
        pet_interrupt(now + 100);
        assert!(!core().brain.as_ref().unwrap().in_routine());

        assert!(is_quiet(2) && !is_quiet(14));

        // Un búfer pequeño no revienta.
        let mut small = vec![0u8; 8];
        assert_eq!(pet_tick(now + 1, &mut small), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn preset_roundtrip() {
        let out = apply_preset("gap=12\n", "gruvbox");
        assert!(out.contains("color.bg=#282828") && out.contains("gap=12"));
        assert!(config::preset_names().contains(&"nord"));
    }
}
