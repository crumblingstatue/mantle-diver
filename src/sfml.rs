use {
    crate::graphics::ScreenRes,
    extension_traits::extension,
    mdv_math::types::{ScreenRect, ScreenVec},
    sfml::{
        graphics::{RectangleShape, RenderTarget, Sprite, Transformable},
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
    pub fn to_sf(self) -> VideoMode {
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

#[extension(pub trait RenderTargetExt)]
impl<T: RenderTarget> T {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "Size is smaller than i16::MAX"
    )]
    fn res(&self) -> ScreenRes {
        let size = self.size();
        ScreenRes {
            w: size.x as u16,
            h: size.y as u16,
        }
    }
}

#[extension(pub trait ScreenRectSfExt)]
impl ScreenRect {
    fn into_sf(&self) -> sfml::graphics::FloatRect {
        sfml::graphics::FloatRect {
            left: f32::from(self.x),
            top: f32::from(self.y),
            width: f32::from(self.w),
            height: f32::from(self.h),
        }
    }
    fn sf_position(&self) -> Vector2f {
        Vector2f::new(f32::from(self.x), f32::from(self.y))
    }
}

#[extension(pub trait ScreenVecSfExt)]
impl ScreenVec {
    fn to_sf_vec2f(&self) -> Vector2f {
        Vector2f {
            x: f32::from(self.x),
            y: f32::from(self.y),
        }
    }
}

#[extension(pub trait RectangleShapeExt)]
impl RectangleShape<'_> {
    fn set_screen_rect(&mut self, rect: ScreenRect) {
        self.set_position((f32::from(rect.x), f32::from(rect.y)));
        self.set_size((f32::from(rect.w), f32::from(rect.h)));
    }
}
