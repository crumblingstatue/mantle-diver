use {
    extension_traits::extension,
    mdv_data::tile::{TileBb, TileDb, TileDef, TileLayer},
};

pub mod tiledb_edit_ui;

use {
    crate::{math::TILE_SIZE, texture_atlas::RectMap},
    mdv_math::types::IntRect,
    std::fmt::Debug,
};

#[extension(pub trait TileDbExt)]
impl TileDb {
    fn update_rects(&mut self, rects: &RectMap) {
        update_rect_def(&mut self.unknown_bg, rects);
        update_rect_def(&mut self.unknown_mid, rects);
        update_rect_db(&mut self.bg, rects);
        update_rect_db(&mut self.mid, rects);
    }
}

fn update_rect_db<Layer: TileLayer>(db: &mut Vec<TileDef<Layer>>, rects: &RectMap)
where
    Layer::SpecificDef: Debug,
{
    for def in db {
        update_rect_def(def, rects);
    }
}

fn update_rect_def<Layer: TileLayer>(
    def: &mut TileDef<Layer>,
    rects: &std::collections::HashMap<String, IntRect>,
) where
    Layer::SpecificDef: Debug,
{
    if !def.graphic_name.is_empty() {
        if let Some(rect) = rects.get(def.graphic_name.as_str()) {
            def.tex_rect = *rect;
            log::info!("Updated rect for {}: {:?}", def.graphic_name.as_str(), rect);
        } else {
            log::error!("Missing texture for {:?}", def.graphic_name.as_str());
        }
    } else {
        log::warn!("Empty graphic name!");
    }
}

const DEFAULT_TILE_BB: TileBb = TileBb {
    x: 0,
    y: 0,
    w: TILE_SIZE,
    h: TILE_SIZE,
};
