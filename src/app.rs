use {
    crate::{
        command::CmdVec,
        config::Config,
        debug::{self, DebugState, DBG_OVR},
        game::{rendering, rendering::RenderState, GameState},
        graphics::{self, ScreenSc, ScreenVec},
        input::Input,
        light::{self, LightState, U16Vec},
        math::{WPosSc, TILE_SIZE, WORLD_EXTENT_PX},
        player::PlayerQuery,
        res::{Res, ResAudio},
        save::Save,
        world::TilePos,
        CliArgs,
    },
    anyhow::Context,
    directories::ProjectDirs,
    egui_sfml::{SfEgui, UserTexSource},
    gamedebug_core::{imm, imm_dbg},
    rand::{thread_rng, Rng},
    rodio::{Decoder, OutputStreamHandle},
    sfml::{
        graphics::{
            BlendMode, Color, Rect, RectangleShape, RenderStates, RenderTarget, RenderTexture,
            RenderWindow, Shape, Sprite, Texture, Transformable, View,
        },
        system::{Vector2, Vector2u},
        window::{Event, Key},
    },
    std::collections::VecDeque,
};

mod command;

/// Application level state (includes game and ui state, etc.)
pub struct App {
    pub rw: RenderWindow,
    pub should_quit: bool,
    pub game: GameState,
    pub sf_egui: SfEgui,
    pub input: Input,
    pub debug: DebugState,
    /// Integer scale for rendering the game
    pub scale: u8,
    pub project_dirs: ProjectDirs,
    pub cmdvec: CmdVec,
    worlds_dir: std::path::PathBuf,
    pub snd: SoundPlayer,
    pub cfg: Config,
    /// Last computed mouse tile position
    pub last_mouse_tpos: TilePos,
    pub music_sink: rodio::Sink,
    pub stream: rodio::OutputStream,
    pub stream_handle: rodio::OutputStreamHandle,
    pub light_state: LightState,
    pub tiles_on_screen: U16Vec,
    pub render: RenderState,
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

impl App {
    pub fn new(
        args: CliArgs,
        res: &mut Res,
        cfg: Config,
        project_dirs: ProjectDirs,
    ) -> anyhow::Result<Self> {
        let rw = graphics::make_window();
        let sf_egui = SfEgui::new(&rw);
        let rw_size = rw.size();
        let rt =
            RenderTexture::new(rw_size.x, rw_size.y).context("Failed to create render texture")?;
        let light_blend_tex = RenderTexture::new(rw_size.x, rw_size.y)
            .context("Failed to create lightmap texture")?;
        let worlds_dir = project_dirs.data_dir().join("worlds");
        let rand_name;
        let wld_name = if args.rand_world {
            let n: u32 = thread_rng().gen();
            rand_name = n.to_string();
            &rand_name
        } else {
            args.world_name
                .as_deref()
                .unwrap_or(cfg.last_world.as_deref().unwrap_or("TestWorld"))
        };
        let path = worlds_dir.join(wld_name);
        let (stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let music_sink = rodio::Sink::try_new(&stream_handle).unwrap();
        music_sink.append(Decoder::new_looped(res.surf_music.clone()).unwrap());
        music_sink.play();
        let mut debug = DebugState::default();
        if args.debug {
            debug.dbg_overlay = true;
            DBG_OVR.set_enabled(true);
        }
        let mut this = Self {
            rw,
            should_quit: false,
            game: GameState::new(wld_name.to_owned(), path, res),
            sf_egui,
            input: Input::default(),
            debug,
            scale: cfg.scale,
            light_state: LightState {
                light_map: Vec::new(),
                light_sources: VecDeque::new(),
                light_blockers: fnv::FnvHashSet::default(),
            },
            project_dirs,
            cmdvec: CmdVec::default(),
            worlds_dir,
            snd: SoundPlayer::new(stream_handle.clone()),
            cfg,
            last_mouse_tpos: TilePos { x: 0, y: 0 },
            music_sink,
            stream,
            stream_handle,
            tiles_on_screen: U16Vec::default(),
            render: RenderState {
                light_blend_rt: light_blend_tex,
                rt,
            },
        };
        this.adapt_to_window_size_and_scale(ScreenVec::from_sf_resolution(rw_size));
        Ok(this)
    }

    pub fn do_game_loop(mut self, res: &mut Res, aud: &ResAudio) {
        while !self.should_quit {
            self.do_event_handling();
            self.do_update(res, aud);
            self.clamp_camera_offset();
            self.do_rendering(res);
            self.input.clear_pressed();
            gamedebug_core::inc_frame();
            imm_dbg!(DBG_OVR.len());
            if !self.game.paused {
                DBG_OVR.clear();
            }
        }
        self.game.tile_db.try_save("data");
        self.game.itemdb.try_save("data");
        self.game.char_db.save().unwrap();
        self.game.world.save();
        std::fs::create_dir_all(self.project_dirs.config_dir()).unwrap();
        self.cfg.last_world = Some(self.game.world.name.clone());
        self.cfg.scale = self.scale;
        self.cfg.save(self.project_dirs.config_dir()).unwrap();
        let (_en, plr) = self
            .game
            .ecw
            .query_mut::<PlayerQuery>()
            .into_iter()
            .next()
            .unwrap();
        let result = Save {
            inventory: self.game.inventory,
            world_seed: self.game.world.seed,
            player: plr.dat.sav(),
        }
        .save(&self.game.world.path);
        log::info!("Save result: {result:?}");
    }

    fn do_event_handling(&mut self) {
        while let Some(ev) = self.rw.poll_event() {
            self.sf_egui.add_event(&ev);
            {
                let ctx = self.sf_egui.context();
                self.input.update_from_event(
                    &ev,
                    ctx.wants_keyboard_input(),
                    ctx.wants_pointer_input(),
                );
            }
            match ev {
                Event::Closed => self.should_quit = true,
                Event::Resized { width, height } => {
                    self.adapt_to_window_size_and_scale(ScreenVec::from_sf_resolution(Vector2 {
                        x: width,
                        y: height,
                    }));
                }
                Event::KeyPressed { code, .. } => match code {
                    Key::F11 => {
                        self.debug.console.show ^= true;
                        self.debug.console.just_opened = true;
                    }
                    Key::F12 => self.debug.panel ^= true,
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn adapt_to_window_size_and_scale(&mut self, size: ScreenVec) {
        log::info!(
            "Adapting to window size: {width}x{height}, scale: {scale}",
            width = size.x,
            height = size.y,
            scale = self.scale
        );
        if self.scale == 0 {
            log::warn!("Adjusting bad 0 scale value to 1");
            self.scale = 1;
        }
        if self.scale > 5 {
            log::warn!("Clamping scale value to 5");
            self.scale = 5;
        }
        // Base size is the in-game surface size that can get scaled up to enlargen graphics.
        let base_size = size.div_by_scale(self.scale);
        let Vector2u { x: rt_w, y: rt_h } = base_size.size_to_sf_resolution();
        self.render.rt = RenderTexture::new(rt_w, rt_h).unwrap();
        self.render.light_blend_rt = RenderTexture::new(rt_w, rt_h).unwrap();
        // We add 2 to include partially visible tiles
        #[expect(clippy::cast_sign_loss, reason = "It's a size, not signed")]
        let tw = (base_size.x / ScreenSc::from(TILE_SIZE)) as u16 + 2;
        #[expect(clippy::cast_sign_loss, reason = "It's a size, not signed")]
        let th = (base_size.y / ScreenSc::from(TILE_SIZE)) as u16 + 2;
        self.tiles_on_screen.x = tw;
        self.tiles_on_screen.y = th;
        self.light_state.light_map = vec![0; tw as usize * th as usize];
        let view = View::from_rect(Rect::new(0., 0., f32::from(size.x), f32::from(size.y)));
        self.rw.set_view(&view);
    }

    fn do_update(&mut self, res: &mut Res, aud: &ResAudio) {
        let rt_size = self.render.rt.size();
        let mut mouse_world_pos = self.game.camera_offset;
        let mut loc = self.input.mouse_down_loc / ScreenSc::from(self.scale);
        mouse_world_pos.x = mouse_world_pos.x.saturating_add_signed(loc.x.into());
        mouse_world_pos.y = mouse_world_pos.y.saturating_add_signed(loc.y.into());
        loc.x /= ScreenSc::from(self.scale);
        loc.y /= ScreenSc::from(self.scale);
        let mouse_tpos = mouse_world_pos.tile_pos();
        self.last_mouse_tpos = mouse_tpos;
        imm!(
            "Mouse @ tile {}, {} ({:?})",
            mouse_tpos.x,
            mouse_tpos.y,
            self.game.world.tile_at_mut(mouse_tpos)
        );
        let m_chk = mouse_tpos.to_chunk();
        imm!("@ chunk {}, {}", m_chk.x, m_chk.y);
        let (m_chk_x, m_chk_y) = m_chk.region();
        imm!("@ region {m_chk_x}, {m_chk_y}");
        self.game.run_systems(
            &self.debug,
            &mut self.input,
            mouse_tpos,
            rt_size,
            &mut self.music_sink,
            res,
            &mut self.snd,
            aud,
            &mut self.cmdvec,
            &self.worlds_dir,
        );
        self::command::dispatch(self, res, mouse_world_pos);
    }

    fn do_rendering(&mut self, res: &mut Res) {
        light::enumerate_light_sources(
            &mut self.game,
            &mut self.light_state,
            self.render.rt.size(),
            self.tiles_on_screen,
        );
        light::light_fill(&mut self.light_state, self.tiles_on_screen);
        rendering::light_blend_pass(
            self.game.camera_offset,
            &mut self.render.light_blend_rt,
            &self.light_state.light_map,
            self.tiles_on_screen,
        );
        self.render.rt.clear(Color::rgb(55, 221, 231));
        rendering::draw_world(&mut self.game, &mut self.render.rt, res);
        rendering::draw_entities(&mut self.game, &mut self.render.rt, res, &self.debug);
        if self.debug.dbg_overlay {
            rendering::draw_debug_overlay(&mut self.render.rt, &mut self.game);
        }
        self.render.rt.display();
        let mut spr = Sprite::with_texture(self.render.rt.texture());
        spr.set_scale((f32::from(self.scale), f32::from(self.scale)));
        self.rw.clear(Color::rgb(40, 10, 70));
        self.rw.draw(&spr);
        // Draw light overlay with multiply blending
        let mut rst = RenderStates::default();
        rst.blend_mode = BlendMode::MULTIPLY;
        self.render.light_blend_rt.display();
        spr.set_texture(self.render.light_blend_rt.texture(), false);
        self.rw.draw_with_renderstates(&spr, &rst);
        drop(spr);
        // Draw ui on top of in-game scene
        self.render.rt.clear(Color::TRANSPARENT);
        let ui_dims = Vector2 {
            x: (self.rw.size().x / u32::from(self.scale)) as f32,
            y: (self.rw.size().y / u32::from(self.scale)) as f32,
        };
        rendering::draw_ui(&mut self.game, &mut self.render.rt, res, ui_dims);
        self.render.rt.display();
        let mut spr = Sprite::with_texture(self.render.rt.texture());
        spr.set_scale((f32::from(self.scale), f32::from(self.scale)));
        self.rw.draw(&spr);
        self.sf_egui
            .do_frame(|ctx| {
                debug::do_debug_ui(
                    ctx,
                    &mut self.debug,
                    &mut self.game,
                    res,
                    &mut self.cmdvec,
                    &self.worlds_dir,
                );
            })
            .unwrap();
        if self.debug.show_atlas {
            let atlas = &res.atlas.tex;
            let size = atlas.size();
            let mut rs = RectangleShape::from_rect(Rect::new(0., 0., size.x as f32, size.y as f32));
            rs.set_fill_color(Color::MAGENTA);
            self.rw.draw(&rs);
            self.rw.draw(&Sprite::with_texture(atlas));
        }
        let mut user_tex = EguiUserTex {
            atlas: &res.atlas.tex,
        };
        self.sf_egui.draw(&mut self.rw, Some(&mut user_tex));
        self.rw.display();
    }
    /// Before rendering, we clamp the camera offset to "sensible coordinates", to avoid having
    /// to deal with edge cases of the camera being near world boundaries.
    fn clamp_camera_offset(&mut self) {
        let ts = WPosSc::from(TILE_SIZE);
        // Let's leave a 100 tile buffer zone just to be safe
        // We don't need to worry about the max, `WORLD_EXTENT_PX` already accounts for a large
        // buffer zone
        let min_buffer_zone = ts * 100;
        self.game.camera_offset.x = self
            .game
            .camera_offset
            .x
            .clamp(min_buffer_zone, WORLD_EXTENT_PX);
        // At the bottom, we leave an extra 10000 space, so player can look at the unbreakable
        // bottom player
        self.game.camera_offset.y = self
            .game
            .camera_offset
            .y
            .clamp(min_buffer_zone, WORLD_EXTENT_PX + 10000);
    }
}

struct EguiUserTex<'a> {
    atlas: &'a Texture,
}

impl<'a> UserTexSource for EguiUserTex<'a> {
    fn get_texture(&mut self, _id: u64) -> (f32, f32, &'a Texture) {
        let size = self.atlas.size();
        (size.x as f32, size.y as f32, self.atlas)
    }
}

/// Tile collision entity for doing physics
pub struct TileColEn {
    pub col: s2dc::Entity,
    pub platform: bool,
}
