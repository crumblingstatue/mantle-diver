use {
    self::{entity_list::EntityList, item_db_edit::ItemDbEdit},
    crate::{math::WorldRect, player::PlayerQuery},
    gamedebug_core::MsgBuf,
};

mod char_db_edit;
mod console;
mod entity_list;
mod item_db_edit;
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
    pub entity_list: EntityList,
    world_mgr: WorldManager,
}

fn debug_panel_ui(debug: &mut DebugState, game: &mut GameState, ctx: &egui::Context) {
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

            for (_en, plr) in game.ecw.query_mut::<PlayerQuery>() {
                ui.collapsing("Player", |ui| {
                    ui.label(format!("Depth: {}", plr.mov.depth_disp()));
                    ui.label(format!(
                        "Offset from center: {}",
                        LengthDisp(plr.mov.mob.en.pos.x as f32 - WorldPos::CENTER as f32)
                    ));
                    ui.label(format!(
                        "Hspeed: {} ({} km/h)",
                        plr.mov.hspeed,
                        px_per_frame_to_km_h(plr.mov.hspeed)
                    ));
                    ui.label(format!(
                        "Vspeed: {} ({} km/h)",
                        plr.mov.vspeed,
                        px_per_frame_to_km_h(plr.mov.vspeed)
                    ));
                    ui.heading("Colors");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Skin");
                        color_edit_button(ui, &mut plr.dat.skin_color);
                        ui.label("Eye");
                        color_edit_button(ui, &mut plr.dat.eye_color);
                        ui.label("Hair");
                        color_edit_button(ui, &mut plr.dat.hair_color);
                        ui.label("Shirt");
                        color_edit_button(ui, &mut plr.dat.shirt_color);
                        ui.label("Pants");
                        color_edit_button(ui, &mut plr.dat.pants_color);
                        ui.label("Shoes");
                        color_edit_button(ui, &mut plr.dat.shoes_color);
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

pub(crate) fn do_debug_ui(
    ctx: &egui::Context,
    debug: &mut DebugState,
    game: &mut GameState,
    res: &mut Res,
    cmd: &mut CmdVec,
    worlds_path: &Path,
) {
    debug_panel_ui(debug, game, ctx);
    debug.tiledb_edit.ui(
        ctx,
        &mut game.tile_db,
        &game.itemdb,
        res.atlas.tex.size(),
        cmd,
    );
    debug.chardb_edit.ui(ctx, &mut game.char_db);
    debug.itemdb_edit.ui(ctx, &mut game.itemdb);
    console_ui(ctx, debug, cmd);
    debug.world_mgr.ui(ctx, game, worlds_path, cmd);
    debug.entity_list.ui(ctx, game);
}

pub enum DbgOvr {
    WldRect { r: WorldRect, c: Color },
}

pub static DBG_OVR: MsgBuf<DbgOvr> = MsgBuf::new(false);
