use {
    mdv_math::types::IntRect,
    sfml::{graphics::Texture, SfBox},
    stampcollect_sfml::SfmlAtlasBuilder,
    std::collections::HashMap,
};

pub type RectMap = HashMap<String, IntRect>;

#[derive(Debug)]
pub struct AtlasBundle {
    pub tex: SfBox<Texture>,
    // Key could be `tiles/dirt` for example, derived from folder+filename without extension
    pub rects: RectMap,
}

impl AtlasBundle {
    pub fn new(res_path: &str) -> anyhow::Result<Self> {
        let mut atlas_builder = SfmlAtlasBuilder::with_size(4096, 4096);
        let mut rects = HashMap::new();
        stampcollect_sfml::collect(
            &format!("{res_path}/graphics"),
            &mut atlas_builder,
            |name, rect| {
                rects.insert(
                    dbg!(name),
                    IntRect {
                        x: rect.x as i32,
                        y: rect.y as i32,
                        w: rect.w as i32,
                        h: rect.h as i32,
                    },
                );
            },
        );
        Ok(AtlasBundle {
            tex: atlas_builder.into_texture(),
            rects,
        })
    }
}
