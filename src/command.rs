use crate::{
    math::WorldPos,
    tiles::{BgTileId, MidTileId},
};

/// A command that can change application or game state
pub enum Cmd {
    /// Quit the application
    QuitApp,
    ToggleFreecam,
    Teleport {
        pos: WorldPos,
        relative: bool,
    },
    TeleportSpawn,
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
    TeleportCursor,
    HurtCtrlEn(f32),
}

pub type CmdVec = Vec<Cmd>;
