use {super::GameState, crate::player::Health};

pub enum Event {
    GroundHit { en: hecs::Entity, vspeed: f32 },
}

pub type EventBuf = Vec<Event>;

pub fn process_events(game: &mut GameState, buf: EventBuf) {
    for event in buf {
        match event {
            Event::GroundHit { en, vspeed } => on_ground_hit(game, en, vspeed),
        }
    }
}

fn on_ground_hit(game: &mut GameState, en: hecs::Entity, vspeed: f32) {
    if vspeed > 16. {
        if let Ok(health) = game.ecw.query_one_mut::<&mut Health>(en) {
            health.current -= (vspeed - 16.) * 1.75;
        }
    }
}
