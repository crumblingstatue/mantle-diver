use {
    crate::texture_atlas::AtlasBundle,
    egui::epaint::ahash::HashMap,
    sfml::{
        audio::{Music, SoundBuffer},
        graphics::{Font, Texture},
        SfBox,
    },
    std::path::Path,
};

#[derive(Debug)]
pub struct Res {
    pub atlas: AtlasBundle,
    pub surf_music: Music<'static>,
    pub und_music: Music<'static>,
    pub sans_font: SfBox<Font>,
    pub forest_bg: SfBox<Texture>,
}

pub struct ResAudio {
    pub sounds: HashMap<String, SfBox<SoundBuffer>>,
}

impl Res {
    pub fn load(res_path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            atlas: AtlasBundle::new(res_path)?,
            surf_music: Music::from_file(&format!("{res_path}/music/music.ogg")).unwrap(),
            und_music: Music::from_file(&format!("{res_path}/music/cave2.ogg")).unwrap(),
            sans_font: Font::from_file(&format!("{res_path}/fonts/sans.ttf")).unwrap(),
            forest_bg: Texture::from_file(&format!("{res_path}/bg/forest.png")).unwrap(),
        })
    }
}

impl ResAudio {
    pub fn load(res_path: &str) -> anyhow::Result<Self> {
        let mut map = HashMap::default();
        walk_res_dir(&format!("{res_path}/sfx"), |path| {
            let snd = SoundBuffer::from_file(path.to_str().unwrap()).unwrap();
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
    folder
        .join(fname.file_stem().unwrap())
        .display()
        .to_string()
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
