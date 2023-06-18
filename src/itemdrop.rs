use {
    crate::{math::WorldPos, player::MovingEnt},
    mdv_data::item::ItemId,
    rand::{seq::SliceRandom, thread_rng, Rng},
    s2dc::Vec2,
};

#[derive(hecs::Bundle)]
pub struct ItemdropBundle {
    pub id: ItemId,
    pub mov: MovingEnt,
}

impl ItemdropBundle {
    pub fn new_at(id: ItemId, pos: WorldPos) -> Self {
        let mut rng = thread_rng();
        let spd = rng.gen_range(0.5..2.4);
        Self {
            id,
            mov: MovingEnt {
                hspeed: *[-spd, spd].choose(&mut rng).unwrap(),
                vspeed: 0.0,
                mob: s2dc::MobileEntity::from_pos_and_bb(pos.to_s2dc(), Vec2 { x: 4, y: 4 }),
            },
        }
    }
}
