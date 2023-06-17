use {
    super::GameState,
    crate::{audio::AudioCtx, player::Health, res::ResAudio},
};

pub enum Event {
    GroundHit { en: hecs::Entity, vspeed: f32 },
}

pub type EventBuf = Vec<Event>;

pub fn process_events(
    game: &mut GameState,
    buf: EventBuf,
    au_ctx: &mut AudioCtx,
    au_res: &ResAudio,
) {
    for event in buf {
        match event {
            Event::GroundHit { en, vspeed } => on_ground_hit(game, en, vspeed, au_ctx, au_res),
        }
    }
}

fn on_ground_hit(
    game: &mut GameState,
    en: hecs::Entity,
    vspeed: f32,
    au_ctx: &mut AudioCtx,
    au_res: &ResAudio,
) {
    if vspeed > 18. {
        if let Ok(health) = game.ecw.query_one_mut::<&mut Health>(en) {
            au_ctx.plr.play(au_res, "etc/ouch");
            health.current -= (vspeed - 18.) * 1.75;
        }
    }
}
