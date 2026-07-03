//! BMS (Be-Music Script) data model.

use std::collections::HashMap;

/// Base-36 (00-ZZ) object id used in channel data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjId(pub u32);

impl ObjId {
    pub const ZERO: ObjId = ObjId(0);

    pub fn from_hex36(s: &str) -> ObjId {
        let s = s.as_bytes();
        let mut v = 0u32;
        for &c in s {
            v *= 36;
            v += match c {
                b'0'..=b'9' => (c - b'0') as u32,
                b'a'..=b'z' => (c - b'a' + 10) as u32,
                b'A'..=b'Z' => (c - b'A' + 10) as u32,
                _ => 0,
            };
        }
        ObjId(v)
    }
}

/// A single object placed on a channel at a fractional measure position.
#[derive(Debug, Clone)]
pub struct BmsNote {
    pub channel: u32,
    /// Measure number (integer >= 0).
    pub measure: u32,
    /// Sub-position inside the measure, in 0..=1 absolute fraction.
    pub fraction: f64,
    pub obj: ObjId,
}

/// Parsed header + note chart.
#[derive(Debug, Clone, Default)]
pub struct BmsData {
    pub title: String,
    pub subtitle: String,
    pub artist: String,
    pub genre: String,
    pub player: u32,                       // #PLAYER
    pub play_level: String,                // #PLAYLEVEL
    pub rank: i32,                         // #RANK judgment difficulty
    pub total: f64,                        // #TOTAL gauge
    pub base_bpm: f64,                     // initial #BPM
    pub stagefile: String,                 // #STAGEFILE BGA banner
    pub banner: String,                    // #BANNER
    pub backbmp: String,                   // #BACKBMP
    pub wav_files: HashMap<ObjId, String>, // #WAVxx
    pub bmp_files: HashMap<ObjId, String>, // #BMPxx (bga images)
    pub bpm_changes: HashMap<ObjId, f64>,  // #BPMxx
    pub stop_changes: HashMap<ObjId, f64>, // #STOPxx (16th note stops)
    pub notes: Vec<BmsNote>,
}

impl BmsData {
    pub fn new() -> Self {
        Self {
            base_bpm: 130.0,
            total: 100.0,
            ..Default::default()
        }
    }
}
