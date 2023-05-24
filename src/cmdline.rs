use {
    crate::{command::Cmd, math::WorldPos},
    clap::Parser,
    splitty::SplitUnquotedChar,
};

#[derive(Parser)]
pub enum CmdLine {
    Quit,
    Freecam,
    Clear,
    Tp(Tp),
    Spawn,
    Give(Give),
    /// Tile db editor
    Tiledb,
    /// Set scale
    Scale(Scale),
    World(World),
    Atlas,
    Greload,
}

#[derive(Parser)]
pub struct Tp {
    x: u32,
    y: u32,
    /// Relative to current position
    #[arg(short, long)]
    rel: bool,
}
impl Tp {
    fn to_world_pos(&self) -> WorldPos {
        WorldPos {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Parser)]
pub struct Give {
    name: String,
    #[arg(default_value_t = 1)]
    amount: u16,
}

#[derive(Parser)]
pub struct Scale {
    scale: u8,
}

#[derive(Parser)]
pub struct World {
    #[arg(default_value = "")]
    name: String,
}

pub enum Dispatch {
    Cmd(Cmd),
    ClearConsole,
    ToggleAtlas,
    ToggleWorldMgr,
}

impl CmdLine {
    pub fn parse_cmdline(cmdline: &str) -> anyhow::Result<Self> {
        let words =
            std::iter::once(" ").chain(SplitUnquotedChar::new(cmdline, ' ').unwrap_quotes(true));
        Ok(Self::try_parse_from(words)?)
    }

    pub(crate) fn dispatch(self) -> Dispatch {
        match self {
            CmdLine::Quit => Dispatch::Cmd(Cmd::QuitApp),
            CmdLine::Freecam => Dispatch::Cmd(Cmd::ToggleFreecam),
            CmdLine::Clear => Dispatch::ClearConsole,
            CmdLine::Tp(tp) => Dispatch::Cmd(Cmd::TeleportPlayer {
                pos: tp.to_world_pos(),
                relative: tp.rel,
            }),
            CmdLine::Spawn => Dispatch::Cmd(Cmd::TeleportPlayerSpawn),
            CmdLine::Give(give) => Dispatch::Cmd(Cmd::GiveItemByName {
                name: give.name,
                amount: give.amount,
            }),
            CmdLine::Tiledb => Dispatch::Cmd(Cmd::ToggleTileDbEdit),
            CmdLine::Scale(scale) => Dispatch::Cmd(Cmd::SetScale(scale.scale)),
            CmdLine::World(world) => {
                if world.name.is_empty() {
                    Dispatch::ToggleWorldMgr
                } else {
                    Dispatch::Cmd(Cmd::LoadWorld(world.name))
                }
            }
            CmdLine::Atlas => Dispatch::ToggleAtlas,
            CmdLine::Greload => Dispatch::Cmd(Cmd::ReloadGraphics),
        }
    }
}
