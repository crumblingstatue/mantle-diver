use {
    ron::{extensions::Extensions, ser::PrettyConfig},
    serde::{Deserialize, Serialize, Serializer},
    std::{
        collections::{BTreeMap, HashMap},
        path::Path,
    },
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub scale: u8,
    pub last_world: Option<String>,
    #[serde(default = "default_res_folder_path")]
    pub res_folder_path: String,
}

pub fn default_res_folder_path() -> String {
    "assets".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale: 1,
            last_world: Default::default(),
            res_folder_path: default_res_folder_path(),
        }
    }
}

impl Config {
    pub fn load(base_dir: &Path) -> anyhow::Result<Self> {
        let path = base_dir.join("config.ron");
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        Ok(ron::from_str(&data)?)
    }
    pub fn save(&self, base_dir: &Path) -> anyhow::Result<()> {
        let path = base_dir.join("config.ron");
        let data = ron::ser::to_string_pretty(self, ron_pretty_cfg())?;
        Ok(std::fs::write(path, data)?)
    }
}

pub fn ron_pretty_cfg() -> PrettyConfig {
    PrettyConfig::default()
        .enumerate_arrays(true)
        .struct_names(true)
        .extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES)
}

/// Based on https://stackoverflow.com/a/42723390
pub fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    hm: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = hm.iter().collect();
    ordered.serialize(serializer)
}
