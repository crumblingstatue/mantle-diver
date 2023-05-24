use {
    crate::{inventory::ItemId, math::WorldPos},
    egui_inspect::derive::Inspect,
    rand::{seq::SliceRandom, thread_rng, Rng},
    s2dc::Vec2,
};

#[derive(Inspect, Debug)]
pub struct Itemdrop {
    pub id: ItemId,
    #[opaque]
    pub s2dc_en: s2dc::MobileEntity,
    pub vspeed: f32,
    pub hspeed: f32,
}

impl Itemdrop {
    pub fn new_at(id: ItemId, pos: WorldPos) -> Self {
        let mut rng = thread_rng();
        let spd = rng.gen_range(0.5..2.4);
        Self {
            id,
            s2dc_en: s2dc::MobileEntity::from_pos_and_bb(
                Vec2 {
                    x: pos.x as i32,
                    y: pos.y as i32,
                },
                Vec2 { x: 4, y: 4 },
            ),
            vspeed: 0.0,
            hspeed: *[-spd, spd].choose(&mut rng).unwrap(),
        }
    }
}
