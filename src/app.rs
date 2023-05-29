use {
    crate::{
        command::CmdVec,
        config::Config,
        debug::{self, DebugState},
        game::{for_each_tile_on_screen, GameState},
        graphics::{self, ScreenSc, ScreenVec},
        input::Input,
        math::{center_offset, TILE_SIZE},
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
    /// RenderTexture for rendering the game at its native resolution
    pub rt: RenderTexture,
    /// Light map overlay, blended together with the non-lighted version of the scene
    pub light_map: RenderTexture,
    pub project_dirs: ProjectDirs,
    pub cmdvec: CmdVec,
    worlds_dir: std::path::PathBuf,
    pub snd: SoundPlayer,
    pub cfg: Config,
    pub on_screen_tile_ents: Vec<TileColEn>,
    /// Last computed mouse tile position
    pub last_mouse_tpos: TilePos,
    pub music_sink: rodio::Sink,
    pub stream: rodio::OutputStream,
    pub stream_handle: rodio::OutputStreamHandle,
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
        sink.append(Decoder::new(aud.sounds[name].clone()).unwrap());
        self.sounds.push_back(sink);
        // Limit max number of sounds
        if self.sounds.len() > 16 {
            self.sounds.pop_front();
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
        let light_map = RenderTexture::new(rw_size.x, rw_size.y)
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
        Ok(Self {
            rw,
            should_quit: false,
            game: GameState::new(wld_name.to_owned(), path, res),
            sf_egui,
            input: Input::default(),
            debug: DebugState::default(),
            scale: cfg.scale,
            rt,
            light_map,
            project_dirs,
            cmdvec: CmdVec::default(),
            worlds_dir,
            snd: SoundPlayer::new(stream_handle.clone()),
            cfg,
            on_screen_tile_ents: Default::default(),
            last_mouse_tpos: TilePos { x: 0, y: 0 },
            music_sink,
            stream,
            stream_handle,
        })
    }

    pub fn do_game_loop(mut self, res: &mut Res, aud: &ResAudio) {
        while !self.should_quit {
            self.do_event_handling();
            self.do_update(res, aud);
            self.do_rendering(res);
            self.input.clear_pressed();
            gamedebug_core::inc_frame();
        }
        self.game.tile_db.try_save("data");
        self.game.itemdb.try_save("data");
        self.game.world.save();
        std::fs::create_dir_all(self.project_dirs.config_dir()).unwrap();
        self.cfg.last_world = Some(self.game.world.name.clone());
        self.cfg.scale = self.scale;
        self.cfg.save(self.project_dirs.config_dir()).unwrap();
        let result = Save {
            inventory: self.game.inventory,
            world_seed: self.game.world.seed,
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
                    self.rt =
                        RenderTexture::new(width / self.scale as u32, height / self.scale as u32)
                            .unwrap();
                    self.light_map =
                        RenderTexture::new(width / self.scale as u32, height / self.scale as u32)
                            .unwrap();
                    let view = View::from_rect(Rect::new(0., 0., width as f32, height as f32));
                    self.rw.set_view(&view);
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

    fn do_update(&mut self, res: &mut Res, aud: &ResAudio) {
        let rt_size = self.rt.size();
        if self.debug.freecam {
            self.do_freecam();
        } else {
            let spd = if self.input.down(Key::LShift) {
                8.0
            } else if self.input.down(Key::LControl) {
                128.0
            } else {
                3.0
            };
            self.game.world.player.hspeed = 0.;
            if self.input.down(Key::A) {
                self.game.world.player.hspeed = -spd;
            }
            if self.input.down(Key::D) {
                self.game.world.player.hspeed = spd;
            }
            if self.input.down(Key::W) && self.game.world.player.can_jump() {
                self.game.world.player.vspeed = -10.0;
                self.game.world.player.jumps_left = 0;
            }
            self.game.world.player.down_intent = self.input.down(Key::S);
            let terminal_velocity = 60.0;
            self.game.world.player.vspeed = self
                .game
                .world
                .player
                .vspeed
                .clamp(-terminal_velocity, terminal_velocity);
            self.on_screen_tile_ents.clear();
            for_each_tile_on_screen(self.game.camera_offset, self.rt.size(), |tp, _sp| {
                let tile = self.game.world.tile_at_mut(tp).mid;
                if tile.empty() {
                    return;
                }
                let tdef = &self.game.tile_db[tile];
                let Some(bb) = tdef.layer.bb else {
                    return;
                };
                let x = tp.x as i32 * TILE_SIZE as i32;
                let y = tp.y as i32 * TILE_SIZE as i32;
                let en = s2dc::Entity::from_rect_corners(
                    x + bb.x as i32,
                    y + bb.y as i32,
                    x + bb.w as i32,
                    y + bb.h as i32,
                );
                self.on_screen_tile_ents.push(TileColEn {
                    col: en,
                    platform: tdef.layer.platform,
                });
            });
            imm_dbg!(self.on_screen_tile_ents.len());
            self.game.world.player.col_en.move_y(
                self.game.world.player.vspeed,
                |player_en, off| {
                    let mut col = false;
                    for en in &self.on_screen_tile_ents {
                        if player_en.would_collide(&en.col, off) {
                            if en.platform {
                                if self.game.world.player.vspeed < 0. {
                                    continue;
                                }
                                // If the player's feet are below the top of the platform,
                                // collision shouldn't happen
                                let player_feet = player_en.pos.y + player_en.bb.y;
                                if player_feet > en.col.pos.y || self.game.world.player.down_intent
                                {
                                    continue;
                                }
                            }
                            col = true;
                            if self.game.world.player.vspeed > 0. {
                                self.game.world.player.jumps_left = 1;
                            }
                            self.game.world.player.vspeed = 0.;
                        }
                    }
                    col
                },
            );
            self.game.world.player.col_en.move_x(
                self.game.world.player.hspeed,
                |player_en, off| {
                    let mut col = false;
                    for en in &self.on_screen_tile_ents {
                        if en.platform {
                            continue;
                        }
                        if player_en.would_collide(&en.col, off) {
                            col = true;
                            self.game.world.player.hspeed = 0.;
                        }
                    }
                    col
                },
            );
            self.game.world.player.vspeed += self.game.gravity;
            let (x, y, _w, _h) = self.game.world.player.col_en.en.xywh();
            self.game.camera_offset.x = (x - rt_size.x as i32 / 2).try_into().unwrap_or(0);
            self.game.camera_offset.y = (y - rt_size.y as i32 / 2).try_into().unwrap_or(0);
        }
        let mut loc = self.input.mouse_down_loc;
        let vco = viewport_center_offset(self.rw.size(), rt_size, self.scale);
        loc.x -= vco.x;
        loc.y -= vco.y;
        loc.x /= self.scale as ScreenSc;
        loc.y /= self.scale as ScreenSc;
        let mut wpos = self.game.camera_offset;
        wpos.x = wpos.x.saturating_add_signed(loc.x.into());
        wpos.y = wpos.y.saturating_add_signed(loc.y.into());
        let mouse_tpos = wpos.tile_pos();
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
        if self.debug.freecam && self.input.pressed(Key::P) {
            self.game.world.player.col_en.en.pos.x = wpos.x as i32;
            self.game.world.player.col_en.en.pos.y = wpos.y as i32;
        }
        self.game
            .item_use_system(&self.input, mouse_tpos, aud, &mut self.snd);
        self.game.biome_watch_system(&mut self.music_sink, res);
        self.game
            .update(&self.input, &mut self.snd, aud, &self.on_screen_tile_ents);
    }

    fn do_freecam(&mut self) {
        let spd = if self.input.down(Key::LShift) {
            100
        } else if self.input.down(Key::LControl) {
            1000
        } else {
            2
        };
        if self.input.down(Key::A) {
            self.game.camera_offset.x = self.game.camera_offset.x.saturating_sub(spd);
        }
        if self.input.down(Key::D) {
            self.game.camera_offset.x = self.game.camera_offset.x.saturating_add(spd);
        }
        if self.input.down(Key::W) {
            self.game.camera_offset.y = self.game.camera_offset.y.saturating_sub(spd);
        }
        if self.input.down(Key::S) {
            self.game.camera_offset.y = self.game.camera_offset.y.saturating_add(spd);
        }
    }

    fn do_rendering(&mut self, res: &mut Res) {
        self.game.light_pass(&mut self.light_map, res);
        self.rt.clear(Color::rgb(55, 221, 231));
        self.game.draw_world(&mut self.rt, res);
        self.game.draw_entities(&mut self.rt, res);
        self.rt.display();
        let mut spr = Sprite::with_texture(self.rt.texture());
        spr.set_scale((self.scale as f32, self.scale as f32));
        let vco = viewport_center_offset(self.rw.size(), self.rt.size(), self.scale);
        spr.set_position((vco.x as f32, vco.y as f32));
        self.rw.clear(Color::rgb(40, 10, 70));
        self.rw.draw(&spr);
        // Draw light overlay with multiply blending
        let mut rst = RenderStates::default();
        rst.blend_mode = BlendMode::MULTIPLY;
        self.light_map.display();
        spr.set_texture(self.light_map.texture(), false);
        self.rw.draw_with_renderstates(&spr, &rst);
        drop(spr);
        // Draw ui on top of in-game scene
        self.rt.clear(Color::TRANSPARENT);
        let ui_dims = Vector2 {
            x: (self.rw.size().x / self.scale as u32) as f32,
            y: (self.rw.size().y / self.scale as u32) as f32,
        };
        self.game.draw_ui(&mut self.rt, res, ui_dims);
        self.rt.display();
        let mut spr = Sprite::with_texture(self.rt.texture());
        spr.set_scale((self.scale as f32, self.scale as f32));
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
        drop(spr);
        self::command::dispatch(self, res);
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

fn viewport_center_offset(rw_size: Vector2u, rt_size: Vector2u, scale: u8) -> ScreenVec {
    let rw_size = rw_size;
    let rt_size = rt_size * scale as u32;
    let x = center_offset(rt_size.x as i32, rw_size.x as i32);
    let y = center_offset(rt_size.y as i32, rw_size.y as i32);
    ScreenVec {
        x: x as ScreenSc,
        y: y as ScreenSc,
    }
}
