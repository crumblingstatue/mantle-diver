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
    #[clap(alias = "tc")]
    Tpc,
    /// Teleport player back to spawn
    Spawn,
    /// Give the player an item
    Give {
        name: String,
        #[arg(default_value_t = 1)]
        amount: u16,
    },
    /// Hurt the controlled entity
    Hurt { amount: f32 },
    /// Char db editor
    Chardb,
    /// Tile db editor
    Tiledb,
    /// Item db editor
    Itemdb,
    /// Set graphics scaling
    Scale { scale: u8 },
    /// Open world management menu
    World {
        #[arg(default_value = "")]
        name: String,
    },
    /// Toggle showing the texture atlas
    Atlas,
    /// Reload graphics
    Greload,
    /// Toggle entity list
    Entlist,
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
            CmdLine::Tp(tp) => Dispatch::Cmd(Cmd::Teleport {
                pos: tp.to_world_pos(),
                relative: tp.rel,
            }),
            CmdLine::Tpc => Dispatch::Cmd(Cmd::TeleportCursor),
            CmdLine::Spawn => Dispatch::Cmd(Cmd::TeleportSpawn),
            CmdLine::Give { name, amount } => Dispatch::Cmd(Cmd::GiveItemByName { name, amount }),
            CmdLine::Hurt { amount } => Dispatch::Cmd(Cmd::HurtCtrlEn(amount)),
            CmdLine::Tiledb => Dispatch::Cmd(Cmd::ToggleTileDbEdit),
            CmdLine::Chardb => {
                debug.chardb_edit.open ^= true;
                Dispatch::Noop
            }
            CmdLine::Scale { scale } => Dispatch::Cmd(Cmd::SetScale(scale)),
            CmdLine::World { name } => {
                if name.is_empty() {
                    Dispatch::ToggleWorldMgr
                } else {
                    Dispatch::Cmd(Cmd::LoadWorld(name))
                }
            }
            CmdLine::Atlas => Dispatch::ToggleAtlas,
            CmdLine::Greload => Dispatch::Cmd(Cmd::ReloadGraphics),
            CmdLine::Itemdb => {
                debug.itemdb_edit.open ^= true;
                Dispatch::Noop
            }
            CmdLine::Entlist => {
                debug.entity_list.open ^= true;
                Dispatch::Noop
            }
        }
    }
}
