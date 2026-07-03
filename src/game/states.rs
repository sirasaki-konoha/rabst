//! Global app states (scenes).

use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    SongSelect,
    Loading,
    Playing,
    Score,
}
