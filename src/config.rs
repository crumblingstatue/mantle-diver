use {
    mdv_data::ron_pretty_cfg,
    serde::{Deserialize, Serialize},
    std::path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub scale: u8,
    pub last_world: Option<String>,
    #[serde(default = "default_res_folder_path")]
    pub res_folder_path: String,
    #[serde(default)]
    pub ui: UiConfig,
    #[serde(default)]
    pub music_vol: f32,
    #[serde(default)]
    pub sfx_vol: f32,
}

#[derive(Serialize, Deserialize)]
pub struct UiConfig {
    pub inv_frame_color: Rgba,
    pub inv_frame_highlight: Rgba,
    pub inv_bg_color: Rgba,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub fn to_sf(self) -> sfml::graphics::Color {
        sfml::graphics::Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        }
    }
    pub fn as_sf_mut(&mut self, f: impl FnOnce(&mut sfml::graphics::Color)) {
        let mut sf = self.to_sf();
        f(&mut sf);
        self.r = sf.r;
        self.g = sf.g;
        self.b = sf.b;
        self.a = sf.a;
    }

    pub(crate) fn coffset(&self, amount: i8) -> Self {
        Self {
            r: self.r.saturating_add_signed(amount),
            g: self.g.saturating_add_signed(amount),
            b: self.b.saturating_add_signed(amount),
            a: self.a,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            inv_frame_color: Rgba {
                r: 188,
                g: 139,
                b: 0,
                a: 255,
            },
            inv_frame_highlight: Rgba {
                r: 0,
                g: 255,
                b: 235,
                a: 255,
            },
            inv_bg_color: Rgba {
                r: 98,
                g: 82,
                b: 20,
                a: 128,
            },
        }
    }
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
            ui: UiConfig::default(),
            music_vol: 1.0,
            sfx_vol: 1.0,
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
