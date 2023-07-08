use {
    crate::graphics::ScreenRes,
    mdv_math::types::ScreenVec,
    sfml::{
        graphics::{Sprite, Transformable},
        system::{Vector2f, Vector2u},
        window::VideoMode,
    },
};

pub trait SpriteExt {
    fn fit_to_size(&mut self, size: Vector2f);
}

impl SpriteExt for Sprite<'_> {
    fn fit_to_size(&mut self, size: Vector2f) {
        let Some(tex) = self.texture() else { return };
        let self_size: Vector2f = tex.size().as_other();
        self.set_scale((size.x / self_size.x, size.y / self_size.y));
    }
}

impl ScreenRes {
    pub fn to_sf(&self) -> VideoMode {
        VideoMode {
            width: self.w.into(),
            height: self.h.into(),
            bits_per_pixel: 32,
        }
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "This should be used for window and texture sizes, which shouldn't exceed u16::MAX"
    )]
    pub fn from_sf_vec(vec: Vector2u) -> Self {
        Self {
            w: vec.x as u16,
            h: vec.y as u16,
        }
    }
}

pub trait SfVec2fExt {
    fn scv_off(&self, off: ScreenVec) -> Self;
}

impl SfVec2fExt for Vector2f {
    fn scv_off(&self, off: ScreenVec) -> Self {
        Self {
            x: self.x + f32::from(off.x),
            y: self.y + f32::from(off.y),
        }
    }
}
