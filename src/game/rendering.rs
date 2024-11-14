pub mod debug;
pub mod game;
pub mod ui;

use sfml::{
    cpp::FBox,
    graphics::{RenderTexture, Vertex},
};

pub struct RenderState {
    /// Light map overlay, blended together with the non-lighted version of the scene
    pub light_blend_rt: FBox<RenderTexture>,
    pub vert_array: Vec<Vertex>,
    /// RenderTexture for rendering the game at its native resolution
    pub rt: FBox<RenderTexture>,
}
