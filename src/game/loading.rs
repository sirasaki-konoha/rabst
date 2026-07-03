//! Loading scene: parses the selected BMS file fully, registers WAV/BMP
//! assets with Bevy's asset server, then transitions to Playing.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::asset::{AssetServer, Handle};
use bevy::audio::AudioSource;
use bevy::image::Image;
use bevy::prelude::*;

use crate::bms::{ChartTiming, parse_file};
use crate::game::SelectedSong;
use crate::game::resources::{LoadedChart, LoadedChartRes};
use crate::game::states::GameState;

const AUDIO_FALLBACK_EXTENSIONS: &[&str] = &["ogg", "wav", "mp3", "flac"];

#[derive(Component)]
pub struct LoadingUi;

/// Maps an object id to a loaded audio handle.
#[derive(Resource, Default)]
pub struct AudioAssets {
    pub handles: HashMap<u32, Handle<AudioSource>>,
}

/// Maps an object id to a loaded image handle (BGA).
#[derive(Resource, Default)]
pub struct BgaAssets {
    pub handles: HashMap<u32, Handle<Image>>,
    /// Default background image (BACKBMP / STAGEFILE).
    pub default: Option<Handle<Image>>,
}

pub fn enter(
    mut commands: Commands,
    selected: Res<SelectedSong>,
    asset_server: Res<AssetServer>,
    mut loaded: ResMut<LoadedChartRes>,
    mut audio_assets: ResMut<AudioAssets>,
    mut bga_assets: ResMut<BgaAssets>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Reset previous assets.
    audio_assets.handles.clear();
    bga_assets.handles.clear();
    bga_assets.default = None;

    let entry = match &selected.entry {
        Some(e) => e.clone(),
        None => {
            error!("No song selected; returning to song select.");
            next_state.set(GameState::SongSelect);
            return;
        }
    };

    let bms = match parse_file(&entry.path) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to parse {:?}: {e:?}", entry.path);
            next_state.set(GameState::SongSelect);
            return;
        }
    };

    // Load audio assets for every referenced #WAVxx. Many BMS packages keep
    // #WAV declarations even when the actual files are Ogg/Vorbis.
    for (&obj, rel) in &bms.wav_files {
        if let Some(path) = resolve_audio_path(&entry.dir, rel) {
            let handle = asset_server.load(path.to_string_lossy().to_string());
            audio_assets.handles.insert(obj.0, handle);
        } else {
            warn!("Missing audio: {:?}", entry.dir.join(rel));
        }
    }

    // Load BMP (BGA) assets.
    for (&obj, rel) in &bms.bmp_files {
        let path = entry.dir.join(rel);
        if path.exists() {
            let handle = asset_server.load(path.to_string_lossy().to_string());
            bga_assets.handles.insert(obj.0, handle);
        } else {
            warn!("Missing BMP: {:?}", path);
        }
    }
    // Default background.
    for rel in [&bms.backbmp, &bms.stagefile, &bms.banner] {
        if rel.is_empty() {
            continue;
        }
        let path = entry.dir.join(rel);
        if path.exists() {
            bga_assets.default = Some(asset_server.load(path.to_string_lossy().to_string()));
            break;
        }
    }

    let timing = ChartTiming::build(&bms);
    info!(
        "Loaded chart: {} ({} notes, {:.1}s)",
        entry.title,
        bms.notes.len(),
        timing.total_seconds
    );

    let wav_paths: HashMap<u32, PathBuf> = bms
        .wav_files
        .iter()
        .filter_map(|(&o, r)| resolve_audio_path(&entry.dir, r).map(|path| (o.0, path)))
        .collect();
    let bmp_paths: HashMap<u32, PathBuf> = bms
        .bmp_files
        .iter()
        .map(|(&o, r)| (o.0, entry.dir.join(r)))
        .collect();

    loaded.chart = Some(LoadedChart {
        song: entry,
        bms,
        timing,
        wav_paths,
        bmp_paths,
    });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            LoadingUi,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    // Transition to playing once assets begin loading; the actual readiness is
    // polled in `update`. We move forward when all audio handles are loaded.
    next_state.set(GameState::Playing);
}

pub fn update() {
    // Currently we transition immediately in `enter`. Asset loading proceeds
    // asynchronously during the Playing scene; notes that play before their
    // sample is ready simply get skipped. This keeps the experience responsive.
}

pub fn exit(mut commands: Commands, query: Query<Entity, With<LoadingUi>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}

fn resolve_audio_path(dir: &Path, rel: &str) -> Option<PathBuf> {
    let declared = dir.join(rel);
    if declared.exists() {
        return Some(declared);
    }

    for ext in AUDIO_FALLBACK_EXTENSIONS {
        let mut candidate = declared.clone();
        candidate.set_extension(ext);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}
