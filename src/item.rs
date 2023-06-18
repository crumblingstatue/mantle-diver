use {
    crate::math::TILE_SIZE, extension_traits::extension, mdv_data::item::ItemDb,
    mdv_math::types::IntRect,
};

#[extension(pub trait ItemDbExt)]
impl ItemDb {
    fn update_rects(&mut self, rects: &std::collections::HashMap<String, IntRect>) {
        for def in &mut self.db {
            if !def.graphic_name.is_empty() {
                if let Some(rect) = rects.get(def.graphic_name.as_str()) {
                    def.tex_rect = *rect;
                    def.tex_rect.w = rect.w.min(TILE_SIZE.into());
                    def.tex_rect.h = rect.h.min(TILE_SIZE.into());
                    log::info!("Updated rect for {}: {:?}", def.graphic_name.as_str(), rect);
                } else {
                    log::error!("Missing texture for {:?}", def.graphic_name.as_str());
                }
            } else {
                log::warn!("Empty graphic name!");
            }
        }
    }
}
