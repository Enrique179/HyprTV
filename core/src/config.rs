//! Configuración y temas. Un `config.ini` de líneas `clave=valor` (editable en el PC y subible con `adb push`), con
//! validación: lo que no se entiende se descarta y un valor fuera de rango vuelve al de por defecto, así un tema roto
//! nunca deja la TV sin launcher.

use std::collections::BTreeMap;

pub enum Kind {
    Bool,
    Int(i32, i32),
    Enum(&'static [&'static str]),
    Color,
    Text,
    IntList,
}

/// Claves conocidas: (clave, tipo, valor por defecto).
pub const SPEC: &[(&str, Kind, &str)] = &[
    ("theme.enabled", Kind::Bool, "1"),
    ("theme.preset", Kind::Text, "hyprland"),
    ("gap", Kind::Int(0, 40), "10"),
    ("radius", Kind::Int(0, 40), "12"),
    ("border.width", Kind::Int(1, 8), "3"),
    ("border.style", Kind::Enum(&["gradient", "solid", "glow"]), "gradient"),
    ("border.anim", Kind::Bool, "0"),
    ("anim.speed", Kind::Int(0, 300), "100"),
    ("font", Kind::Enum(&["mono", "sans", "serif"]), "mono"),
    ("bg.mode", Kind::Enum(&["gradient", "solid", "image"]), "gradient"),
    ("layout", Kind::Enum(&["grid", "master"]), "grid"),
    ("columns", Kind::Int(3, 8), "4"),
    ("tile.ratio", Kind::Int(40, 100), "56"),
    ("tile.labels", Kind::Bool, "1"),
    ("bar.enabled", Kind::Bool, "1"),
    ("clock.24h", Kind::Bool, "1"),
    ("ws.count", Kind::Int(1, 6), "3"),
    ("ws.1.name", Kind::Text, "apps"),
    ("ws.2.name", Kind::Text, "media"),
    ("ws.3.name", Kind::Text, "sistema"),
    ("ws.4.name", Kind::Text, "4"),
    ("ws.5.name", Kind::Text, "5"),
    ("ws.6.name", Kind::Text, "6"),
    ("ring.enabled", Kind::Bool, "1"),
    ("ring.padding", Kind::Int(0, 20), "4"),
    // `lista`: solo en las apps de `ring.allow` (las que no pintan su propio resaltado); `todas`: en todas menos `ring.deny`.
    ("ring.mode", Kind::Enum(&["lista", "todas", "ninguna"]), "lista"),
    ("ring.allow", Kind::Text, "com.android.tv.settings,com.android.settings,com.android.vending,com.google.android.packageinstaller,com.google.android.permissioncontroller,com.android.packageinstaller"),
    ("ring.deny", Kind::Text, ""),
    ("pet.enabled", Kind::Bool, "1"),
    // Rutina cuando la TV no se usa: pasea, baila, señala lo enfocado...
    ("pet.idle", Kind::Bool, "1"),
    ("pet.idle.minutes", Kind::Int(1, 30), "2"),
    ("pet.look", Kind::Enum(&["bufanda", "gorro", "coletas", "gafas", "slayer"]), "slayer"),
    ("pet.size", Kind::Int(60, 200), "100"),
    ("pet.corner", Kind::Enum(&["br", "bl"]), "br"),
    ("pet.chatter", Kind::Int(0, 3), "2"),
    ("pet.quiet", Kind::Bool, "1"),
    ("pet.quiet.start", Kind::Int(0, 23), "23"),
    ("pet.quiet.end", Kind::Int(0, 23), "7"),
    ("pet.muted", Kind::Text, ""),
    ("combo.pet", Kind::IntList, "82,82,82"),
    ("combo.theme", Kind::IntList, "165,165,165"),
    ("combo.ring", Kind::IntList, ""),
    ("combo.corner", Kind::IntList, ""),
    ("combo.ringapp", Kind::IntList, ""),
    ("color.bg", Kind::Color, "#07060C"),
    ("color.bg2", Kind::Color, "#1A1033"),
    ("color.panel", Kind::Color, "#0D0B16"),
    ("color.border", Kind::Color, "#2B2447"),
    ("color.text", Kind::Color, "#E9E4FF"),
    ("color.muted", Kind::Color, "#8A82B0"),
    ("color.a1", Kind::Color, "#D6BCFF"),
    ("color.a2", Kind::Color, "#8B5CF6"),
    ("color.a3", Kind::Color, "#E879F9"),
    ("color.danger", Kind::Color, "#FF6B8A"),
    ("bg.image", Kind::Text, "bg.png"),
];

const COLOR_KEYS: [&str; 10] = [
    "color.bg", "color.bg2", "color.panel", "color.border", "color.text", "color.muted", "color.a1", "color.a2", "color.a3",
    "color.danger",
];

/// (nombre, colores en el orden de `COLOR_KEYS`)
pub const PRESETS: &[(&str, [&str; 10])] = &[
    ("hyprland", ["#07060C", "#1A1033", "#0D0B16", "#2B2447", "#E9E4FF", "#8A82B0", "#D6BCFF", "#8B5CF6", "#E879F9", "#FF6B8A"]),
    ("catppuccin", ["#1E1E2E", "#313244", "#181825", "#45475A", "#CDD6F4", "#9399B2", "#89B4FA", "#CBA6F7", "#F5C2E7", "#F38BA8"]),
    ("tokyonight", ["#1A1B26", "#24283B", "#16161E", "#3B4261", "#C0CAF5", "#737AA2", "#7AA2F7", "#BB9AF7", "#7DCFFF", "#F7768E"]),
    ("gruvbox", ["#282828", "#3C3836", "#1D2021", "#504945", "#EBDBB2", "#A89984", "#FABD2F", "#FE8019", "#B8BB26", "#FB4934"]),
    ("nord", ["#2E3440", "#3B4252", "#272C36", "#4C566A", "#ECEFF4", "#81A1C1", "#88C0D0", "#81A1C1", "#B48EAD", "#BF616A"]),
    ("rosepine", ["#191724", "#26233A", "#1F1D2E", "#403D52", "#E0DEF4", "#908CAA", "#EBBCBA", "#C4A7E7", "#9CCFD8", "#EB6F92"]),
    ("dracula", ["#282A36", "#44475A", "#21222C", "#44475A", "#F8F8F2", "#6272A4", "#BD93F9", "#FF79C6", "#8BE9FD", "#FF5555"]),
];

pub fn preset_names() -> Vec<&'static str> {
    PRESETS.iter().map(|p| p.0).collect()
}

fn spec(key: &str) -> Option<&'static (&'static str, Kind, &'static str)> {
    SPEC.iter().find(|s| s.0 == key)
}

fn normalize_color(v: &str) -> Option<String> {
    let h = v.trim().strip_prefix('#')?;
    if (h.len() == 6 || h.len() == 8) && h.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(format!("#{}", h.to_ascii_uppercase()))
    } else {
        None
    }
}

/// Valida `value` para `key`; `None` si no se acepta. También admite las claves dinámicas `app.<paquete>` (espacio 1..6).
pub fn normalize(key: &str, value: &str) -> Option<String> {
    let v = value.trim();
    if let Some(pkg) = key.strip_prefix("app.") {
        let ok_pkg = !pkg.is_empty() && pkg.len() <= 120 && pkg.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_');
        return match v.parse::<i32>() {
            Ok(n) if ok_pkg && (1..=6).contains(&n) => Some(n.to_string()),
            _ => None,
        };
    }
    let (_, kind, _) = spec(key)?;
    match kind {
        Kind::Bool => match v {
            "1" | "true" | "on" | "si" | "sí" => Some("1".into()),
            "0" | "false" | "off" | "no" => Some("0".into()),
            _ => None,
        },
        Kind::Int(lo, hi) => v.parse::<i32>().ok().map(|n| n.clamp(*lo, *hi).to_string()),
        Kind::Enum(options) => options.iter().find(|o| o.eq_ignore_ascii_case(v)).map(|o| o.to_string()),
        Kind::Color => normalize_color(v),
        Kind::Text => {
            // Una línea, sin saltos que rompan el formato, y acotado (las listas de paquetes caben en más).
            let cap = if key.ends_with(".allow") || key.ends_with(".deny") || key == "pet.muted" { 2000 } else { 200 };
            if v.len() <= cap && !v.contains('\n') && !v.contains('\r') {
                Some(v.to_string())
            } else {
                None
            }
        }
        Kind::IntList => {
            if v.is_empty() {
                return Some(String::new());
            }
            let nums: Option<Vec<i32>> = v.split(',').map(|p| p.trim().parse::<i32>().ok().filter(|n| (1..=400).contains(n))).collect();
            nums.filter(|n| n.len() <= 8).map(|n| n.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    map: BTreeMap<String, String>,
}

impl Config {
    pub fn defaults() -> Config {
        Config { map: SPEC.iter().map(|(k, _, d)| (k.to_string(), d.to_string())).collect() }
    }

    /// Lee un `config.ini`. Parte de los valores por defecto; si trae un `theme.preset` conocido se aplica primero y los
    /// colores sueltos del archivo lo retocan.
    pub fn parse(text: &str) -> Config {
        let pairs: Vec<(&str, &str)> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with(';'))
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.trim(), v.trim()))
            .collect();
        let mut c = Config::defaults();
        if let Some((_, p)) = pairs.iter().find(|(k, _)| *k == "theme.preset") {
            c.apply_preset(p);
        }
        for (k, v) in pairs {
            if k != "theme.preset" {
                c.set(k, v);
            }
        }
        c
    }

    pub fn dump(&self) -> String {
        let mut out = String::new();
        for (k, v) in &self.map {
            out.push_str(k);
            out.push('=');
            out.push_str(v);
            out.push('\n');
        }
        out
    }

    pub fn get(&self, key: &str) -> &str {
        self.map.get(key).map(String::as_str).unwrap_or("")
    }

    pub fn get_i32(&self, key: &str) -> i32 {
        self.get(key).parse().unwrap_or(0)
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key) == "1"
    }

    pub fn ints(&self, key: &str) -> Vec<i32> {
        self.get(key).split(',').filter_map(|p| p.trim().parse().ok()).collect()
    }

    /// Cambia un valor si es válido. Devuelve si se aceptó.
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        match normalize(key, value) {
            Some(v) => {
                if key.starts_with("color.")
                    && self.map.get(key) != Some(&v)
                    && self.map.get("theme.preset").map(String::as_str) != Some("custom")
                {
                    // Retocar un color suelto deja de ser el preset puro (releer los mismos valores no cuenta).
                    self.map.insert("theme.preset".into(), "custom".into());
                }
                if key == "theme.preset" {
                    return self.apply_preset(&v);
                }
                self.map.insert(key.to_string(), v);
                true
            }
            None => false,
        }
    }

    pub fn apply_preset(&mut self, name: &str) -> bool {
        match PRESETS.iter().find(|p| p.0.eq_ignore_ascii_case(name)) {
            Some((n, colors)) => {
                for (k, c) in COLOR_KEYS.iter().zip(colors.iter()) {
                    self.map.insert(k.to_string(), c.to_string());
                }
                self.map.insert("theme.preset".into(), n.to_string());
                true
            }
            None => {
                if name == "custom" {
                    self.map.insert("theme.preset".into(), "custom".into());
                    true
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_all_valid() {
        for (k, _, d) in SPEC {
            assert_eq!(normalize(k, d).as_deref().unwrap_or("<inválido>"), *d, "valor por defecto de {k}");
        }
    }

    #[test]
    fn roundtrip_and_garbage_is_dropped() {
        let c = Config::parse("gap=20\nclave.rara=1\nradius=999\nborder.style=NEON\ncolor.a1=#abcdef\n# comentario\nbasura");
        assert_eq!(c.get_i32("gap"), 20);
        assert_eq!(c.get_i32("radius"), 40, "se recorta al máximo");
        assert_eq!(c.get("border.style"), "gradient", "valor inválido: por defecto");
        assert_eq!(c.get("color.a1"), "#ABCDEF");
        assert_eq!(c.get("clave.rara"), "");
        let again = Config::parse(&c.dump());
        assert_eq!(again.dump(), c.dump());
    }

    #[test]
    fn preset_then_overrides() {
        let c = Config::parse("theme.preset=nord\ncolor.a2=#FF0000");
        assert_eq!(c.get("color.bg"), "#2E3440");
        assert_eq!(c.get("color.a2"), "#FF0000");
        assert_eq!(c.get("theme.preset"), "custom", "tocar un color lo hace personalizado");
        let mut d = Config::defaults();
        assert!(d.set("theme.preset", "dracula"));
        assert_eq!(d.get("color.a1"), "#BD93F9");
        assert!(!d.set("theme.preset", "no-existe"));
    }

    #[test]
    fn app_assignments_and_lists() {
        let mut c = Config::defaults();
        assert!(c.set("app.com.google.android.youtube.tv", "2"));
        assert!(!c.set("app.com.x", "9"), "espacio fuera de 1..6");
        assert!(!c.set("app.bad pkg", "1"));
        assert!(c.set("combo.pet", "82, 82 ,82"));
        assert_eq!(c.ints("combo.pet"), vec![82, 82, 82]);
        assert!(!c.set("combo.pet", "a,b"));
        assert!(c.set("combo.ring", ""));
        assert!(c.ints("combo.ring").is_empty());
    }

    #[test]
    fn ring_lists_and_idle_settings() {
        let c = Config::defaults();
        assert_eq!(c.get("ring.mode"), "lista");
        assert!(c.get("ring.allow").split(',').any(|p| p == "com.android.tv.settings"));
        assert!(!c.get("ring.allow").split(',').any(|p| p.contains("youtube") || p.contains("mgstv")), "las apps con su propio foco quedan fuera");
        assert!(c.get_bool("pet.idle") && c.get_i32("pet.idle.minutes") == 2);
        let mut c = Config::defaults();
        assert!(c.set("pet.idle.minutes", "99"));
        assert_eq!(c.get_i32("pet.idle.minutes"), 30);
        let long = (0..60).map(|i| format!("com.ejemplo.app{i}")).collect::<Vec<_>>().join(",");
        assert!(c.set("ring.deny", &long), "las listas largas de paquetes caben");
        assert!(!c.set("pet.muted", &"x".repeat(2100)));
        assert!(!c.set("ring.mode", "siempre"));
    }

    #[test]
    fn bool_aliases() {
        let mut c = Config::defaults();
        assert!(c.set("pet.enabled", "off"));
        assert!(!c.get_bool("pet.enabled"));
        assert!(c.set("pet.enabled", "si"));
        assert!(c.get_bool("pet.enabled"));
    }
}
