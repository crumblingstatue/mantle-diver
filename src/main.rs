#![feature(lint_reasons)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use {
    app::App,
    clap::Parser,
    config::Config,
    directories::ProjectDirs,
    gamedebug_core::IMMEDIATE,
    res::{Res, ResAudio},
    std::backtrace::Backtrace,
};

mod app;
mod bitmanip;
mod char;
mod cmdline;
mod command;
mod config;
mod crafting;
mod debug;
mod game;
mod graphics;
mod input;
mod inventory;
mod itemdrop;
mod light;
mod math;
mod player;
mod res;
mod save;
mod stringfmt;
mod texture_atlas;
mod tiles;
mod time;
mod world;

#[derive(Parser)]
pub struct CliArgs {
    world_name: Option<String>,
    #[arg(long = "rand")]
    rand_world: bool,
    /// Show debug overlay
    #[arg(short, long)]
    debug: bool,
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
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let msg = if let Some(s) = payload.downcast_ref::<&str>() {
            s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s
        } else {
            "Unknown panic payload"
        };
        let (file, line, column) = match panic_info.location() {
            Some(loc) => (loc.file(), loc.line().to_string(), loc.column().to_string()),
            None => ("unknown", "unknown".into(), "unknown".into()),
        };
        let bt = Backtrace::capture();
        eprintln!("{msg}\n{file}:{line}:{column}\n{bt}");
        rfd::MessageDialog::new()
            .set_title("Mantle Diver panicked!")
            .set_description(&format!(
                "\
                {msg}\n\n\
                Location:\n\
                {file}:{line}:{column}\n\n\
                Backtrace: {bt}\n\n\
                Terminating."
            ))
            .set_level(rfd::MessageLevel::Error)
            .show();
    }));
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
