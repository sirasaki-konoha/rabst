//! Game scenes and shared resources.

pub mod loading;
pub mod play;
pub mod resources;
pub mod score;
pub mod song_select;
pub mod states;

pub use resources::SelectedSong;

use crate::bms::SongEntry;
use bevy::prelude::*;

/// In-memory library of discovered songs.
#[derive(Resource, Default)]
pub struct SongLibrary {
    pub songs: Vec<SongEntry>,
    pub selected_index: usize,
}

impl SongLibrary {
    pub fn selected(&self) -> Option<&SongEntry> {
        self.songs.get(self.selected_index)
    }
}
