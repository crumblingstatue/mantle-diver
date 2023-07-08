use sfml::{
    graphics::{Sprite, Transformable},
    system::Vector2f,
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
