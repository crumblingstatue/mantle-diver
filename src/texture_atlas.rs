use {
    crate::{
        math::IntRect,
        res::{path_key, walk_res_dir},
    },
    sfml::{graphics::Texture, SfBox},
    std::collections::HashMap,
    texture_packer::{texture::Texture as _, TexturePacker, TexturePackerConfig},
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
        let cfg = TexturePackerConfig {
            max_width: 512,
            max_height: 512,
            allow_rotation: false,
            border_padding: 0,
            texture_padding: 0,
            texture_extrusion: 0,
            trim: true,
            texture_outlines: false,
        };
        let mut packer = TexturePacker::new_skyline(cfg);
        walk_res_dir(&format!("{res_path}/graphics"), |path| {
            let img = image::open(path).unwrap();
            let key = path_key(path);
            packer.pack_own(key, img).unwrap();
        });
        let mut rects = HashMap::new();
        let mut tex = Texture::new().unwrap();
        log::info!(
            "Texture atlas size is: {}x{}",
            packer.width(),
            packer.height()
        );
        if !tex.create(packer.width(), packer.height()) {
            panic!("Failed to create texture");
        }
        let pixbuf = make_pix_buf(&packer);
        unsafe {
            tex.update_from_pixels(&pixbuf, packer.width(), packer.height(), 0, 0);
        }
        for (k, frame) in packer.get_frames() {
            rects.insert(
                k.clone(),
                IntRect {
                    x: frame.frame.x as i32,
                    y: frame.frame.y as i32,
                    w: frame.frame.w as i32,
                    h: frame.frame.h as i32,
                },
            );
        }
        Ok(AtlasBundle { tex, rects })
    }
}

fn make_pix_buf(packer: &TexturePacker<image::DynamicImage, String>) -> Vec<u8> {
    let (w, h) = (packer.width(), packer.height());
    let px_size = 4;
    let mut vec = vec![0; w as usize * h as usize * px_size as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = ((y * w + x) * px_size) as usize;
            if let Some(px) = packer.get(x, y) {
                vec[idx..idx + px_size as usize].copy_from_slice(&px.0);
            }
        }
    }
    vec
}
