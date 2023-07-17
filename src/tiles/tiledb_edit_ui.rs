use {
    crate::{
        command::{Cmd, CmdVec},
        graphics::ScreenRes,
        math::{IntRectExt, TILE_SIZE},
    },
    mdv_data::{
        item::ItemId,
        tile::{Bg, BgTileId, Mid, MidTileId, TileDb, TileDef, TileItemDrop, TileLayer},
    },
    mdv_math::types::{ScreenSc, ScreenVec},
    std::fmt::Debug,
};

#[derive(Debug, Default)]
pub struct TileDbEdit {
    pub open: bool,
    layer: Layer,
    selected: usize,
}
impl TileDbEdit {
    pub(crate) fn ui(
        &mut self,
        ctx: &egui::Context,
        tile_db: &mut TileDb,
        item_db: &ItemDb,
        atlas_size: ScreenRes,
        cmd: &mut CmdVec,
    ) {
        if !self.open {
            return;
        }
        egui::Window::new("Tiledb editor")
            .open(&mut self.open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.layer, Layer::Bg, "Bg");
                    ui.selectable_value(&mut self.layer, Layer::Mid, "Mid");
                });
                ui.separator();
                match self.layer {
                    Layer::Bg => db_ui(
                        &mut tile_db.bg,
                        ui,
                        &mut self.selected,
                        item_db,
                        atlas_size,
                        cmd,
                    ),
                    Layer::Mid => db_ui(
                        &mut tile_db.mid,
                        ui,
                        &mut self.selected,
                        item_db,
                        atlas_size,
                        cmd,
                    ),
                }
            });
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Layer {
    Bg,
    Mid,
}

impl Default for Layer {
    fn default() -> Self {
        Self::Bg
    }
}

use {super::DEFAULT_TILE_BB, mdv_data::item::ItemDb};

trait TileLayerExt: TileLayer {
    fn special_ui(_def: &mut Self::SpecificDef, _ui: &mut egui::Ui) {}
    fn paint(idx: usize, cmd: &mut CmdVec);
}

impl TileLayerExt for Bg {
    fn paint(idx: usize, cmd: &mut CmdVec) {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "We don't expect to have more than 65535 tiles"
        )]
        cmd.push(Cmd::SetBgTileAtCursor(BgTileId::from_idx(idx as u16)))
    }
}

impl TileLayerExt for Mid {
    fn special_ui(def: &mut <Mid as TileLayer>::SpecificDef, ui: &mut egui::Ui) {
        match &mut def.bb {
            Some(bb) => {
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut bb.x));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut bb.y));
                    ui.label("w");
                    ui.add(egui::DragValue::new(&mut bb.w));
                    ui.label("h");
                    ui.add(egui::DragValue::new(&mut bb.h));
                });
            }
            None => {
                if ui.button("Insert bb").clicked() {
                    def.bb = Some(DEFAULT_TILE_BB);
                }
            }
        }
        ui.checkbox(&mut def.platform, "platform");
    }

    fn paint(idx: usize, cmd: &mut CmdVec) {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "We don't expect to have more than 65535 tiles"
        )]
        cmd.push(Cmd::SetMidTileAtCursor(MidTileId::from_idx(idx as u16)));
    }
}

fn db_ui<Layer: TileLayer + TileLayerExt + Debug>(
    db: &mut Vec<TileDef<Layer>>,
    ui: &mut egui::Ui,
    sel_idx: &mut usize,
    item_db: &ItemDb,
    atlas_size: ScreenRes,
    cmd: &mut CmdVec,
) where
    <Layer as TileLayer>::SpecificDef: Debug + Default,
{
    if ui.button("Add new default").clicked() {
        db.push(Default::default());
    }
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for (i, def) in db.iter().enumerate() {
                if ui
                    .selectable_label(*sel_idx == i, format!("{} {}", i + 1, def.graphic_name))
                    .clicked()
                {
                    *sel_idx = i;
                }
            }
        });
    ui.separator();
    if let Some(def) = db.get_mut(*sel_idx) {
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(
                    egui::TextureId::User(0),
                    egui::vec2(f32::from(TILE_SIZE), f32::from(TILE_SIZE)),
                )
                .uv(def.tex_rect.to_egui_uv(atlas_size)),
            );
            ui.text_edit_singleline(&mut def.graphic_name);
        });
        ui.checkbox(&mut def.neigh_aware, "Neighbour aware");
        ui.horizontal(|ui| {
            ui.label("Draw offset x");
            ui.add(egui::DragValue::new(&mut def.draw_offs.x));
            ui.label("y");
            ui.add(egui::DragValue::new(&mut def.draw_offs.y));
        });
        ui.horizontal(|ui| {
            ui.label("Blend list");
            if ui.button("✚").clicked() {
                def.blend_list.push(0);
            }
        });
        def.blend_list.retain_mut(|id| {
            let mut retain = true;
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(id));
                if ui.button("➖").clicked() {
                    retain = false;
                }
            });
            retain
        });
        match &mut def.light {
            Some(light) => {
                ui.horizontal(|ui| {
                    ui.label("x");
                    ui.add(egui::DragValue::new(&mut light.x));
                    ui.label("y");
                    ui.add(egui::DragValue::new(&mut light.y));
                });
            }
            None => {
                if ui.button("Insert light emit").clicked() {
                    def.light = Some(ScreenVec {
                        x: ScreenSc::from(TILE_SIZE) / 2,
                        y: ScreenSc::from(TILE_SIZE) / 2,
                    });
                }
            }
        }
        ui.horizontal(|ui| {
            ui.label("Health");
            ui.add(egui::DragValue::new(&mut def.health));
        });
        ui.horizontal(|ui| match &mut def.item_drop {
            Some(drop) => {
                egui::ComboBox::new("itemdrop_combo", "Item drop")
                    .selected_text(
                        item_db
                            .get(drop.id)
                            .map(|def| &def.name[..])
                            .unwrap_or("Nothing"),
                    )
                    .show_ui(ui, |ui| {
                        for (id, item_def) in item_db.iter() {
                            if ui.selectable_label(drop.id == id, &item_def.name).clicked() {
                                drop.id = id;
                            }
                        }
                    });
                ui.label("Amount range");
                let (mut start, mut end) = (*drop.qty_range.start(), *drop.qty_range.end());
                ui.add(egui::DragValue::new(&mut start));
                ui.add(egui::DragValue::new(&mut end));
                drop.qty_range = start..=end;
            }
            None => {
                if ui.button("Add drop").clicked() {
                    def.item_drop = Some(TileItemDrop {
                        qty_range: 0..=0,
                        id: ItemId::EMPTY,
                    })
                }
            }
        });
        match &mut def.hit_sound {
            Some(snd) => {
                ui.text_edit_singleline(snd);
            }
            None => {
                if ui.button("Add hit sound").clicked() {
                    def.hit_sound = Some(String::default());
                }
            }
        }
        ui.checkbox(&mut def.uprootable, "Uprootable");
        Layer::special_ui(&mut def.layer, ui);
        ui.label("O to paint tile at cursor");
        if ui.input(|inp| inp.key_down(egui::Key::O)) {
            Layer::paint(*sel_idx + 1, cmd)
        }
    }
}
