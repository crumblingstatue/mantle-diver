use {
    crate::{math::WorldPos, save::Rgb, stringfmt::LengthDisp},
    s2dc::{vec2, MobileEntity},
    sfml::graphics::Color,
};

pub struct PlayerColors {
    pub skin: Color,
    pub eye: Color,
    pub hair: Color,
    pub pants: Color,
    pub shirt: Color,
    pub shoes: Color,
}

pub struct MovingEnt {
    pub hspeed: f32,
    pub vspeed: f32,
    pub mob: MobileEntity,
}

pub struct Health {
    pub current: f32,
    pub max: f32,
}

pub struct MoveExtra {
    pub jumps_left: u8,
    /// true if the player wants to jump down from a platform
    pub down_intent: bool,
    pub facing_dir: FacingDir,
}

impl Default for MoveExtra {
    fn default() -> Self {
        Self {
            jumps_left: Default::default(),
            down_intent: Default::default(),
            facing_dir: FacingDir::Right,
        }
    }
}

impl MoveExtra {
    pub fn can_jump(&self) -> bool {
        self.jumps_left > 0
    }
}

impl MovingEnt {
    fn new(pos: WorldPos, size: s2dc::Vec2) -> Self {
        Self {
            hspeed: 0.0,
            vspeed: 0.0,
            mob: MobileEntity::from_pos_and_bb(pos.to_s2dc(), size),
        }
    }
    pub fn feet_y(&self) -> i32 {
        self.mob.en.pos.y + self.mob.en.bb.y
    }
    pub fn depth_disp(&self) -> LengthDisp {
        LengthDisp(self.feet_y() as f32 - WorldPos::SURFACE as f32)
    }
}

#[derive(hecs::Bundle)]
pub struct PlayerBundle {
    pub mov: MovingEnt,
    pub mov_extra: MoveExtra,
    pub dat: PlayerColors,
    pub health: Health,
}

impl PlayerBundle {
    pub fn new_at(pos: WorldPos) -> Self {
        Self {
            mov: MovingEnt::new(pos, vec2(20, 46)),
            mov_extra: MoveExtra::default(),
            dat: PlayerColors::default(),
            health: Health {
                current: 100.,
                max: 100.,
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FacingDir {
    Left,
    Right,
}

impl Default for PlayerColors {
    fn default() -> Self {
        Self {
            skin: Color::rgb(249, 209, 151),
            eye: Color::WHITE,
            hair: Color::rgb(105, 203, 255),
            pants: Color::rgb(43, 85, 142),
            shirt: Color::rgb(170, 37, 7),
            shoes: Color::rgb(74, 44, 0),
        }
    }
}

impl PlayerColors {
    pub(crate) fn update_from_save(&mut self, sav: &crate::save::PlayerSav) {
        self.hair = sav.hair_color.to_sf();
        self.eye = sav.eye_color.to_sf();
        self.skin = sav.skin_color.to_sf();
        self.shirt = sav.shirt_color.to_sf();
        self.pants = sav.pants_color.to_sf();
        self.shoes = sav.shoes_color.to_sf();
    }

    pub(crate) fn sav(&self) -> crate::save::PlayerSav {
        crate::save::PlayerSav {
            skin_color: Rgb::from_sf(self.skin),
            eye_color: Rgb::from_sf(self.eye),
            hair_color: Rgb::from_sf(self.hair),
            shirt_color: Rgb::from_sf(self.shirt),
            pants_color: Rgb::from_sf(self.pants),
            shoes_color: Rgb::from_sf(self.shoes),
        }
    }
}
