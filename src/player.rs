use {
    crate::{
        math::{WorldPos, TILE_SIZE},
        stringfmt::LengthDisp,
        world::{TPosSc, TilePos},
    },
    s2dc::{vec2, MobileEntity},
};

#[derive(Debug)]
pub struct Player {
    pub col_en: MobileEntity,
    pub vspeed: f32,
    pub hspeed: f32,
    pub jumps_left: u8,
    /// true if the player wants to jump down from a platform
    pub down_intent: bool,
}

impl Player {
    pub fn new_at(pos: WorldPos) -> Self {
        Self {
            col_en: MobileEntity::from_pos_and_bb(vec2(pos.x as i32, pos.y as i32), vec2(20, 46)),
            vspeed: 0.0,
            hspeed: 0.0,
            jumps_left: 0,
            down_intent: false,
        }
    }
    #[expect(dead_code)]
    pub fn center_tp(&self) -> TilePos {
        TilePos {
            x: (self.col_en.en.pos.x / TILE_SIZE as i32) as TPosSc,
            y: (self.col_en.en.pos.y / TILE_SIZE as i32) as TPosSc,
        }
    }
    pub fn can_jump(&self) -> bool {
        self.jumps_left > 0
    }
    pub fn feet_y(&self) -> i32 {
        self.col_en.en.pos.y + self.col_en.en.bb.y
    }
    pub fn depth_disp(&self) -> LengthDisp {
        LengthDisp(self.feet_y() as f32 - WorldPos::SURFACE as f32)
    }
}
