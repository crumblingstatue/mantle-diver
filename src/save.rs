use {
    crate::inventory::Inventory,
    serde::{Deserialize, Serialize},
    std::path::Path,
};

/// Holds the chunk-independent save data like inventory, etc.
#[derive(Serialize, Deserialize)]
pub struct Save {
    pub inventory: Inventory,
    pub world_seed: i32,
    pub player: PlayerSav,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSav {
    pub skin_color: Rgb,
    pub eye_color: Rgb,
    pub hair_color: Rgb,
    pub shirt_color: Rgb,
    pub pants_color: Rgb,
    pub shoes_color: Rgb,
}

#[derive(Serialize, Deserialize)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub fn to_sf(&self) -> sfml::graphics::Color {
        sfml::graphics::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 255,
        }
    }

    pub(crate) fn from_sf(sf: sfml::graphics::Color) -> Rgb {
        Self {
            r: sf.r,
            g: sf.g,
            b: sf.b,
        }
    }
}

impl Save {
    pub fn save(&self, world_dir: &Path) -> anyhow::Result<()> {
        Ok(std::fs::write(
            world_dir.join("save.dat"),
            rmp_serde::to_vec(self)?,
        )?)
    }
    pub fn load(world_dir: &Path) -> anyhow::Result<Self> {
        Ok(rmp_serde::from_slice(&std::fs::read(
            world_dir.join("save.dat"),
        )?)?)
    }
}
