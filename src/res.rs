use {
    crate::texture_atlas::AtlasBundle,
    egui_sfml::egui::epaint::ahash::HashMap,
    sfml::{
        graphics::{Font, Texture},
        SfBox,
    },
    std::{io::Cursor, path::Path},
};

pub type AuBuf = Cursor<Vec<u8>>;

pub struct Res {
    pub atlas: AtlasBundle,
    pub surf_music: AuBuf,
    pub und_music: AuBuf,
    pub sans_font: SfBox<Font>,
    pub forest_bg: SfBox<Texture>,
}

pub struct ResAudio {
    pub sounds: HashMap<String, Cursor<Vec<u8>>>,
}

pub fn load_sound<P: AsRef<Path>>(path: P) -> anyhow::Result<AuBuf> {
    let data = std::fs::read(path)?;
    Ok(std::io::Cursor::new(data))
}

impl Res {
    pub fn load(res_path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            atlas: AtlasBundle::new(res_path)?,
            surf_music: load_sound(format!("{res_path}/music/calm.ogg"))?,
            und_music: load_sound(format!("{res_path}/music/underground.mp3"))?,
            sans_font: Font::from_file(&format!("{res_path}/fonts/ShareTechMono-Regular.ttf"))
                .unwrap(),
            forest_bg: Texture::from_file(&format!("{res_path}/bg/sky.png")).unwrap(),
        })
    }
}

impl ResAudio {
    pub fn load(res_path: &str) -> anyhow::Result<Self> {
        let mut map = HashMap::default();
        walk_res_dir(&format!("{res_path}/sfx"), |path| {
            let snd = load_sound(path.to_str().unwrap()).unwrap();
            map.insert(path_key(path), snd);
        });
        Ok(Self { sounds: map })
    }
}

pub fn path_key(path: &Path) -> String {
    let mut rev_iter = path.components().rev();
    let fname = rev_iter.next().unwrap();
    let folder = rev_iter.next().unwrap();
    let fname: &Path = fname.as_ref();
    let folder: &Path = folder.as_ref();
    let key_string = folder
        .join(fname.file_stem().unwrap())
        .display()
        .to_string();
    key_string.replace('\\', "/")
}

#[test]
fn test_path_key() {
    assert_eq!(
        &path_key("/home/person/res/graphics/tiles/foo.png".as_ref()),
        "tiles/foo"
    );
}

pub fn walk_res_dir(base: &str, mut f: impl FnMut(&Path)) {
    for en in walkdir::WalkDir::new(base) {
        let en = en.unwrap();
        if en.file_type().is_file() {
            f(en.path());
        }
    }
}
