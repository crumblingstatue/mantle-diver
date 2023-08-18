use {
    self::{
        entity_list::EntityList, graphic_picker::GraphicPicker, item_db_edit::ItemDbEdit,
        recipe_edit::RecdbEd,
    },
    crate::{
        config::Config,
        graphics::ScreenRes,
        math::WorldRect,
        player::{MovingEnt, PlayerColors},
    },
    egui::Color32,
    gamedebug_core::MsgBuf,
};

mod char_db_edit;
mod console;
mod entity_list;
pub mod graphic_picker;
mod item_db_edit;
mod recipe_edit;
mod world_manager;

use {
    self::{
        char_db_edit::CharDbEdit,
        console::{console_ui, Console},
        world_manager::WorldManager,
    },
    crate::{
        command::CmdVec,
        game::GameState,
        math::{px_per_frame_to_km_h, WorldPos},
        res::Res,
        stringfmt::LengthDisp,
        tiles::tiledb_edit_ui::TileDbEdit,
    },
    gamedebug_core::IMMEDIATE,
    sfml::graphics::Color,
    std::path::Path,
};

#[derive(Default)]
pub struct DebugState {
    pub panel: bool,
    pub freecam: bool,
    pub tiledb_edit: TileDbEdit,
    pub show_atlas: bool,
    pub console: Console,
    pub dbg_overlay: bool,
    pub chardb_edit: CharDbEdit,
    pub itemdb_edit: ItemDbEdit,
    pub graphic_picker: GraphicPicker,
    pub recdb_edit: RecdbEd,
    pub entity_list: EntityList,
    pub mouse_wpos: WorldPos,
    world_mgr: WorldManager,
}

fn debug_panel_ui(
    debug: &mut DebugState,
    game: &mut GameState,
    ctx: &egui::Context,
    cfg: &mut Config,
) {
    let mut open = debug.panel;
    egui::Window::new("Debug (F12)")
        .open(&mut open)
        .show(ctx, |ui| {
            if debug.freecam {
                ui.label("Cam x");
                ui.add(egui::DragValue::new(&mut game.camera_offset.x));
                ui.label("Cam y");
                ui.add(egui::DragValue::new(&mut game.camera_offset.y));
                let co = game.camera_offset;
                ui.label(format!(
                    "Cam Depth: {}",
                    LengthDisp(co.y as f32 - WorldPos::SURFACE as f32)
                ));
                ui.label(format!(
                    "Cam offset from center: {}",
                    LengthDisp(co.x as f32 - WorldPos::CENTER as f32)
                ));
            }
            ui.collapsing("Controlled entity", |ui| {
                match game.ecw.query_one_mut::<&mut MovingEnt>(game.controlled_en) {
                    Ok(mov) => {
                        ui.label(format!("Depth: {}", mov.depth_disp()));
                        ui.label(format!(
                            "Offset from center: {}",
                            LengthDisp(mov.mob.en.pos.x as f32 - WorldPos::CENTER as f32)
                        ));
                        ui.label(format!(
                            "Hspeed: {} ({} km/h)",
                            mov.hspeed,
                            px_per_frame_to_km_h(mov.hspeed)
                        ));
                        ui.label(format!(
                            "Vspeed: {} ({} km/h)",
                            mov.vspeed,
                            px_per_frame_to_km_h(mov.vspeed)
                        ));
                    }
                    Err(e) => {
                        ui.label(e.to_string());
                    }
                }
            });
            for (_en, dat) in game.ecw.query_mut::<&mut PlayerColors>() {
                ui.collapsing("PlayerData", |ui| {
                    ui.heading("Colors");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Skin");
                        color_edit_button(ui, &mut dat.skin);
                        ui.label("Eye");
                        color_edit_button(ui, &mut dat.eye);
                        ui.label("Hair");
                        color_edit_button(ui, &mut dat.hair);
                        ui.label("Shirt");
                        color_edit_button(ui, &mut dat.shirt);
                        ui.label("Pants");
                        color_edit_button(ui, &mut dat.pants);
                        ui.label("Shoes");
                        color_edit_button(ui, &mut dat.shoes);
                    });
                });
                if ui
                    .checkbox(&mut debug.dbg_overlay, "Draw debug overlay")
                    .clicked()
                {
                    DBG_OVR.toggle();
                }
                ui.separator();
            }
            ui.collapsing("Config", |ui| {
                ui.label("Inv frame color");
                cfg.ui.inv_frame_color.as_sf_mut(|c| {
                    color_edit_button_srgba(ui, c);
                });
                ui.label("Inv frame highlight color");
                cfg.ui.inv_frame_highlight.as_sf_mut(|c| {
                    color_edit_button_srgba(ui, c);
                });
                ui.label("Inv bg color");
                cfg.ui.inv_bg_color.as_sf_mut(|c| {
                    color_edit_button_srgba(ui, c);
                });
            });
            ui.label("Elapsed ticks");
            ui.add(egui::DragValue::new(&mut game.world.ticks));
            egui::ScrollArea::vertical().show(ui, |ui| {
                IMMEDIATE.for_each(|msg| {
                    ui.label(msg);
                });
            });
            IMMEDIATE.clear();
        });
    debug.panel = open;
}

fn color_edit_button(ui: &mut egui::Ui, c: &mut Color) {
    let mut rgb = [c.r, c.g, c.b];
    ui.color_edit_button_srgb(&mut rgb);
    c.r = rgb[0];
    c.g = rgb[1];
    c.b = rgb[2];
}

fn color_edit_button_srgba(ui: &mut egui::Ui, c: &mut Color) {
    let mut rgba = Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a);
    ui.color_edit_button_srgba(&mut rgba);
    c.r = rgba.r();
    c.g = rgba.g();
    c.b = rgba.b();
    c.a = rgba.a();
}

pub(crate) fn do_debug_ui(
    ctx: &egui::Context,
    debug: &mut DebugState,
    game: &mut GameState,
    res: &Res,
    cmd: &mut CmdVec,
    worlds_path: &Path,
    cfg: &mut Config,
) {
    debug_panel_ui(debug, game, ctx, cfg);
    let atlas_size = ScreenRes::from_sf_vec(res.atlas.tex.size());
    debug.tiledb_edit.ui(
        ctx,
        &mut game.tile_db,
        &game.itemdb,
        atlas_size,
        cmd,
        &mut debug.graphic_picker,
    );
    debug.chardb_edit.ui(ctx, &mut game.char_db);
    debug
        .itemdb_edit
        .ui(ctx, atlas_size, &mut debug.graphic_picker, game);
    debug.recdb_edit.ui(
        ctx,
        &mut game.recipe_db,
        &game.itemdb,
        &mut debug.itemdb_edit,
    );
    console_ui(ctx, debug, cmd);
    debug.world_mgr.ui(ctx, game, worlds_path, cmd);
    debug.entity_list.ui(ctx, game);
    debug.graphic_picker.ui(ctx, atlas_size, res);
}

#[expect(
    clippy::enum_variant_names,
    reason = "So far we only have Wld debug, could change"
)]
pub enum DbgOvr {
    WldRect {
        r: WorldRect,
        c: Color,
    },
    WldCircle {
        pos: WorldPos,
        radius: u16,
        c: Color,
    },
    WldLine {
        p1: WorldPos,
        p2: WorldPos,
    },
}

pub static DBG_OVR: MsgBuf<DbgOvr> = MsgBuf::new(false);
