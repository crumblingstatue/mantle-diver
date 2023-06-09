use {
    crate::{command::Cmd, debug::DebugState, math::WorldPos},
    clap::Parser,
    splitty::SplitUnquotedChar,
};

#[derive(Parser)]
pub enum CmdLine {
    /// Quit the game
    Quit,
    /// Toggle free camera mode
    #[clap(alias = "fc")]
    Freecam,
    /// Clear the console log
    Clear,
    /// Teleport player to coordinates
    Tp(Tp),
    /// Teleport player to cursor
    Tpc,
    /// Teleport player back to spawn
    Spawn,
    /// Give the player an item
    Give(Give),
    /// Char db editor
    Chardb,
    /// Tile db editor
    Tiledb,
    /// Item db editor
    Itemdb,
    /// Set graphics scaling
    Scale(Scale),
    /// Open world management menu
    World(World),
    /// Toggle showing the texture atlas
    Atlas,
    /// Reload graphics
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
    Noop,
}

impl CmdLine {
    pub fn parse_cmdline(cmdline: &str) -> anyhow::Result<Self> {
        let words =
            std::iter::once(" ").chain(SplitUnquotedChar::new(cmdline, ' ').unwrap_quotes(true));
        Ok(Self::try_parse_from(words)?)
    }

    pub(crate) fn dispatch(self, debug: &mut DebugState) -> Dispatch {
        match self {
            CmdLine::Quit => Dispatch::Cmd(Cmd::QuitApp),
            CmdLine::Freecam => Dispatch::Cmd(Cmd::ToggleFreecam),
            CmdLine::Clear => Dispatch::ClearConsole,
            CmdLine::Tp(tp) => Dispatch::Cmd(Cmd::TeleportPlayer {
                pos: tp.to_world_pos(),
                relative: tp.rel,
            }),
            CmdLine::Tpc => Dispatch::Cmd(Cmd::TeleportPlayerCursor),
            CmdLine::Spawn => Dispatch::Cmd(Cmd::TeleportPlayerSpawn),
            CmdLine::Give(give) => Dispatch::Cmd(Cmd::GiveItemByName {
                name: give.name,
                amount: give.amount,
            }),
            CmdLine::Tiledb => Dispatch::Cmd(Cmd::ToggleTileDbEdit),
            CmdLine::Chardb => {
                debug.chardb_edit.open ^= true;
                Dispatch::Noop
            }
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
            CmdLine::Itemdb => {
                debug.itemdb_edit.open ^= true;
                Dispatch::Noop
            }
        }
    }
}
