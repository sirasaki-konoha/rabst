//! Song list discovery: scan a directory tree for `.bms` files and collect
//! metadata used by the song-select scene.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::model::BmsData;
use super::parser;

#[derive(Debug, Clone)]
pub struct SongEntry {
    pub title: String,
    pub artist: String,
    pub genre: String,
    pub level: String,
    pub player: u32,
    pub path: PathBuf,
    pub dir: PathBuf,
    pub banner: Option<PathBuf>, // absolute path if available
}

pub fn scan_directory(root: &Path) -> Result<Vec<SongEntry>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        if !matches!(ext.as_str(), "bms" | "bme" | "bml" | "pms") {
            continue;
        }
        if let Ok(data) = load_bms_header(path) {
            let dir = entry.path().parent().unwrap_or(path).to_path_buf();
            out.push(SongEntry {
                title: if data.title.is_empty() {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    data.title
                },
                artist: data.artist,
                genre: data.genre,
                level: data.play_level,
                player: data.player,
                path: path.to_path_buf(),
                dir,
                banner: None,
            });
        }
    }
    out.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    Ok(out)
}

fn load_bms_header(path: &Path) -> Result<BmsData> {
    let text = std::fs::read_to_string(path).with_context(|| format!("read {:?}", path))?;
    parser::parse(&text)
}
