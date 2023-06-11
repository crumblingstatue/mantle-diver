use {
    super::App,
    crate::{
        command::Cmd, game::GameState, graphics::ScreenVec, math::WorldPos, player::MovingEnt,
        res::Res, texture_atlas::AtlasBundle,
    },
    sfml::graphics::RenderTarget,
    std::fmt::Write,
};

pub(super) fn dispatch(app: &mut App, res: &mut Res, mouse_world_pos: WorldPos) {
    for cmd in std::mem::take(&mut app.cmdvec) {
        match cmd {
            Cmd::QuitApp => app.should_quit = true,
            Cmd::ToggleFreecam => app.debug.freecam ^= true,
            Cmd::Teleport { pos, relative } => {
                if let Ok(mov) = app
                    .game
                    .ecw
                    .query_one_mut::<&mut MovingEnt>(app.game.controlled_en)
                {
                    if relative {
                        let s2dc = pos.to_s2dc();
                        mov.mob.en.pos.x += s2dc.x;
                        mov.mob.en.pos.y += s2dc.y;
                    } else {
                        mov.mob.en.pos = pos.to_s2dc()
                    }
                }
            }
            Cmd::TeleportCursor => {
                if let Ok(mov) = app
                    .game
                    .ecw
                    .query_one_mut::<&mut MovingEnt>(app.game.controlled_en)
                {
                    mov.mob.en.pos = mouse_world_pos.to_s2dc();
                }
            }
            Cmd::TeleportSpawn => {
                if let Ok(mov) = app
                    .game
                    .ecw
                    .query_one_mut::<&mut MovingEnt>(app.game.controlled_en)
                {
                    mov.mob.en.pos = app.game.spawn_point.to_s2dc()
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
                app.adapt_to_window_size_and_scale(ScreenVec::from_sf_resolution(app.rw.size()));
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
