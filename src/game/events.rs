use {
    super::GameState,
    crate::{app::SoundPlayer, player::Health, res::ResAudio},
};

pub enum Event {
    GroundHit { en: hecs::Entity, vspeed: f32 },
}

pub type EventBuf = Vec<Event>;

pub fn process_events(game: &mut GameState, buf: EventBuf, snd: &mut SoundPlayer, aud: &ResAudio) {
    for event in buf {
        match event {
            Event::GroundHit { en, vspeed } => on_ground_hit(game, en, vspeed, snd, aud),
        }
    }
}

fn on_ground_hit(
    game: &mut GameState,
    en: hecs::Entity,
    vspeed: f32,
    snd: &mut SoundPlayer,
    aud: &ResAudio,
) {
    if vspeed > 18. {
        if let Ok(health) = game.ecw.query_one_mut::<&mut Health>(en) {
            snd.play(aud, "etc/ouch");
            health.current -= (vspeed - 18.) * 1.75;
        }
    }
}
