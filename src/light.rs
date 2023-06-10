use {
    crate::{
        debug::{DbgOvr, DBG_OVR},
        game::{for_each_tile_on_screen, GameState},
        graphics::ScreenVec,
        math::{WorldRect, TILE_SIZE},
        tiles::MidTileId,
    },
    fnv::FnvHashSet,
    gamedebug_core::imm_dbg,
    sfml::{graphics::Color, system::Vector2u},
    std::collections::VecDeque,
};

pub struct LightSrc {
    pub map_idx: usize,
    pub intensity: u8,
}

pub struct LightState {
    pub light_map: Vec<u8>,
    pub light_sources: VecDeque<LightSrc>,
    pub light_blockers: FnvHashSet<usize>,
}

pub(crate) fn light_fill(light_state: &mut LightState, tiles_on_screen: U16Vec) {
    let stride = tiles_on_screen.x as usize;
    imm_dbg!(light_state.light_sources.len());
    imm_dbg!(light_state.light_blockers.len());
    light_state.light_map.fill(0);
    let len = light_state.light_map.len();
    // for each marked cell:
    while let Some(src) = light_state.light_sources.pop_front() {
        let fall_off = if light_state.light_blockers.contains(&src.map_idx) {
            80
        } else {
            20
        };
        // check each neighboring cell
        // if its 'brightness' is less than the current brightness minus some falloff value,
        // then update its brightness and mark it as well.
        // Left
        if src.map_idx > 0 {
            let idx = src.map_idx - 1;
            let val = light_state.light_map[idx];
            let new_intensity = src.intensity.saturating_sub(fall_off);
            if val < new_intensity {
                light_state.light_map[idx] = new_intensity;
                light_state.light_sources.push_back(LightSrc {
                    map_idx: idx,
                    intensity: new_intensity,
                });
            }
        }
        // Right
        if src.map_idx + 1 < len {
            let idx = src.map_idx + 1;
            let val = light_state.light_map[idx];
            let new_intensity = src.intensity.saturating_sub(fall_off);
            if val < new_intensity {
                light_state.light_map[idx] = new_intensity;
                light_state.light_sources.push_back(LightSrc {
                    map_idx: idx,
                    intensity: new_intensity,
                });
            }
        }
        // Up
        if src.map_idx > stride {
            let idx = src.map_idx - stride;
            let val = light_state.light_map[idx];
            let new_intensity = src.intensity.saturating_sub(fall_off);
            if val < new_intensity {
                light_state.light_map[idx] = new_intensity;
                light_state.light_sources.push_back(LightSrc {
                    map_idx: idx,
                    intensity: new_intensity,
                });
            }
        }
        // Down
        if src.map_idx + stride < len {
            let idx = src.map_idx + stride;
            let val = light_state.light_map[idx];
            let new_intensity = src.intensity.saturating_sub(fall_off);
            if val < new_intensity {
                light_state.light_map[idx] = new_intensity;
                light_state.light_sources.push_back(LightSrc {
                    map_idx: idx,
                    intensity: new_intensity,
                });
            }
        }
    }
    // continue until there are no more marked cells
}

#[derive(Default, Clone, Copy)]
pub struct U16Vec {
    pub x: u16,
    pub y: u16,
}

/// Gather up all the information on light sources that can have a visible effect on the screen.
///
/// This should fill up the `light_sources` array
pub(crate) fn enumerate_light_sources(
    game: &mut GameState,
    light_state: &mut LightState,
    rt_size: Vector2u,
    tiles_on_screen: U16Vec,
) {
    light_state.light_sources.clear();
    light_state.light_blockers.clear();
    let mut i = 0usize;
    for_each_tile_on_screen(
        game.camera_offset,
        ScreenVec::from_sf_resolution(rt_size),
        |tp, _sp| {
            let t = game.world.tile_at_mut(tp);
            let ls = t.mid == MidTileId::TORCH || (t.bg.empty() && t.mid.empty());
            if ls {
                if i > tiles_on_screen.x as usize {
                    let idx = i - tiles_on_screen.x as usize - 1;
                    light_state.light_sources.push_back(LightSrc {
                        map_idx: idx,
                        intensity: 255,
                    });
                }
                DBG_OVR.push(DbgOvr::WldRect {
                    r: WorldRect {
                        topleft: tp.to_world(),
                        w: TILE_SIZE.into(),
                        h: TILE_SIZE.into(),
                    },
                    c: Color::YELLOW,
                });
            }
            let lb = t.mid == MidTileId::DIRT || t.mid == MidTileId::STONE;
            if lb {
                if i > tiles_on_screen.x as usize {
                    let idx = i - tiles_on_screen.x as usize - 1;
                    light_state.light_blockers.insert(idx);
                }
                DBG_OVR.push(DbgOvr::WldRect {
                    r: WorldRect {
                        topleft: tp.to_world(),
                        w: TILE_SIZE.into(),
                        h: TILE_SIZE.into(),
                    },
                    c: Color::RED,
                });
            }
            i += 1;
        },
    );
}
