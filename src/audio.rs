use {
    crate::res::ResAudio,
    rand::{thread_rng, Rng},
    rodio::{Decoder, OutputStreamHandle},
    std::collections::VecDeque,
};

pub struct AudioCtx {
    pub music_sink: rodio::Sink,
    pub stream: rodio::OutputStream,
    pub stream_handle: rodio::OutputStreamHandle,
    pub plr: SoundPlayer,
}

impl AudioCtx {
    pub fn new() -> Self {
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let plr = SoundPlayer::new(stream_handle.clone());
        Self {
            music_sink: rodio::Sink::try_new(&stream_handle).unwrap(),
            stream,
            stream_handle,
            plr,
        }
    }

    pub(crate) fn play_music(&self, data: &std::io::Cursor<Vec<u8>>) {
        if !self.music_sink.empty() {
            self.music_sink.clear();
        }
        self.music_sink
            .append(Decoder::new_looped(data.clone()).unwrap());
        self.music_sink.play();
    }
}

pub struct SoundPlayer {
    sounds: VecDeque<rodio::Sink>,
    stream_handle: OutputStreamHandle,
}

impl SoundPlayer {
    pub fn new(stream: OutputStreamHandle) -> Self {
        Self {
            sounds: Default::default(),
            stream_handle: stream,
        }
    }
    pub fn play(&mut self, aud: &ResAudio, name: &str) {
        let sink = rodio::Sink::try_new(&self.stream_handle).unwrap();
        let mut rng = thread_rng();
        sink.set_speed(rng.gen_range(0.94..=1.1));
        match aud.sounds.get(name) {
            Some(name) => {
                sink.append(Decoder::new(name.clone()).unwrap());
                self.sounds.push_back(sink);
                // Limit max number of sounds
                if self.sounds.len() > 16 {
                    self.sounds.pop_front();
                }
            }
            None => {
                log::error!("No such sound: {name}");
            }
        }
    }
}
