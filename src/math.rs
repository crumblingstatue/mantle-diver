use {
    crate::world::{TPosSc, TilePos},
    num_traits::{Num, Signed},
    serde::{Deserialize, Serialize},
    sfml::system::Vector2u,
    std::fmt::Debug,
};

pub type WPosSc = u32;

#[derive(Clone, Copy, Debug)]
pub struct WorldPos {
    pub x: WPosSc,
    pub y: WPosSc,
}

impl WorldPos {
    /// This is fine because all entity coordinates are confined to be positive
    pub fn from_en(en: &s2dc::Entity) -> Self {
        Self {
            x: en.pos.x as WPosSc,
            y: en.pos.y as WPosSc,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorldRect {
    pub topleft: WorldPos,
    pub w: WPosSc,
    pub h: WPosSc,
}
impl WorldRect {
    /// This is fine because all entity coordinates are confined to be positive
    pub(crate) fn from_s2dc_en(en: &s2dc::Entity) -> Self {
        let (x, y, w, h) = en.xywh();
        Self {
            topleft: WorldPos {
                x: x as u32,
                y: y as u32,
            },
            w: w as u32,
            h: h as u32,
        }
    }
}

/// Tile size in pixels
/// FIXME: While this is defined as a symbolic constant, the game can verily easily break
/// if this is not set to 32.
pub const TILE_SIZE: u8 = 32;
/// Pixels per meter.
pub const PX_PER_M: f32 = TILE_SIZE as f32 * 2.;
/// Meters per pixel
pub const M_PER_PX: f32 = 1. / PX_PER_M;
pub const FPS_TARGET: u8 = 60;

pub fn px_per_frame_to_m_per_s(px_per_frame: f32) -> f32 {
    let m_per_frame = px_per_frame / PX_PER_M;
    m_per_frame * f32::from(FPS_TARGET)
}

pub fn px_per_frame_to_km_h(px_per_frame: f32) -> f32 {
    px_per_frame_to_m_per_s(px_per_frame) * 3.6
}

/// World extent in tiles. Roughly 50km*50km.
pub const WORLD_EXTENT: TPosSc = 100_000;

impl WorldPos {
    pub fn tile_pos(&self) -> TilePos {
        TilePos {
            x: wp_to_tp(self.x),
            y: wp_to_tp(self.y),
        }
    }
    /// Horizontal center of the world
    pub const CENTER: WPosSc = (WORLD_EXTENT / 2) * TILE_SIZE as WPosSc;
    /// Vertical surface level.
    /// You can build 10 km high.
    pub const SURFACE: WPosSc = 20_000 * TILE_SIZE as WPosSc;
    pub const SURFACE_CENTER: Self = Self {
        x: Self::CENTER,
        y: Self::SURFACE,
    };

    pub(crate) fn to_s2dc(self) -> s2dc::Vec2 {
        s2dc::Vec2 {
            x: self.x as i32,
            y: self.y as i32,
        }
    }
}

pub fn wp_to_tp(wp: WPosSc) -> TPosSc {
    wp / WPosSc::from(TILE_SIZE)
}

// Get the offset required to center an object of `xw` width inside an object of `yw` width.
//
// For example, let's say `xw` (+) is 10 and we want to center it inside `yw` (-), which is 20
//
// ++++++++++           (x uncentered)
// -------------------- (y)
//      ++++++++++      (x centered)
//
// In this case, we needed to add 5 to x to achieve centering.
// This is the offset that this function calculates.
//
// We can calulate it by subtracting `xw` from `yw` (10), and dividing it by 2.
pub fn center_offset<N: From<u8> + Copy + Signed>(xw: N, yw: N) -> N {
    let diff = yw - xw;
    diff / N::from(2)
}

/// A smooth triangle-wave like transform of the input value, oscillating between 0 and the ceiling.
#[cfg_attr(not(test), expect(dead_code))]
pub fn smoothwave<T: Num + From<u8> + PartialOrd + Copy>(input: T, max: T) -> T {
    let period = max * T::from(2);
    let value = input % period;
    if value < max {
        value
    } else {
        period - value
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy)]
pub struct IntRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}
impl IntRect {
    pub(crate) fn to_sf(self) -> sfml::graphics::Rect<i32> {
        sfml::graphics::Rect::<i32> {
            left: self.x,
            top: self.y,
            width: self.w,
            height: self.h,
        }
    }

    pub(crate) fn to_egui_uv(self, tex_size: Vector2u) -> egui::Rect {
        let (sx, sy) = (tex_size.x as f32, tex_size.y as f32);
        egui::Rect::from_min_size(
            egui::pos2(self.x as f32 / sx, self.y as f32 / sy),
            egui::vec2(self.w as f32 / sx, self.h as f32 / sy),
        )
    }
}

pub fn step_towards(current: &mut f32, desired: f32, step: f32) {
    let diff = desired - *current; // Calculate the difference between current and desired values
    let direction = diff.signum(); // Get the sign of the difference to know which way to move

    if diff.abs() < step {
        // Check if the difference is less than the closeness threshold
        *current = desired; // If it is, set current to desired and return
        return;
    }

    let step = step * direction; // Multiply the step by the sign to get the correct direction
    *current += step; // Add the step to the current value
}

#[test]
fn test_smooth_wave() {
    assert_eq!(smoothwave(0, 100), 0);
    assert_eq!(smoothwave(50, 100), 50);
    assert_eq!(smoothwave(125, 100), 75);
    assert_eq!(smoothwave(150, 100), 50);
    assert_eq!(smoothwave(175, 100), 25);
    assert_eq!(smoothwave(199, 100), 1);
    assert_eq!(smoothwave(200, 100), 0);
    assert_eq!(smoothwave(201, 100), 1);
}

#[test]
fn test_wp_to_tp() {
    assert_eq!(wp_to_tp(0), 0);
    assert_eq!(wp_to_tp(1), 0);
    assert_eq!(wp_to_tp(33), 1);
}
