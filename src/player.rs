use {
    crate::{
        math::{WorldPos, TILE_SIZE},
        save::Rgb,
        stringfmt::LengthDisp,
        world::{TPosSc, TilePos},
    },
    s2dc::{vec2, MobileEntity},
    sfml::graphics::Color,
};

#[derive(Debug)]
pub struct Player {
    pub col_en: MobileEntity,
    pub vspeed: f32,
    pub hspeed: f32,
    pub jumps_left: u8,
    /// true if the player wants to jump down from a platform
    pub down_intent: bool,
    pub skin_color: Color,
    pub eye_color: Color,
    pub hair_color: Color,
    pub pants_color: Color,
    pub shirt_color: Color,
    pub shoes_color: Color,
    pub facing_dir: FacingDir,
}

#[derive(Debug, PartialEq)]
pub enum FacingDir {
    Left,
    Right,
}

impl Player {
    pub fn new_at(pos: WorldPos) -> Self {
        Self {
            col_en: MobileEntity::from_pos_and_bb(vec2(pos.x as i32, pos.y as i32), vec2(20, 46)),
            vspeed: 0.0,
            hspeed: 0.0,
            jumps_left: 0,
            down_intent: false,
            skin_color: Color::WHITE,
            eye_color: Color::WHITE,
            hair_color: Color::BLACK,
            pants_color: Color::WHITE,
            shirt_color: Color::WHITE,
            shoes_color: Color::WHITE,
            facing_dir: FacingDir::Right,
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

    pub(crate) fn update_from_save(&mut self, sav: &crate::save::PlayerSav) {
        self.hair_color = sav.hair_color.to_sf();
        self.eye_color = sav.eye_color.to_sf();
        self.skin_color = sav.skin_color.to_sf();
        self.shirt_color = sav.shirt_color.to_sf();
        self.pants_color = sav.pants_color.to_sf();
        self.shoes_color = sav.shoes_color.to_sf();
    }

    pub(crate) fn sav(&self) -> crate::save::PlayerSav {
        crate::save::PlayerSav {
            skin_color: Rgb::from_sf(self.skin_color),
            eye_color: Rgb::from_sf(self.eye_color),
            hair_color: Rgb::from_sf(self.hair_color),
            shirt_color: Rgb::from_sf(self.shirt_color),
            pants_color: Rgb::from_sf(self.pants_color),
            shoes_color: Rgb::from_sf(self.shoes_color),
        }
    }
}
