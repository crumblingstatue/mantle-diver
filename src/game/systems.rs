use {
    self::pause_menu::open_menu,
    super::{events::Event, Biome, GameState, TransientTileState, TransientTileStates},
    crate::{
        app::TileColEn,
        audio::AudioCtx,
        data,
        debug::{DbgOvr, DebugState, DBG_OVR},
        egui_ext::EguiUiExt,
        graphics::ScreenRes,
        input::{Input, InputAction},
        itemdrop::{ItemdropBundle, PickupCooldown},
        math::{world_y_depth, WorldPos, WorldRect, TILE_SIZE},
        player::{FacingDir, Health, MoveExtra, MovingEnt},
        res::{Res, ResAudio},
        world::{TilePos, World},
    },
    egui_sfml::egui,
    mdv_data::{
        item::{ItemId, ItemStack, UseAction},
        tile::{LayerAccess, TileDb, TileDef, TileId, TileLayer},
    },
    mdv_math::{types::ScreenVec, util::step_towards},
    rand::{seq::SliceRandom, thread_rng, Rng},
    sfml::{graphics::Color, window::Key},
    std::ops::{ControlFlow, Index},
};

pub mod pause_menu;

enum DominantOffset {
    Horizontal,
    Vertical,
}

/// Determines whether `target` is more horizontally or more vertically offset from `source`.
#[expect(clippy::cast_possible_wrap, reason = "Positions can fit in i32")]
fn dominant_offset(source: WorldPos, target: WorldPos) -> DominantOffset {
    let x_offset = (target.x as i32 - source.x as i32).abs();
    let y_offset = (target.y as i32 - source.y as i32).abs();

    if x_offset > y_offset {
        DominantOffset::Horizontal
    } else {
        DominantOffset::Vertical
    }
}

pub(super) fn item_use_system(
    game: &mut GameState,
    input: &Input,
    mouse_tpos: TilePos,
    au_res: &ResAudio,
    au_ctx: &mut AudioCtx,
    mouse_wpos: WorldPos,
    debug: &DebugState,
) {
    let Ok(mov) = game.ecw.query_one_mut::<&mut MovingEnt>(game.controlled_en) else {
        log::warn!("No controlled entity");
        return;
    };
    // Without an active item, we don't even know how we want to do targeting
    let Some(active_slot) = game.inventory.slots.get_mut(game.ui.selected_inv_slot) else {
        log::error!("Selected slot {} out of bounds", game.ui.selected_inv_slot);
        return;
    };
    if active_slot.qty == 0 {
        return;
    }
    let Some(itemdef) = &game.itemdb.get(active_slot.id) else {
        return;
    };
    // If we want to mine a mid tile, we need to do a specific kind of targeting
    let mine_targeting = itemdef.use1.is_mid_mine();
    let player_pos = WorldPos::from_en(&mov.mob.en);
    let ptr_within_circle = mouse_wpos.within_circle(player_pos, game.tile_interact_radius);
    DBG_OVR.push(DbgOvr::WldCircle {
        pos: WorldPos::from_en(&mov.mob.en),
        radius: game.tile_interact_radius,
        c: if ptr_within_circle {
            Color::GREEN
        } else {
            Color::RED
        },
    });
    let mut target_tpos = None;
    if game.smart_cursor && mine_targeting && !debug.freecam {
        // Cast multiple different lines until we succeed
        let sources = match dominant_offset(player_pos, mouse_wpos) {
            DominantOffset::Horizontal => [
                player_pos.tile_pos(),
                player_pos.tile_pos().y_off(-1),
                player_pos.tile_pos().y_off(1),
            ],
            DominantOffset::Vertical => [
                player_pos.tile_pos(),
                player_pos.tile_pos().x_off(-1),
                player_pos.tile_pos().x_off(1),
            ],
        };
        for src_tpos in sources {
            bresenham(src_tpos.to_signed(), mouse_tpos.to_signed(), |tpos| {
                if !tpos
                    .to_world()
                    .within_circle(player_pos, game.tile_interact_radius)
                {
                    return ControlFlow::Break(());
                }
                DBG_OVR.push(DbgOvr::WldRect {
                    r: tpos.tile_world_rect(),
                    c: Color::CYAN,
                });
                let id = game.world.tile_at_mut(tpos).mid;
                if !id.empty() && game.tile_db[id].is_impassable() {
                    target_tpos = Some(tpos);
                    return ControlFlow::Break(());
                }
                ControlFlow::Continue(())
            });
            if target_tpos.is_some() {
                break;
            }
        }
        game.highlight_tp = target_tpos;
    } else {
        game.highlight_tp = None;
        if ptr_within_circle || debug.freecam {
            target_tpos = Some(mouse_tpos);
        }
    }
    DBG_OVR.push(DbgOvr::WldLine {
        p1: player_pos,
        p2: mouse_wpos,
    });
    if !(input.lmb_down || input.rmb_down) {
        return;
    }
    let ticks = game.world.ticks;
    let tile_place_cooldown = 8;
    let action = if input.lmb_down {
        &itemdef.use1
    } else if input.rmb_down {
        &itemdef.use2
    } else {
        return;
    };
    // Not all use actions might need a target tile position, but right now they do
    let Some(tpos_needed_always_fixme) = target_tpos else {
        return;
    };
    do_use_action(
        action,
        &mut game.world,
        ticks,
        tile_place_cooldown,
        active_slot,
        tpos_needed_always_fixme,
        au_ctx,
        au_res,
        &mut game.last_tile_place,
        &mut game.last_mine_attempt,
        &mut game.transient_tile_states,
        &game.tile_db,
        mov,
    );
    // Make sure that fully consumed stacks are cleared
    if active_slot.qty == 0 {
        active_slot.id = ItemId::EMPTY;
    }
}

#[expect(clippy::too_many_arguments)]
fn do_use_action(
    action: &UseAction,
    world: &mut World,
    ticks: u64,
    tile_place_cooldown: u64,
    active_slot: &mut ItemStack,
    target_tpos: TilePos,
    au_ctx: &mut AudioCtx,
    au_res: &ResAudio,
    last_tile_place: &mut u64,
    last_mine_attempt: &mut u64,
    transient_block_states: &mut TransientTileStates,
    tile_db: &TileDb,
    player_mov: &MovingEnt,
) {
    match action {
        UseAction::PlaceBgTile { id } => {
            let mut can_place_this_here = true;
            let above = world.tile_at_mut(target_tpos.y_off(-1)).bg;
            let below = world.tile_at_mut(target_tpos.y_off(1)).bg;
            let left = world.tile_at_mut(target_tpos.x_off(-1)).bg;
            let right = world.tile_at_mut(target_tpos.x_off(1)).bg;
            let t = world.tile_at_mut(target_tpos);
            let is_bg_wall_here = !t.bg.empty();
            // Don't allow placing bg tiles in thin air. They need to be connected to some other bg tile.
            #[expect(clippy::collapsible_else_if, reason = "It's easier to read this way")]
            if is_bg_wall_here {
                return;
            } else {
                if ![above, below, left, right]
                    .into_iter()
                    .any(|id| !id.empty())
                {
                    can_place_this_here = false;
                }
            }
            if can_place_this_here && ticks - *last_tile_place > tile_place_cooldown {
                if let Some(snd) = &tile_db[*id].hit_sound {
                    au_ctx.plr.play(au_res, snd);
                }
                t.bg = *id;
                active_slot.qty -= 1;
                *last_tile_place = ticks;
            }
        }
        UseAction::PlaceMidTile { id } => {
            let mut can_place_this_here = true;
            let above = world.tile_at_mut(target_tpos.y_off(-1)).mid;
            let below = world.tile_at_mut(target_tpos.y_off(1)).mid;
            let left = world.tile_at_mut(target_tpos.x_off(-1)).mid;
            let right = world.tile_at_mut(target_tpos.x_off(1)).mid;
            let t = world.tile_at_mut(target_tpos);
            let is_bg_wall_here = !t.bg.empty();
            // Don't allow placing tiles in thin air. They need to be connected to some other solid block.
            // Or at least there needs to be a background wall there.
            #[expect(clippy::collapsible_if, reason = "It's easier to read this way")]
            if !is_bg_wall_here {
                if ![above, below, left, right].into_iter().any(|id| {
                    if id.empty() {
                        return false;
                    }
                    tile_db[id].layer.bb.is_some()
                }) {
                    can_place_this_here = false;
                }
            }
            if !t.mid.empty() {
                can_place_this_here = false;
            }
            if can_place_this_here && ticks - *last_tile_place > tile_place_cooldown {
                let tdef = &tile_db[*id];
                if tdef.is_impassable() && player_mov.overlaps_tp(target_tpos) {
                    return;
                }
                if let Some(snd) = &tdef.hit_sound {
                    au_ctx.plr.play(au_res, snd);
                }
                t.mid = *id;
                active_slot.qty -= 1;
                *last_tile_place = ticks;
            }
        }
        UseAction::RemoveTile { layer } => {
            let t = world.tile_at_mut(target_tpos);
            match layer {
                LayerAccess::Bg => t.bg = TileId::EMPTY,
                LayerAccess::Mid => t.mid = TileId::EMPTY,
            }
        }
        UseAction::MineTile { power, delay } => {
            let t = world.tile_at_mut(target_tpos);
            mine_tile(
                &t.mid,
                ticks,
                delay,
                target_tpos,
                power,
                au_ctx,
                au_res,
                last_mine_attempt,
                transient_block_states,
                tile_db,
            );
        }
        UseAction::MineBgTile { power, delay } => {
            // Only allow digging walls if they have an empty neighbour.
            //
            // Being able to just dig any wall gives a "light source cheat" at the dirt level,
            // and also diminishes the feeling of being deep underground being able to just
            // poke holes anywhere underground.
            //
            // However, the game recognizes the desire to mine backwalls for environment manipulation
            // and building purposes. This approach tries to strike a balance.
            //
            // TODO: Allow digging walls anywhere for user placed walls.
            // Distinguish them from naturally placed walls, which can't be digged anywhere.
            let empty_above = world.tile_at_mut(target_tpos.y_off(-1)).bg.empty();
            let empty_below = world.tile_at_mut(target_tpos.y_off(1)).bg.empty();
            let empty_left = world.tile_at_mut(target_tpos.x_off(-1)).bg.empty();
            let empty_right = world.tile_at_mut(target_tpos.x_off(1)).bg.empty();
            let has_empty_neighbour = empty_above || empty_below || empty_left || empty_right;
            let t = world.tile_at_mut(target_tpos);
            // Also only allow digging the backwall if the mid tile is empty.
            let empty_mid = t.mid.empty();
            if !has_empty_neighbour || !empty_mid {
                return;
            }
            mine_tile(
                &t.bg,
                ticks,
                delay,
                target_tpos,
                power,
                au_ctx,
                au_res,
                last_mine_attempt,
                transient_block_states,
                tile_db,
            );
        }
        UseAction::Nothing => {}
    }
}

#[expect(clippy::too_many_arguments)]
fn mine_tile<L: TileLayer>(
    tid: &TileId<L>,
    ticks: u64,
    delay: &u64,
    mouse_tpos: TilePos,
    power: &f32,
    au_ctx: &mut AudioCtx,
    au_res: &ResAudio,
    last_mine_attempt: &mut u64,
    transient_block_states: &mut TransientTileStates,
    tile_db: &TileDb,
) where
    TileDb: Index<TileId<L>, Output = TileDef<L>>,
{
    if *tid == TileId::EMPTY || ticks - *last_mine_attempt < *delay {
        return;
    }
    let tdef = &tile_db[*tid];
    let state = transient_block_states
        .entry(super::TilestateKey {
            pos: mouse_tpos,
            layer: L::LAYER,
        })
        .or_insert(TransientTileState {
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
        au_ctx.plr.play(au_res, hit_snd);
    }
    *last_mine_attempt = ticks;
}

pub(super) fn move_system(game: &mut GameState, rt_size: ScreenVec, debug: &DebugState) {
    for (ecs_en, (mov, mut mov_xtra)) in game
        .ecw
        .query_mut::<(&mut MovingEnt, Option<&mut MoveExtra>)>()
    {
        DBG_OVR.push(DbgOvr::WldRect {
            r: WorldRect {
                topleft: WorldPos::from_en(&mov.mob.en),
                w: 3,
                h: 3,
            },
            c: Color::RED,
        });
        let wrect = calc_mov_wrect(mov);
        DBG_OVR.push(DbgOvr::WldRect {
            r: wrect,
            c: Color::YELLOW,
        });
        DBG_OVR.push(DbgOvr::WldRect {
            r: WorldRect::from_s2dc_en(&mov.mob.en),
            c: Color::BLUE,
        });
        let tile_ents: Vec<TileColEn> = calc_tile_ents(&mut game.world, &game.tile_db, wrect);
        for en in &tile_ents {
            DBG_OVR.push(DbgOvr::WldRect {
                r: WorldRect::from_s2dc_en(&en.col),
                c: Color::GREEN,
            });
        }
        let terminal_velocity = 60.0;
        mov.vspeed = mov.vspeed.clamp(-terminal_velocity, terminal_velocity);
        mov.mob.move_y(mov.vspeed, |player_en, off| {
            let mut col = false;
            for en in tile_ents.iter() {
                if player_en.would_collide(&en.col, off) {
                    if en.platform {
                        if mov.vspeed < 0. {
                            continue;
                        }
                        // If the feet are below the top of the platform,
                        // collision shouldn't happen
                        let feet = player_en.pos.y + player_en.bb.y;
                        if feet > en.col.pos.y
                            || mov_xtra.as_ref().map_or(false, |xtra| xtra.down_intent)
                        {
                            continue;
                        }
                    }
                    col = true;
                    if let Some(xtra) = &mut mov_xtra {
                        if mov.vspeed > 0. {
                            xtra.jumps_left = 1;
                        }
                    }
                    game.event_buf.push(Event::GroundHit {
                        en: ecs_en,
                        vspeed: mov.vspeed,
                    });
                    mov.vspeed = 0.;
                }
            }
            col
        });
        mov.mob.move_x(mov.hspeed, |player_en, off| {
            let mut col = false;
            for en in tile_ents.iter() {
                if en.platform {
                    continue;
                }
                if player_en.would_collide(&en.col, off) {
                    col = true;
                    mov.hspeed = 0.;
                }
            }
            col
        });
        mov.vspeed += game.gravity;
        if !debug.freecam && ecs_en == game.controlled_en {
            let (x, y, _w, _h) = mov.mob.en.xywh();
            game.camera_offset.x = (x - i32::from(rt_size.x) / 2).try_into().unwrap_or(0);
            game.camera_offset.y = (y - i32::from(rt_size.y) / 2).try_into().unwrap_or(0);
        }
    }
}

/// Calculate tile check pixel rectangle, for which tiles to check for collision
fn calc_mov_wrect(mov: &MovingEnt) -> WorldRect {
    // Movement speeds are low enough that we don't have to fear truncation
    #[expect(clippy::cast_possible_truncation)]
    let mut hvec = mov.hspeed.round() as i32;
    #[expect(clippy::cast_possible_truncation)]
    let mut vvec = mov.vspeed.round() as i32;
    // We add 1 just to be on the safe side
    hvec += (mov.mob.en.bb.x + 1) * hvec.signum();
    vvec += (mov.mob.en.bb.y + 1) * vvec.signum();
    let y = mov.mob.en.pos.y;
    let x = mov.mob.en.pos.x;
    let top = y.min(y + vvec);
    let left = x.min(x + hvec);
    let w = hvec.unsigned_abs();
    let h = vvec.unsigned_abs();
    #[expect(
        clippy::cast_sign_loss,
        reason = "Entity positions are assumed to never be negative"
    )]
    WorldRect {
        topleft: WorldPos {
            x: left as u32,
            y: top as u32,
        },
        w,
        h,
    }
}

fn calc_tile_ents(world: &mut World, tile_db: &TileDb, wrect: WorldRect) -> Vec<TileColEn> {
    let mut ents = vec![];
    // FIXME: We need to massively overcompensate the size of the area to check
    // due to incorrect/imprecise calculations.
    let x = (wrect.topleft.x / u32::from(TILE_SIZE)) - 1;
    let y = (wrect.topleft.y / u32::from(TILE_SIZE)) - 1;
    let w = (wrect.w / u32::from(TILE_SIZE)) + 4;
    let h = (wrect.h / u32::from(TILE_SIZE)) + 4;
    DBG_OVR.push(DbgOvr::WldRect {
        r: WorldRect {
            topleft: WorldPos {
                x: x * u32::from(TILE_SIZE),
                y: y * u32::from(TILE_SIZE),
            },
            w: w * u32::from(TILE_SIZE),
            h: h * u32::from(TILE_SIZE),
        },
        c: Color::MAGENTA,
    });
    for y in y..y + h {
        for x in x..x + w {
            let tp = TilePos { x, y };
            let tile = world.tile_at_mut(tp).mid;
            if tile.empty() {
                continue;
            }
            let tdef = &tile_db[tile];
            let Some(bb) = tdef.layer.bb else {
                continue;
            };
            let (x, y) = tp.to_s2dc_en_pos();
            let en = s2dc::Entity::from_rect_corners(
                x + i32::from(bb.x),
                y + i32::from(bb.y),
                x + i32::from(bb.w),
                y + i32::from(bb.h),
            );
            ents.push(TileColEn {
                col: en,
                platform: tdef.layer.platform,
            });
        }
    }
    ents
}

pub(super) fn biome_watch_system(game: &mut GameState, au_ctx: &AudioCtx, res: &Res) {
    let depth = world_y_depth(game.camera_offset.y);
    let depth_tiles = depth / i32::from(TILE_SIZE);
    if depth_tiles > 70 {
        game.current_biome = Biome::Underground;
    } else {
        game.current_biome = Biome::Surface;
    }
    if game.current_biome != game.prev_biome {
        game.prev_biome = game.current_biome;
        match game.current_biome {
            Biome::Surface => {
                au_ctx.play_music(&res.surf_music);
            }
            Biome::Underground => {
                au_ctx.play_music(&res.und_music);
            }
        }
    }
}

/// Control the movements of the controlled entity (usually player character)
pub(super) fn move_control_system(game: &mut GameState, input: &Input) {
    let Ok((mov, mov_extra)) = game
        .ecw
        .query_one_mut::<(&mut MovingEnt, &mut MoveExtra)>(game.controlled_en)
    else {
        log::warn!("No controlled entity");
        return;
    };
    let spd = if input.down_raw(Key::LShift) {
        8.0
    } else if input.down_raw(Key::LControl) {
        128.0
    } else {
        3.0
    };
    mov.hspeed = 0.;
    if input.down(InputAction::Left) {
        mov.hspeed = -spd;
        mov_extra.facing_dir = FacingDir::Left;
    }
    if input.down(InputAction::Right) {
        mov.hspeed = spd;
        mov_extra.facing_dir = FacingDir::Right;
    }
    if input.down(InputAction::Jump) && mov_extra.can_jump() {
        mov.vspeed = -10.0;
        mov_extra.jumps_left = 0;
    }
    mov_extra.down_intent = input.down(InputAction::Down);
    if input.pressed(InputAction::ThrowItem) {
        if let Some(stack) = game.inventory.take_from_slot(game.ui.selected_inv_slot, 1) {
            let pos = mov.world_pos();
            let en = game.ecw.spawn(ItemdropBundle::new_at(stack.id, pos));
            game.ecw
                .insert_one(
                    en,
                    PickupCooldown {
                        tick_dropped: game.world.ticks,
                        cooldown: 100,
                    },
                )
                .unwrap();
        }
    }
}

pub(super) fn freecam_move_system(game: &mut GameState, input: &Input) {
    let spd = if input.down_raw(Key::LShift) {
        100
    } else if input.down_raw(Key::LControl) {
        1000
    } else {
        2
    };
    if input.down(InputAction::Left) {
        game.camera_offset.x = game.camera_offset.x.saturating_sub(spd);
    }
    if input.down(InputAction::Right) {
        game.camera_offset.x = game.camera_offset.x.saturating_add(spd);
    }
    if input.down(InputAction::Up) {
        game.camera_offset.y = game.camera_offset.y.saturating_sub(spd);
    }
    if input.down(InputAction::Down) {
        game.camera_offset.y = game.camera_offset.y.saturating_add(spd);
    }
}

pub(super) fn inventory_input_system(game: &mut GameState, input: &Input) {
    if input.pressed_raw(Key::Num1) {
        game.ui.select_inv_slot(0);
    }
    if input.pressed_raw(Key::Num2) {
        game.ui.select_inv_slot(1);
    }
    if input.pressed_raw(Key::Num3) {
        game.ui.select_inv_slot(2);
    }
    if input.pressed_raw(Key::Num4) {
        game.ui.select_inv_slot(3);
    }
    if input.pressed_raw(Key::Num5) {
        game.ui.select_inv_slot(4);
    }
    if input.pressed_raw(Key::Num6) {
        game.ui.select_inv_slot(5);
    }
    if input.pressed_raw(Key::Num7) {
        game.ui.select_inv_slot(6);
    }
    if input.pressed_raw(Key::Num8) {
        game.ui.select_inv_slot(7);
    }
    if input.pressed_raw(Key::Num9) {
        game.ui.select_inv_slot(8);
    }
    if input.pressed_raw(Key::Num0) {
        game.ui.select_inv_slot(9);
    }
    if input.down(InputAction::SelectTorch) {
        if let Some(slot) = game.inventory.find_item_idx(data::item::TORCH) {
            game.ui.selected_inv_slot = slot;
        }
    } else {
        game.ui.selected_inv_slot = game.ui.persistent_selected_inv_slot;
    }
}
/// Update transient blocks
pub(super) fn transient_blocks_system(game: &mut GameState) {
    game.transient_tile_states.retain(|key, state| {
        step_towards(&mut state.rot, 0.0, 0.9);
        step_towards(&mut state.scale, 1.0, 0.04);
        // Kill tiles with 0 health
        let mut retain = true;
        if state.health <= 0.0 {
            let tile = &mut game.world.tile_at_mut(key.pos);
            match key.layer {
                LayerAccess::Bg => {
                    process_tile_item_drop(&game.tile_db, &mut game.ecw, tile.bg, &key.pos);
                    tile.bg = TileId::EMPTY;
                }
                LayerAccess::Mid => {
                    process_tile_item_drop(&game.tile_db, &mut game.ecw, tile.mid, &key.pos);
                    tile.mid = TileId::EMPTY;
                }
            }
            retain = false;
        }
        retain
    });
}
/// Claim item drops player contacts with
pub(super) fn item_drop_claim_system(
    game: &mut GameState,
    au_ctx: &mut AudioCtx,
    au_res: &ResAudio,
) {
    let Ok(mut plr_query) = game.ecw.query_one::<&MovingEnt>(game.player_en) else {
        log::error!("No player query to run item drop claim system on");
        return;
    };
    let Some(plr_mov) = plr_query.get() else {
        log::error!("No player");
        return;
    };
    DBG_OVR.push(DbgOvr::WldCircle {
        pos: plr_mov.world_pos(),
        radius: game.item_pickup_radius,
        c: Color::YELLOW,
    });
    for (en, (id, mov, cd)) in game
        .ecw
        .query::<(&ItemId, &mut MovingEnt, Option<&mut PickupCooldown>)>()
        .iter()
    {
        // Horizontal friction
        step_towards(&mut mov.hspeed, 0.0, 0.03);
        if let Some(cd) = cd
            && game.world.ticks < cd.tick_dropped + cd.cooldown
        {
            return;
        }
        // Player interaction
        // "Magnetism" behavior when player is close to an item drop
        if game.inventory.item_can_be_added(*id, 1)
            && mov.within_radius_of_other(plr_mov, game.item_pickup_radius)
        {
            mov.move_towards_other(plr_mov, 4.0);
        }
        #[expect(clippy::collapsible_if)]
        if plr_mov.mob.en.collides(&mov.mob.en) {
            if game.inventory.add(*id, 1, &game.itemdb) {
                au_ctx.plr.play(au_res, "etc/pickup");
                game.ecb.despawn(en);
            }
        }
    }
    drop(plr_query);
    game.ecb.run_on(&mut game.ecw);
}

fn process_tile_item_drop<L: mdv_data::tile::TileLayer>(
    tile_db: &TileDb,
    wld: &mut hecs::World,
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
        return;
    };
    let amount = thread_rng().gen_range(drop.qty_range.clone());
    for _ in 0..amount {
        wld.spawn(ItemdropBundle::new_at(
            drop.id,
            WorldPos {
                x: pos.x * u32::from(TILE_SIZE) + u32::from(TILE_SIZE) / 2,
                y: pos.y * u32::from(TILE_SIZE) + u32::from(TILE_SIZE) / 2,
            },
        ));
    }
}

pub enum UiHudInputSystemMsg {
    None,
    CursorOccupied,
}

impl UiHudInputSystemMsg {
    pub fn cursor_occupied(&self) -> bool {
        matches!(self, UiHudInputSystemMsg::CursorOccupied)
    }
}

/// Input system for UI/HUD.
#[must_use]
pub(crate) fn ui_hud_input_system(
    game: &mut GameState,
    input: &Input,
    scale: u8,
    screen_res: ScreenRes,
) -> UiHudInputSystemMsg {
    let mut msg = UiHudInputSystemMsg::None;
    if input.pressed_raw(Key::Escape) && !game.ui.menu.open {
        open_menu(game);
    }
    if input.pressed_raw(Key::P) {
        game.paused ^= true;
        game.pause_next_frame = false;
    }
    if input.pressed_raw(Key::Period) {
        game.paused = false;
        game.pause_next_frame = true;
    }
    if input.pressed_raw(Key::R) {
        // Reverse gravity
        game.gravity = -game.gravity;
    }
    if input.pressed(InputAction::Inventory) {
        game.ui.inv.open ^= true;
    }
    if input.pressed(InputAction::Crafting) {
        game.ui.craft.open ^= true;
    }
    if input.pressed(InputAction::SmartCursorToggle) {
        game.smart_cursor ^= true;
    }
    let mp = input.mouse_down_loc.scaled(scale);
    if game.ui.inv.open
        && crate::game::ui::Inventory::screen_rect(screen_res, game.inventory.slots.len())
            .contains_screen_pos(mp)
    {
        msg = UiHudInputSystemMsg::CursorOccupied;
    }
    if input.lmb_pressed {
        for (i, rect) in game.ui.hotbar_rects.iter().enumerate() {
            if rect.contains_screen_pos(mp) {
                game.ui.selected_inv_slot = i;
            }
        }
        if game.ui.inv.open {
            for (i, rect) in game.ui.inv_rects.iter().enumerate() {
                if rect.contains_screen_pos(mp) {
                    match &mut game.inventory.grabbed {
                        Some(grabbed) => {
                            std::mem::swap(grabbed, &mut game.inventory.slots[i]);
                        }
                        None => {
                            let item =
                                std::mem::replace(&mut game.inventory.slots[i], ItemStack::empty());
                            game.inventory.grabbed = Some(item);
                        }
                    }
                }
            }
        }
    }
    msg
}

pub(crate) fn health_system(game: &mut GameState) {
    for (en, health) in game.ecw.query_mut::<&mut Health>() {
        if health.current < 1. {
            if en == game.player_en {
                game.player_en = hecs::Entity::DANGLING;
                game.respawn_timer = 500;
            }
            game.ecb.despawn(en);
        }
    }
    game.ecb.run_on(&mut game.ecw);
}

pub(crate) fn craft_ui_system(
    game: &mut GameState,
    egui_ctx: &egui::Context,
    atlas_size: ScreenRes,
) {
    if !game.ui.craft.open {
        return;
    }
    egui::Window::new("Crafting").show(egui_ctx, |ui| {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                for (rec_idx, recipe) in game.recipe_db.recipes.iter().enumerate() {
                    let item_id = recipe.output.id;

                    if let Some(out_def) = game.itemdb.get(item_id) {
                        ui.horizontal(|ui| {
                            ui.graphic_image(&out_def.tex_rect, atlas_size);
                            if ui
                                .selectable_label(
                                    game.ui.craft.selected_recipe == Some(rec_idx),
                                    &out_def.name,
                                )
                                .clicked()
                            {
                                game.ui.craft.selected_recipe = Some(rec_idx);
                            }
                        });
                    }
                }
            });
            ui.vertical(|ui| {
                let mut can_craft = true;
                if let &Some(rec_idx) = &game.ui.craft.selected_recipe {
                    let recipe = &game.recipe_db.recipes[rec_idx];
                    ui.heading("Requires");
                    for inp_stack in &recipe.input {
                        let Some(item_def) = game.itemdb.get(inp_stack.id) else {
                            ui.label("<invalid recipe>");
                            continue;
                        };
                        let need = inp_stack.qty;
                        let have = game.inventory.count_item(inp_stack.id);
                        if have == 0 || u64::from(need) > have {
                            can_craft = false;
                        }
                        ui.horizontal(|ui| {
                            ui.graphic_image(&item_def.tex_rect, atlas_size);
                            if need == 0 {
                                ui.label(&item_def.name);
                            } else {
                                ui.label(format!("{}: {have}/{need}", &item_def.name));
                            }
                        });
                    }
                    ui.heading("Crafted at");
                    for station in &recipe.stations {
                        let label = match station {
                            mdv_data::recipe::Station::Player => "anywhere",
                        };
                        ui.label(label);
                    }
                    if ui
                        .add_enabled(can_craft, egui::Button::new("Craft"))
                        .clicked()
                    {
                        for inp_stack in &recipe.input {
                            game.inventory.remove(inp_stack.id, inp_stack.qty);
                        }
                        game.inventory
                            .add(recipe.output.id, recipe.output.qty, &game.itemdb);
                    }
                }
            });
        });
    });
}

pub(crate) fn interact_system(
    game: &mut GameState,
    input: &Input,
    mouse_tpos: TilePos,
    mouse_wpos: WorldPos,
) {
    let Ok(mov) = game.ecw.query_one_mut::<&mut MovingEnt>(game.controlled_en) else {
        log::warn!("No controlled entity");
        return;
    };
    let player_pos = WorldPos::from_en(&mov.mob.en);
    let ptr_within_circle = mouse_wpos.within_circle(player_pos, game.tile_interact_radius);
    if input.pressed(InputAction::Interact) && ptr_within_circle {
        let tile = game.world.tile_at_mut(mouse_tpos);
        if !tile.mid.empty() {
            // Uproot uprootable tiles
            if game.tile_db[tile.mid].uprootable {
                process_tile_item_drop(&game.tile_db, &mut game.ecw, tile.mid, &mouse_tpos);
                tile.mid = TileId::EMPTY;
            }
        }
    }
}

struct TilePosSigned {
    x: i32,
    y: i32,
}

impl TilePos {
    #[expect(clippy::cast_possible_wrap, reason = "Positions can fit in i32")]
    fn to_signed(self) -> TilePosSigned {
        TilePosSigned {
            x: self.x as i32,
            y: self.y as i32,
        }
    }
}

/// Based on https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm#Rust
fn bresenham(p1: TilePosSigned, p2: TilePosSigned, mut f: impl FnMut(TilePos) -> ControlFlow<()>) {
    let dx: i32 = i32::abs(p2.x - p1.x);
    let dy: i32 = i32::abs(p2.y - p1.y);
    let sx: i32 = {
        if p1.x < p2.x {
            1
        } else {
            -1
        }
    };
    let sy: i32 = {
        if p1.y < p2.y {
            1
        } else {
            -1
        }
    };

    let mut error: i32 = (if dx > dy { dx } else { -dy }) / 2;
    let mut current_x: i32 = p1.x;
    let mut current_y: i32 = p1.y;
    #[expect(clippy::cast_sign_loss, reason = "Positions are always positive")]
    loop {
        if f(TilePos {
            x: current_x as u32,
            y: current_y as u32,
        })
        .is_break()
        {
            return;
        }

        if current_x == p2.x && current_y == p2.y {
            break;
        }

        let error2: i32 = error;

        if error2 > -dx {
            error -= dy;
            current_x += sx;
        }
        if error2 < dy {
            error += dx;
            current_y += sy;
        }
    }
}
