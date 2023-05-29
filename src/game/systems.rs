use {
    super::{Biome, GameState, TransientBlockState},
    crate::{
        app::{SoundPlayer, TileColEn},
        game::for_each_tile_on_screen,
        input::Input,
        inventory::{self, ItemId, UseAction},
        itemdrop::Itemdrop,
        math::{step_towards, WorldPos, TILE_SIZE},
        res::{Res, ResAudio},
        tiles::{self, TileDb, TileDef, TileId},
        world::TilePos,
    },
    gamedebug_core::imm_dbg,
    rand::{seq::SliceRandom, thread_rng, Rng},
    rodio::Decoder,
    sfml::{system::Vector2u, window::Key},
    std::ops::Index,
};

pub(super) fn item_use_system(
    game: &mut GameState,
    input: &Input,
    mouse_tpos: TilePos,
    aud: &ResAudio,
    snd: &mut SoundPlayer,
) {
    if !input.lmb_down {
        return;
    }
    let Some(active_slot) = game.inventory.slots.get_mut(game.selected_inv_slot) else {
        log::error!("Selected slot {} out of bounds", game.selected_inv_slot);
        return;
    };
    if active_slot.qty == 0 {
        return;
    }
    let Some(itemdef) = &game.itemdb.get(active_slot.id) else {
        return;
    };
    let ticks = game.world.ticks;
    let t = game.world.tile_at_mut(mouse_tpos);
    match &itemdef.use_action {
        UseAction::PlaceBgTile { id } => {
            if t.bg.empty() {
                t.bg = *id;
                active_slot.qty -= 1;
            }
        }
        UseAction::PlaceMidTile { id } => {
            if t.mid.empty() {
                t.mid = *id;
                active_slot.qty -= 1;
            }
        }
        UseAction::PlaceFgTile { id } => {
            if t.fg.empty() {
                t.fg = *id;
                active_slot.qty -= 1;
            }
        }
        UseAction::RemoveTile { layer } => match layer {
            inventory::TileLayer::Bg => t.bg = TileId::EMPTY,
            inventory::TileLayer::Mid => t.mid = TileId::EMPTY,
            inventory::TileLayer::Fg => t.fg = TileId::EMPTY,
        },
        UseAction::MineTile { power, delay } => 'block: {
            if t.mid == TileId::EMPTY || ticks - game.last_mine_attempt < *delay {
                break 'block;
            }
            let tdef = &game.tile_db[t.mid];
            let state =
                game.transient_block_state
                    .entry(mouse_tpos)
                    .or_insert(TransientBlockState {
                        health: tdef.health,
                        rot: 0.0,
                        scale: 1.0,
                    });
            let mut rng = thread_rng();
            let abs_rot = rng.gen_range(8.0..=16.0);
            let max_scale = rng.gen_range(1.1..=1.3);
            let min_scale = rng.gen_range(0.8..=0.9);
            state.rot = *[-abs_rot, abs_rot].choose(&mut rng).unwrap();
            state.scale = *[min_scale, max_scale].choose(&mut rng).unwrap();
            state.health -= power;
            if let Some(hit_snd) = &tdef.hit_sound {
                snd.play(aud, hit_snd);
            }
            game.last_mine_attempt = ticks;
        }
    }
    // Make sure that fully consumed stacks are cleared
    if active_slot.qty == 0 {
        active_slot.id = ItemId::EMPTY;
    }
}

pub(super) fn biome_watch_system(game: &mut GameState, music_sink: &mut rodio::Sink, res: &Res) {
    if game.camera_offset.y > 642_000 {
        game.current_biome = Biome::Underground;
    } else {
        game.current_biome = Biome::Surface;
    }
    if game.current_biome != game.prev_biome {
        game.prev_biome = game.current_biome;
        match game.current_biome {
            Biome::Surface => {
                if !music_sink.empty() {
                    music_sink.clear();
                }
                music_sink.append(Decoder::new_looped(res.surf_music.clone()).unwrap());
                music_sink.play();
            }
            Biome::Underground => {
                if !music_sink.empty() {
                    music_sink.clear();
                }
                music_sink.append(Decoder::new_looped(res.und_music.clone()).unwrap());
                music_sink.play();
            }
        }
    }
}

pub(super) fn player_move_system(
    game: &mut GameState,
    input: &Input,
    rt_size: Vector2u,
    on_screen_tile_ents: &mut Vec<TileColEn>,
) {
    let spd = if input.down(Key::LShift) {
        8.0
    } else if input.down(Key::LControl) {
        128.0
    } else {
        3.0
    };
    game.world.player.hspeed = 0.;
    if input.down(Key::A) {
        game.world.player.hspeed = -spd;
    }
    if input.down(Key::D) {
        game.world.player.hspeed = spd;
    }
    if input.down(Key::W) && game.world.player.can_jump() {
        game.world.player.vspeed = -10.0;
        game.world.player.jumps_left = 0;
    }
    game.world.player.down_intent = input.down(Key::S);
    let terminal_velocity = 60.0;
    game.world.player.vspeed = game
        .world
        .player
        .vspeed
        .clamp(-terminal_velocity, terminal_velocity);
    on_screen_tile_ents.clear();
    for_each_tile_on_screen(game.camera_offset, rt_size, |tp, _sp| {
        let tile = game.world.tile_at_mut(tp).mid;
        if tile.empty() {
            return;
        }
        let tdef = &game.tile_db[tile];
        let Some(bb) = tdef.layer.bb else {
            return;
        };
        let x = tp.x as i32 * TILE_SIZE as i32;
        let y = tp.y as i32 * TILE_SIZE as i32;
        let en = s2dc::Entity::from_rect_corners(
            x + bb.x as i32,
            y + bb.y as i32,
            x + bb.w as i32,
            y + bb.h as i32,
        );
        on_screen_tile_ents.push(TileColEn {
            col: en,
            platform: tdef.layer.platform,
        });
    });
    imm_dbg!(on_screen_tile_ents.len());
    game.world
        .player
        .col_en
        .move_y(game.world.player.vspeed, |player_en, off| {
            let mut col = false;
            for en in on_screen_tile_ents.iter() {
                if player_en.would_collide(&en.col, off) {
                    if en.platform {
                        if game.world.player.vspeed < 0. {
                            continue;
                        }
                        // If the player's feet are below the top of the platform,
                        // collision shouldn't happen
                        let player_feet = player_en.pos.y + player_en.bb.y;
                        if player_feet > en.col.pos.y || game.world.player.down_intent {
                            continue;
                        }
                    }
                    col = true;
                    if game.world.player.vspeed > 0. {
                        game.world.player.jumps_left = 1;
                    }
                    game.world.player.vspeed = 0.;
                }
            }
            col
        });
    game.world
        .player
        .col_en
        .move_x(game.world.player.hspeed, |player_en, off| {
            let mut col = false;
            for en in on_screen_tile_ents.iter() {
                if en.platform {
                    continue;
                }
                if player_en.would_collide(&en.col, off) {
                    col = true;
                    game.world.player.hspeed = 0.;
                }
            }
            col
        });
    game.world.player.vspeed += game.gravity;
    let (x, y, _w, _h) = game.world.player.col_en.en.xywh();
    game.camera_offset.x = (x - rt_size.x as i32 / 2).try_into().unwrap_or(0);
    game.camera_offset.y = (y - rt_size.y as i32 / 2).try_into().unwrap_or(0);
}
pub(super) fn freecam_move_system(game: &mut GameState, mouse_world_pos: WorldPos, input: &Input) {
    let spd = if input.down(Key::LShift) {
        100
    } else if input.down(Key::LControl) {
        1000
    } else {
        2
    };
    if input.down(Key::A) {
        game.camera_offset.x = game.camera_offset.x.saturating_sub(spd);
    }
    if input.down(Key::D) {
        game.camera_offset.x = game.camera_offset.x.saturating_add(spd);
    }
    if input.down(Key::W) {
        game.camera_offset.y = game.camera_offset.y.saturating_sub(spd);
    }
    if input.down(Key::S) {
        game.camera_offset.y = game.camera_offset.y.saturating_add(spd);
    }
    if input.pressed(Key::P) {
        game.world.player.col_en.en.pos.x = mouse_world_pos.x as i32;
        game.world.player.col_en.en.pos.y = mouse_world_pos.y as i32;
    }
}

pub(super) fn inventory_input_system(game: &mut GameState, input: &Input) {
    if input.pressed(Key::Num1) {
        game.selected_inv_slot = 0;
    }
    if input.pressed(Key::Num2) {
        game.selected_inv_slot = 1;
    }
    if input.pressed(Key::Num3) {
        game.selected_inv_slot = 2;
    }
    if input.pressed(Key::Num4) {
        game.selected_inv_slot = 3;
    }
    if input.pressed(Key::Num5) {
        game.selected_inv_slot = 4;
    }
    if input.pressed(Key::Num6) {
        game.selected_inv_slot = 5;
    }
    if input.pressed(Key::Num7) {
        game.selected_inv_slot = 6;
    }
    if input.pressed(Key::Num8) {
        game.selected_inv_slot = 7;
    }
    if input.pressed(Key::Num9) {
        game.selected_inv_slot = 8;
    }
    if input.pressed(Key::Num0) {
        game.selected_inv_slot = 9;
    }
}
/// Update transient blocks
pub(super) fn transient_blocks_system(game: &mut GameState) {
    game.transient_block_state.retain(|pos, state| {
        step_towards(&mut state.rot, 0.0, 0.9);
        step_towards(&mut state.scale, 1.0, 0.04);
        // Kill tiles with 0 health
        let mut retain = true;
        if state.health <= 0.0 {
            let tile = &mut game.world.tile_at_mut(*pos);
            process_tile_item_drop(&game.tile_db, &mut game.item_drops, tile.mid, pos);
            tile.mid = TileId::EMPTY;
            // If the mid is destroyed, the front content pops off as well
            process_tile_item_drop(&game.tile_db, &mut game.item_drops, tile.fg, pos);
            tile.fg = TileId::EMPTY;
            retain = false;
        }
        retain
    });
}
/// Claim item drops player contacts with
pub(super) fn item_drop_claim_system(
    game: &mut GameState,
    on_screen_tile_ents: &[TileColEn],
    snd: &mut SoundPlayer,
    aud: &ResAudio,
) {
    game.item_drops.retain_mut(|itemdrop| {
        itemdrop.s2dc_en.move_x(itemdrop.hspeed, |en, off| {
            for t_en in on_screen_tile_ents {
                if en.would_collide(&t_en.col, off) {
                    itemdrop.hspeed = 0.0;
                    return true;
                }
            }
            false
        });
        step_towards(&mut itemdrop.hspeed, 0.0, 0.03);
        itemdrop.s2dc_en.move_y(itemdrop.vspeed, |en, off| {
            for t_en in on_screen_tile_ents {
                if en.would_collide(&t_en.col, off) {
                    itemdrop.vspeed = 0.0;
                    return true;
                }
            }
            false
        });
        itemdrop.vspeed += game.gravity;
        let mut retain = true;
        #[expect(clippy::collapsible_if)]
        if game.world.player.col_en.en.collides(&itemdrop.s2dc_en.en) {
            if game.inventory.add(itemdrop.id, 1) {
                snd.play(aud, "etc/pickup");
                retain = false;
            }
        }
        retain
    });
}

fn process_tile_item_drop<L: tiles::TileLayer>(
    tile_db: &TileDb,
    item_drops: &mut Vec<Itemdrop>,
    id: TileId<L>,
    pos: &TilePos,
) where
    TileDb: Index<TileId<L>, Output = TileDef<L>>,
{
    if id.empty() {
        log::warn!("Empty tile id: {id:?}");
        return;
    }
    let Some(drop) = &tile_db[id].item_drop else {
        return
    };
    let amount = thread_rng().gen_range(drop.qty_range.clone());
    for _ in 0..amount {
        item_drops.push(Itemdrop::new_at(
            drop.id,
            WorldPos {
                x: pos.x * TILE_SIZE as u32 + TILE_SIZE as u32 / 2,
                y: pos.y * TILE_SIZE as u32 + TILE_SIZE as u32 / 2,
            },
        ));
    }
}

pub(super) fn pause_menu_system(game: &mut GameState, input: &Input) {
    if input.pressed(Key::Escape) {
        game.menu_open = false;
    }
}

pub(crate) fn general_input_system(game: &mut GameState, input: &Input) {
    if input.pressed(Key::Escape) {
        game.menu_open = true;
    }
}
