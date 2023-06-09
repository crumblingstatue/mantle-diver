use {
    super::App,
    crate::{
        command::Cmd, game::GameState, math::WorldPos, player::PlayerQuery, res::Res,
        texture_atlas::AtlasBundle,
    },
    sfml::{graphics::RenderTarget, system::Vector2u},
    std::fmt::Write,
};

pub(super) fn dispatch(app: &mut App, res: &mut Res, mouse_world_pos: WorldPos) {
    for cmd in std::mem::take(&mut app.cmdvec) {
        match cmd {
            Cmd::QuitApp => app.should_quit = true,
            Cmd::ToggleFreecam => app.debug.freecam ^= true,
            Cmd::TeleportPlayer { pos, relative } => {
                for (_en, plr) in app.game.ecw.query_mut::<PlayerQuery>() {
                    if relative {
                        let s2dc = pos.to_s2dc();
                        plr.mov.mob.en.pos.x += s2dc.x;
                        plr.mov.mob.en.pos.y += s2dc.y;
                    } else {
                        plr.mov.mob.en.pos = pos.to_s2dc()
                    }
                }
            }
            Cmd::TeleportPlayerCursor => {
                for (_en, plr) in app.game.ecw.query_mut::<PlayerQuery>() {
                    plr.mov.mob.en.pos.x = mouse_world_pos.x as i32;
                    plr.mov.mob.en.pos.y = mouse_world_pos.y as i32;
                }
            }
            Cmd::TeleportPlayerSpawn => {
                for (_en, plr) in app.game.ecw.query_mut::<PlayerQuery>() {
                    plr.mov.mob.en.pos = app.game.spawn_point.to_s2dc()
                }
            }
            Cmd::GiveItemByName { name, amount } => {
                for (id, item) in app.game.itemdb.iter() {
                    if item.name == name {
                        app.game.inventory.add(id, amount);
                        return;
                    }
                }
                writeln!(
                    &mut app.debug.console.log,
                    "Item with name '{name}' not found"
                )
                .unwrap();
            }
            Cmd::ToggleTileDbEdit => app.debug.tiledb_edit.open ^= true,
            Cmd::SetScale(scale) => {
                app.scale = scale;
                let Vector2u { x, y } = app.rw.size();
                app.adapt_to_window_size_and_scale(x, y);
            }
            Cmd::LoadWorld(name) => {
                app.game.world.save();
                let path = app.worlds_dir.join(&name);
                app.game = GameState::new(name, path, res);
            }
            Cmd::ReloadGraphics => {
                res.atlas = AtlasBundle::new(&app.cfg.res_folder_path).unwrap();
                app.game.tile_db.update_rects(&res.atlas.rects);
            }
            Cmd::SetBgTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).bg = id;
            }
            Cmd::SetMidTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).mid = id;
            }
            Cmd::SetFgTileAtCursor(id) => {
                app.game.world.tile_at_mut(app.last_mouse_tpos).fg = id;
            }
        }
    }
}
