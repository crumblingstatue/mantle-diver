pub mod debug;
pub mod game;
pub mod ui;

use sfml::{
    graphics::{RenderTexture, Vertex},
    SfBox,
};

pub struct RenderState {
    /// Light map overlay, blended together with the non-lighted version of the scene
    pub light_blend_rt: SfBox<RenderTexture>,
    pub vert_array: Vec<Vertex>,
    /// RenderTexture for rendering the game at its native resolution
    pub rt: SfBox<RenderTexture>,
}
