//! Resources shared between scenes: the selected song, the loaded chart, and
//! the final play result.

use std::path::PathBuf;

use bevy::prelude::*;

use crate::bms::{BmsData, ChartTiming, SongEntry};

/// The full set of data needed to play a song, produced by the Loading scene
/// and consumed by the Playing scene.
pub struct LoadedChart {
    pub song: SongEntry,
    pub bms: BmsData,
    pub timing: ChartTiming,
    /// Absolute paths of the WAV files indexed by object id.
    pub wav_paths: std::collections::HashMap<u32, PathBuf>,
    /// Absolute paths of the BMP (BGA image) files indexed by object id.
    pub bmp_paths: std::collections::HashMap<u32, PathBuf>,
}

/// Score result handed off from Playing -> Score scene.
#[derive(Resource, Default, Debug, Clone)]
pub struct PlayResult {
    pub title: String,
    pub perfect: u32,
    pub great: u32,
    pub good: u32,
    pub bad: u32,
    pub poor: u32,
    pub max_combo: u32,
    pub notes_total: u32,
}

impl PlayResult {
    pub fn score(&self) -> u32 {
        let total = self.notes_total.max(1) as f64;
        let pts =
            (self.perfect as f64 * 1.0 + self.great as f64 * 0.7 + self.good as f64 * 0.4) as f64;
        ((pts / total) * 1_000_000.0) as u32
    }

    pub fn grade(&self) -> &'static str {
        let s = self.score();
        match s {
            s if s >= 950_000 => "EX",
            s if s >= 900_000 => "S",
            s if s >= 800_000 => "A",
            s if s >= 700_000 => "B",
            s if s >= 600_000 => "C",
            s if s >= 500_000 => "D",
            _ => "E",
        }
    }
}

/// Selected song entry (SongSelect -> Loading).
#[derive(Resource, Default, Debug, Clone)]
pub struct SelectedSong {
    pub entry: Option<SongEntry>,
}

#[derive(Resource, Default)]
pub struct LoadedChartRes {
    pub chart: Option<LoadedChart>,
}
