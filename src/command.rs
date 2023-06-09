use crate::{
    math::WorldPos,
    tiles::{BgTileId, FgTileId, MidTileId},
};

/// A command that can change application or game state
pub enum Cmd {
    /// Quit the application
    QuitApp,
    ToggleFreecam,
    TeleportPlayer {
        pos: WorldPos,
        relative: bool,
    },
    TeleportPlayerSpawn,
    GiveItemByName {
        name: String,
        amount: u16,
    },
    ToggleTileDbEdit,
    SetScale(u8),
    LoadWorld(String),
    ReloadGraphics,
    SetBgTileAtCursor(BgTileId),
    SetMidTileAtCursor(MidTileId),
    SetFgTileAtCursor(FgTileId),
    TeleportPlayerCursor,
}

pub type CmdVec = Vec<Cmd>;
