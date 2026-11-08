//! Los 4 modelos de la mascota (port de `PetLook.kt`): mismo esqueleto, otro pelo, otra ropa y otros colores.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HairStyle {
    Bob,
    Short,
    Pigtails,
    Long,
    /// Casco cerrado: no hay pelo ni cara, solo cúpula y visor.
    Helmet,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TopStyle {
    Jacket,
    Hoodie,
    Dress,
    BandTee,
    /// Peto con hombreras.
    Armor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Extra {
    Scarf,
    Beanie,
    Bows,
    Glasses,
    /// El visor va dentro del casco; no añade nada encima.
    Visor,
}

/// Colores en 0xAARRGGBB.
#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub id: &'static str,
    pub skin: u32,
    pub hair: u32,
    pub hair_light: u32,
    pub top: u32,
    pub top_dark: u32,
    pub accent: u32,
    pub accent_dark: u32,
    pub pants: u32,
    pub boot: u32,
    pub iris: u32,
    pub iris_light: u32,
    pub hair_style: HairStyle,
    pub top_style: TopStyle,
    pub extra: Extra,
}

impl Look {
    /// Con vestido o camiseta se ve el antebrazo; con chaqueta o sudadera la manga llega a la mano.
    pub fn bare_forearm(&self) -> bool {
        matches!(self.top_style, TopStyle::Dress | TopStyle::BandTee)
    }
}

pub const SCARF: Look = Look {
    id: "bufanda",
    skin: 0xFFFFE0CE,
    hair: 0xFF211D2B,
    hair_light: 0xFF4F4866,
    top: 0xFFA67C58,
    top_dark: 0xFF6D4C3A,
    accent: 0xFFD32F2F,
    accent_dark: 0xFF9E1B1B,
    pants: 0xFFF0E8D8,
    boot: 0xFF4A332A,
    iris: 0xFF2F2E3E,
    iris_light: 0xFF7A7896,
    hair_style: HairStyle::Bob,
    top_style: TopStyle::Jacket,
    extra: Extra::Scarf,
};

pub const BEANIE: Look = Look {
    id: "gorro",
    skin: 0xFFF2C9A5,
    hair: 0xFF5B3A29,
    hair_light: 0xFF8A5E43,
    top: 0xFFE0A526,
    top_dark: 0xFFA8761A,
    accent: 0xFF2BA6A0,
    accent_dark: 0xFF1B7773,
    pants: 0xFF3B5B8C,
    boot: 0xFFEDEDED,
    iris: 0xFF3B2A20,
    iris_light: 0xFF8C6B55,
    hair_style: HairStyle::Short,
    top_style: TopStyle::Hoodie,
    extra: Extra::Beanie,
};

pub const PIGTAILS: Look = Look {
    id: "coletas",
    skin: 0xFFFFE3D3,
    hair: 0xFFF2C94C,
    hair_light: 0xFFFFE48A,
    top: 0xFFF48FB1,
    top_dark: 0xFFCF5E8A,
    accent: 0xFFFF5C8D,
    accent_dark: 0xFFB0305F,
    pants: 0xFFFFE3D3,
    boot: 0xFF7E57C2,
    iris: 0xFF2E6FB5,
    iris_light: 0xFF8EC5F2,
    hair_style: HairStyle::Pigtails,
    top_style: TopStyle::Dress,
    extra: Extra::Bows,
};

pub const ROCKER: Look = Look {
    id: "gafas",
    skin: 0xFFE8B896,
    hair: 0xFF4A1E3F,
    hair_light: 0xFF8A4A7C,
    top: 0xFF26242E,
    top_dark: 0xFF111018,
    accent: 0xFFA78BFA,
    accent_dark: 0xFF6D4FD6,
    pants: 0xFF54506B,
    boot: 0xFF1C1B22,
    iris: 0xFF2F2E3E,
    iris_light: 0xFF7A7896,
    hair_style: HairStyle::Long,
    top_style: TopStyle::BandTee,
    extra: Extra::Glasses,
};

pub const SLAYER: Look = Look {
    id: "slayer",
    // Con casco no hay piel: `skin` es el brazo desnudo bajo las hombreras.
    skin: 0xFF8F8A6A,
    // Casco y hombrera izquierda
    hair: 0xFF5E7440,
    hair_light: 0xFF8FA766,
    // Peto
    top: 0xFF55663A,
    // Gris oscuro: mentonera, orejeras, rodilleras, guantelete izquierdo
    top_dark: 0xFF3A3D38,
    // Naranja de la franja del peto
    accent: 0xFFC26A1C,
    // Marrón de la hombrera y el guantelete derechos
    accent_dark: 0xFF7A5236,
    pants: 0xFF4F6036,
    boot: 0xFF4A4D44,
    // Cristal del visor
    iris: 0xFF1F7396,
    iris_light: 0xFF8AD8EE,
    hair_style: HairStyle::Helmet,
    top_style: TopStyle::Armor,
    extra: Extra::Visor,
};

pub const ALL: [Look; 5] = [SCARF, BEANIE, PIGTAILS, ROCKER, SLAYER];

/// Un id que no existe cae en el primero (la bufanda), como en el reproductor.
pub fn of(id: &str) -> Look {
    ALL.iter().copied().find(|l| l.id == id).unwrap_or(SCARF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_id_falls_back_to_scarf() {
        assert_eq!(of("nada").id, "bufanda");
        assert_eq!(of("gafas").id, "gafas");
        assert!(of("coletas").hair_style == HairStyle::Pigtails);
        assert!(of("slayer").hair_style == HairStyle::Helmet);
    }
}
