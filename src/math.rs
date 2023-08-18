use {
    crate::{
        graphics::ScreenRes,
        world::{TPosSc, TilePos, TPOS_SC_MAX},
    },
    extension_traits::extension,
    mdv_math::types::{IntRect, ScreenSc, ScreenVec},
    sfml::system::Vector2,
    std::{fmt::Debug, ops::Add},
};

#[extension(pub trait ScreenVecExt)]
impl ScreenVec {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "We expect that this function will be called only on resolutions,
                  which are small enough"
    )]
    fn from_sf_resolution(size: Vector2<u32>) -> Self {
        Self {
            x: size.x as ScreenSc,
            y: size.y as ScreenSc,
        }
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "We expect that this function will be called only on resolutions,
                  which are small enough"
    )]
    fn from_reso_i32(x: i32, y: i32) -> Self {
        Self {
            x: x as ScreenSc,
            y: y as ScreenSc,
        }
    }
    #[expect(
        clippy::cast_sign_loss,
        reason = "Assumed to be called on sizes, which are positive"
    )]
    fn size_to_sf_resolution(self) -> Vector2<u32> {
        Vector2 {
            x: self.x as u32,
            y: self.y as u32,
        }
    }

    fn div_by_scale(&self, scale: u8) -> Self {
        Self {
            x: self.x / ScreenSc::from(scale),
            y: self.y / ScreenSc::from(scale),
        }
    }
}

pub type WPosSc = u32;

/// World pixel position
///
/// Invariants:
///
/// - Not larger than 8 million (world boundary)
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldPos {
    pub x: WPosSc,
    pub y: WPosSc,
}

#[expect(
    clippy::cast_possible_wrap,
    reason = "World pos doesn't exceed i32::MAX"
)]
pub fn world_y_depth(y: WPosSc) -> i32 {
    y as i32 - WorldPos::SURFACE as i32
}

impl WorldPos {
    #[expect(
        clippy::cast_sign_loss,
        reason = "All entity coordinates are confined to be positive"
    )]
    pub fn from_en(en: &s2dc::Entity) -> Self {
        Self {
            x: en.pos.x as WPosSc,
            y: en.pos.y as WPosSc,
        }
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "After modulo with tile size, it fits into a screen scalar"
    )]
    pub(crate) fn tile_modulo(&self) -> ScreenVec {
        ScreenVec {
            x: (self.x % u32::from(TILE_SIZE)) as ScreenSc,
            y: (self.y % u32::from(TILE_SIZE)) as ScreenSc,
        }
    }
    #[expect(
        clippy::cast_possible_wrap,
        reason = "World pos doesn't exceed i32::MAX"
    )]
    pub fn to_signed(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }

    pub(crate) fn within_circle(&self, circle_pos: WorldPos, radius: u16) -> bool {
        let ((x, y), (cx, cy)) = (self.to_signed(), circle_pos.to_signed());
        mdv_math::util::point_within_circle(
            i64::from(cx),
            i64::from(cy),
            i64::from(radius),
            i64::from(x),
            i64::from(y),
        )
    }
    #[expect(
        clippy::cast_possible_wrap,
        reason = "Position is always smaller than i32::MAX"
    )]
    pub fn depth(&self) -> i32 {
        self.y as i32 - WorldPos::SURFACE as i32
    }
}

impl std::ops::Sub for WorldPos {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x = self.x.saturating_sub(rhs.x);
        self.y = self.y.saturating_sub(rhs.y);
        self
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
        let (x, y, w, h) = en.xywh_unsigned();
        Self {
            topleft: WorldPos { x, y },
            w,
            h,
        }
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        rect_overlap_xywh(
            self.topleft.x,
            self.topleft.y,
            self.w,
            self.h,
            other.topleft.x,
            other.topleft.y,
            other.w,
            other.h,
        )
    }
}

#[expect(clippy::too_many_arguments)]
pub fn rect_overlap_xywh<T: Add<Output = T> + Copy + PartialOrd>(
    x1: T,
    y1: T,
    w1: T,
    h1: T,
    x2: T,
    y2: T,
    w2: T,
    h2: T,
) -> bool {
    x1 + w1 > x2 && x1 < x2 + w2 && y1 + h1 > y2 && y1 < y2 + h2
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
pub const WORLD_EXTENT: TPosSc = 250_000;
#[expect(clippy::assertions_on_constants)]
const _: () = assert!(
    WORLD_EXTENT < TPOS_SC_MAX,
    "World extent can't be larger than maximum sound TPosSc value"
);
pub const WORLD_EXTENT_PX: WPosSc = WORLD_EXTENT * TILE_SIZE as WPosSc;

impl WorldPos {
    pub fn tile_pos(&self) -> TilePos {
        TilePos {
            x: wp_to_tp(self.x),
            y: wp_to_tp(self.y),
        }
    }
    /// Horizontal center of the world
    pub const CENTER: WPosSc = (WORLD_EXTENT / 2) * TILE_SIZE as WPosSc;
    pub const SURFACE: WPosSc = TilePos::SURFACE * TILE_SIZE as WPosSc;
    pub const SURFACE_CENTER: Self = Self {
        x: Self::CENTER,
        y: Self::SURFACE,
    };
    #[expect(
        clippy::cast_possible_wrap,
        reason = "World pos doesn't exceed i32::MAX"
    )]
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

#[extension(pub trait IntRectExt)]
impl IntRect {
    fn to_sf(self) -> sfml::graphics::Rect<i32> {
        sfml::graphics::Rect::<i32> {
            left: self.x,
            top: self.y,
            width: self.w,
            height: self.h,
        }
    }

    fn to_egui_uv(self, tex_size: ScreenRes) -> egui::Rect {
        let (sx, sy) = (f32::from(tex_size.w), f32::from(tex_size.h));
        egui::Rect::from_min_size(
            egui::pos2(self.x as f32 / sx, self.y as f32 / sy),
            egui::vec2(self.w as f32 / sx, self.h as f32 / sy),
        )
    }

    fn to_egui_size(self) -> egui::Vec2 {
        egui::vec2(self.w as f32, self.h as f32)
    }
}

#[test]
fn test_wp_to_tp() {
    assert_eq!(wp_to_tp(0), 0);
    assert_eq!(wp_to_tp(1), 0);
    assert_eq!(wp_to_tp(33), 1);
}

trait S2DcEntityExt {
    fn xywh_unsigned(&self) -> (u32, u32, u32, u32);
}

impl S2DcEntityExt for s2dc::Entity {
    #[expect(
        clippy::cast_sign_loss,
        reason = "Entity coordinates are assumed to be positive"
    )]
    fn xywh_unsigned(&self) -> (u32, u32, u32, u32) {
        let (x, y, w, h) = self.xywh();
        (x as u32, y as u32, w as u32, h as u32)
    }
}
