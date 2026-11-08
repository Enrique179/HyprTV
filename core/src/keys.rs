//! Atajos del mando: secuencias de teclas pulsadas seguidas (cada una a menos de `WINDOW_MS` de la anterior).
//! Solo se "consume" la última tecla de la secuencia; las anteriores pasan al sistema como siempre.

pub const WINDOW_MS: u64 = 1_200;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    TogglePet = 1,
    ToggleTheme = 2,
    ToggleRing = 3,
    PetCorner = 4,
    /// Enciende o apaga el borde de foco solo para la app que está delante.
    RingThisApp = 5,
}

#[derive(Default)]
pub struct Keys {
    combos: Vec<(Action, Vec<i32>)>,
    history: Vec<(i32, u64)>,
}

impl Keys {
    pub fn set_combos(&mut self, combos: Vec<(Action, Vec<i32>)>) {
        self.combos = combos.into_iter().filter(|(_, c)| !c.is_empty()).collect();
        self.history.clear();
    }

    /// Registra una pulsación (solo "abajo", sin repeticiones) y dice si completa algún atajo.
    pub fn press(&mut self, code: i32, now: u64) -> Option<Action> {
        if self.combos.is_empty() {
            return None;
        }
        // Una pausa larga empieza una secuencia nueva.
        if let Some(&(_, last)) = self.history.last() {
            if now.saturating_sub(last) > WINDOW_MS {
                self.history.clear();
            }
        }
        self.history.push((code, now));
        let longest = self.combos.iter().map(|(_, c)| c.len()).max().unwrap_or(0);
        if self.history.len() > longest {
            let extra = self.history.len() - longest;
            self.history.drain(0..extra);
        }
        // Gana el atajo más largo si dos terminan a la vez (p. ej. uno es sufijo del otro).
        let mut best: Option<(Action, usize)> = None;
        for (action, combo) in &self.combos {
            let n = combo.len();
            if self.history.len() < n {
                continue;
            }
            let tail = &self.history[self.history.len() - n..];
            let same = tail.iter().zip(combo).all(|((k, _), c)| k == c);
            let in_time = tail.windows(2).all(|w| w[1].1.saturating_sub(w[0].1) <= WINDOW_MS);
            if same && in_time && best.is_none_or(|(_, len)| n > len) {
                best = Some((*action, n));
            }
        }
        if best.is_some() {
            self.history.clear();
        }
        best.map(|(a, _)| a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Keys {
        let mut k = Keys::default();
        k.set_combos(vec![(Action::TogglePet, vec![82, 82, 82]), (Action::ToggleTheme, vec![165, 165, 165, 165, 165])]);
        k
    }

    #[test]
    fn triple_press_triggers_only_on_the_last() {
        let mut k = keys();
        assert_eq!(k.press(82, 0), None);
        assert_eq!(k.press(82, 300), None);
        assert_eq!(k.press(82, 600), Some(Action::TogglePet));
        // Después empieza de cero.
        assert_eq!(k.press(82, 700), None);
    }

    #[test]
    fn slow_presses_do_not_count() {
        let mut k = keys();
        k.press(82, 0);
        k.press(82, 1_000);
        assert_eq!(k.press(82, 2_500), None, "la tercera llega fuera de la ventana");
    }

    #[test]
    fn other_keys_break_the_sequence() {
        let mut k = keys();
        k.press(82, 0);
        k.press(19, 100);
        assert_eq!(k.press(82, 200), None);
        assert_eq!(k.press(82, 300), None);
    }

    #[test]
    fn long_combo_and_empty_combos() {
        let mut k = keys();
        let mut r = None;
        for i in 0..5 {
            r = k.press(165, i * 200);
        }
        assert_eq!(r, Some(Action::ToggleTheme));
        let mut none = Keys::default();
        assert_eq!(none.press(82, 0), None);
    }

    #[test]
    fn longer_wins_when_one_is_a_suffix() {
        let mut k = Keys::default();
        k.set_combos(vec![(Action::TogglePet, vec![7, 7]), (Action::ToggleTheme, vec![8, 7, 7])]);
        k.press(8, 0);
        k.press(7, 100);
        assert_eq!(k.press(7, 200), Some(Action::ToggleTheme));
    }
}
