use {
    crate::{
        data,
        game::GameState,
        graphics::ScreenRes,
        math::{WPosSc, TILE_SIZE},
        world::{TPosSc, TilePos},
    },
    fnv::FnvHashSet,
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

pub(crate) fn light_fill(light_state: &mut LightState, enum_info: LightEnumInfo) {
    let lightmap_size = enum_info.width as usize * enum_info.height as usize;
    light_state.light_map.resize(lightmap_size, 0);
    light_state.light_map.fill(0);
    let len = light_state.light_map.len();
    let stride = enum_info.width as usize;
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
            let Some(&val) = light_state.light_map.get(idx) else {
                log::error!(
                    "Index {idx} out of bounds (len: {})",
                    light_state.light_map.len()
                );
                return;
            };
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

#[derive(Clone, Copy)]
pub struct LightEnumInfo {
    /// Width of the enumerated area
    pub width: u16,
    /// Height of the enumerated area
    pub height: u16,
}

/// Gather up all the information on light sources that can have a visible effect on the screen.
///
/// This should fill up the `light_sources` array
pub(crate) fn enumerate_light_sources(
    game: &mut GameState,
    light_state: &mut LightState,
    rt_res: ScreenRes,
) -> LightEnumInfo {
    light_state.light_sources.clear();
    light_state.light_blockers.clear();
    let mut i = 0usize;
    // Define width and height
    let on_screen_w = WPosSc::from(rt_res.w) / WPosSc::from(TILE_SIZE);
    let on_screen_h = WPosSc::from(rt_res.h) / WPosSc::from(TILE_SIZE);
    let reach = TPosSc::from(MAX_TILE_REACH);
    let width = (reach * 2) + on_screen_w;
    let height = (reach * 2) + on_screen_h;
    // Start from current camera offset minus light reach
    let mut tp = game.camera_offset.tile_pos();
    tp.x -= reach;
    tp.y -= reach;
    let tp_x_init = tp.x;
    // Separate trackers... Sorry, I'm bad at math
    let mut x = 0;
    let mut y = 0;
    loop {
        let t = game.world.tile_at_mut(tp);
        let underground = tp.y > TilePos::SURFACE + 100;
        let empty = t.bg.empty() && t.mid.empty();
        let intensity = if empty {
            if underground {
                0
            } else {
                game.ambient_light
            }
        } else {
            255
        };
        let ls = t.mid == data::tile::mid::TILES_TORCH || empty;
        if ls {
            light_state.light_sources.push_back(LightSrc {
                map_idx: i,
                intensity,
            });
        }
        let lb = t.mid == data::tile::mid::TILES_DIRT || t.mid == data::tile::mid::TILES_STONE;
        if lb {
            light_state.light_blockers.insert(i);
        }
        i += 1;
        tp.x += 1;
        x += 1;
        if x >= width {
            tp.x = tp_x_init;
            x = 0;
            tp.y += 1;
            y += 1;
        }
        if y >= height {
            break;
        }
    }
    LightEnumInfo {
        width: width.try_into().unwrap(),
        height: height.try_into().unwrap(),
    }
}

/// Max light source reach in any on direction, in tiles.
///
/// When enumerating light sources, we need to consider light sources that are off-screen,
/// but still have an effect on the on-screen lighting.
///
/// This means we need to go through a much larger area than just the screen when enumerating.
/// How much larger? The answer to that is the maximum reach any light source can have in any
/// one direction. This is what this constant codifies.
pub const MAX_TILE_REACH: u8 = 10;
