use {
    crate::{graphics::ScreenRes, math::IntRectExt},
    egui_sfml::egui,
    extension_traits::extension,
    mdv_math::types::IntRect,
};

#[extension(pub trait EguiUiExt)]
impl egui::Ui {
    fn graphic_image(&mut self, rect: &IntRect, atlas_size: ScreenRes) -> egui::Response {
        self.add(
            egui::Image::new(egui::load::SizedTexture::new(
                egui::TextureId::User(0),
                rect.to_egui_size(),
            ))
            .uv(rect.to_egui_uv(atlas_size)),
        )
    }
    fn graphic_image_button(&mut self, rect: &IntRect, atlas_size: ScreenRes) -> egui::Response {
        self.add(
            egui::ImageButton::new(egui::load::SizedTexture::new(
                egui::TextureId::User(0),
                rect.to_egui_size(),
            ))
            .uv(rect.to_egui_uv(atlas_size)),
        )
    }
}
