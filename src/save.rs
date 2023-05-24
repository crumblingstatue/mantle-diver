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
