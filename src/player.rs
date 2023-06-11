use {
    crate::{math::WorldPos, save::Rgb, stringfmt::LengthDisp},
    s2dc::{vec2, MobileEntity},
    sfml::graphics::Color,
};

pub struct PlayerData {
    pub skin_color: Color,
    pub eye_color: Color,
    pub hair_color: Color,
    pub pants_color: Color,
    pub shirt_color: Color,
    pub shoes_color: Color,
}

pub struct MovingEnt {
    pub hspeed: f32,
    pub vspeed: f32,
    pub mob: MobileEntity,
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

pub struct IsPlayer;

#[derive(hecs::Bundle)]
pub struct PlayerBundle {
    pub mov: MovingEnt,
    pub mov_extra: MoveExtra,
    pub dat: PlayerData,
    pub _is: IsPlayer,
}

#[derive(hecs::Query)]
pub struct PlayerQuery<'a> {
    pub mov: &'a mut MovingEnt,
    pub mov_extra: &'a mut MoveExtra,
    pub dat: &'a mut PlayerData,
    _is: &'a IsPlayer,
}

impl PlayerBundle {
    pub fn new_at(pos: WorldPos) -> Self {
        Self {
            mov: MovingEnt::new(pos, vec2(20, 46)),
            mov_extra: MoveExtra::default(),
            dat: PlayerData::default(),
            _is: IsPlayer,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FacingDir {
    Left,
    Right,
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            skin_color: Color::rgb(249, 209, 151),
            eye_color: Color::WHITE,
            hair_color: Color::rgb(105, 203, 255),
            pants_color: Color::rgb(43, 85, 142),
            shirt_color: Color::rgb(170, 37, 7),
            shoes_color: Color::rgb(74, 44, 0),
        }
    }
}

impl PlayerData {
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
