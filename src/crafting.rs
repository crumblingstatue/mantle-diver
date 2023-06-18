use {
    mdv_data::{item::ItemStack, ron_pretty_cfg},
    serde::{Deserialize, Serialize},
};

/// Crafting station
#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum Station {
    Player,
}

/// Crafting recipe
#[derive(Serialize, Deserialize)]
pub struct Recipe {
    /// Stations that this recipe can be crafted at
    pub(crate) stations: Vec<Station>,
    /// What items this recipe requires
    pub(crate) input: Vec<ItemStack>,
    /// Result of crafting the recipe
    pub(crate) output: ItemStack,
}

#[derive(Serialize, Deserialize, Default)]
pub struct RecipeDb {
    pub recipes: Vec<Recipe>,
}

const PATH: &str = "recipes.ron";

impl RecipeDb {
    pub fn load_or_default(data_path: &str) -> Self {
        match std::fs::read_to_string(format!("{data_path}/{PATH}")) {
            Ok(data) => match ron::from_str(&data) {
                Ok(db) => db,
                Err(e) => {
                    log::warn!("Failed to load recipe database: {e}\nCreating default.");
                    Default::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to load recipe database: {e}\nCreating default.");
                Default::default()
            }
        }
    }
    pub fn try_save(&self, data_path: &str) {
        match ron::ser::to_string_pretty(self, ron_pretty_cfg()) {
            Ok(str) => match std::fs::write(format!("{data_path}/{PATH}"), str) {
                Ok(()) => {}
                Err(e) => log::warn!("Failed to save recipe db: {e}"),
            },
            Err(e) => log::warn!("Failed to save recipe db: {e}"),
        }
    }
}
