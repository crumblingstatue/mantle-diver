use {
    crate::math::WorldPos,
    mdv_data::tile::{BgTileId, MidTileId},
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
    MusVolInc,
    MusVolDec,
    GodToggle,
    SfxVolDec,
    SfxVolInc,
}

pub type CmdVec = Vec<Cmd>;
