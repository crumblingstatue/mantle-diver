#![feature(lint_reasons)]

use {
    app::App,
    clap::Parser,
    config::Config,
    directories::ProjectDirs,
    gamedebug_core::IMMEDIATE,
    res::{Res, ResAudio},
};

mod app;
mod bitmanip;
mod cmdline;
mod command;
mod config;
mod debug;
mod game;
mod graphics;
mod input;
mod inventory;
mod itemdrop;
mod math;
mod player;
mod res;
mod save;
mod stringfmt;
mod texture_atlas;
mod tiles;
mod world;

#[derive(Parser)]
pub struct CliArgs {
    world_name: Option<String>,
    #[arg(long = "rand")]
    rand_world: bool,
}

fn try_main() -> anyhow::Result<()> {
    IMMEDIATE.set_enabled(true);
    let cli_args = CliArgs::parse();
    let project_dirs = ProjectDirs::from("", "", "mantle-diver").unwrap();
    let cfg = Config::load(project_dirs.config_dir())?;
    let mut res = Res::load(&cfg.res_folder_path)?;
    let aud = ResAudio::load(&cfg.res_folder_path)?;
    let app = App::new(cli_args, &mut res, cfg, project_dirs)?;
    app.do_game_loop(&mut res, &aud);
    Ok(())
}

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();
    if let Err(e) = try_main() {
        rfd::MessageDialog::new()
            .set_title("Fatal error")
            .set_description(&e.to_string())
            .show();
    }
}
