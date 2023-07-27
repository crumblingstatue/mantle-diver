use {
    crate::{graphics::ScreenRes, math::IntRectExt},
    extension_traits::extension,
    mdv_math::types::IntRect,
};

#[extension(pub trait EguiUiExt)]
impl egui::Ui {
    fn graphic_image(&mut self, rect: &IntRect, atlas_size: ScreenRes) {
        self.add(
            egui::Image::new(egui::TextureId::User(0), rect.to_egui_size())
                .uv(rect.to_egui_uv(atlas_size)),
        );
    }
}
