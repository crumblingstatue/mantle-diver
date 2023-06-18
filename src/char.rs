use {
    crate::{config::ron_pretty_cfg, math::ScreenVec},
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

/// Character (graphics) database
#[derive(Serialize, Deserialize, Default)]
pub struct CharDb {
    #[serde(serialize_with = "crate::config::ordered_map")]
    pub graphic_offsets: HashMap<String, Offset>,
}

impl CharDb {
    pub fn save(&self) -> anyhow::Result<()> {
        let s = ron::ser::to_string_pretty(self, ron_pretty_cfg())?;
        std::fs::write("data/char.ron", s.as_bytes())?;
        Ok(())
    }
    pub fn load() -> anyhow::Result<Self> {
        match std::fs::read_to_string("data/char.ron") {
            Ok(data) => Ok(ron::from_str(&data)?),
            Err(e) => {
                log::error!("Failed to load char db: {e}. Creating default");
                Ok(CharDb::default())
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Offset {
    /// Offset when facing left
    pub left: ScreenVec,
    /// Offset when facing right
    pub right: ScreenVec,
}
