//! Playing scene: spawns notes (as UI rectangles), drives audio/BGA
//! scheduling, handles key input and judgments, and transitions to the Score
//! scene on completion.

use bevy::audio::{AudioPlayer, PlaybackMode, PlaybackSettings, Volume};
use bevy::image::Image;
use bevy::prelude::*;
use bevy::time::Stopwatch;

use crate::channels::{
    CHANNEL_BGA_BASE, CHANNEL_BGA_LAYER, CHANNEL_BGM, CHANNEL_BPM_CHANGE, CHANNEL_BPM_CHANGE_EXT,
    CHANNEL_STOP, lane_for_channel,
};
use crate::game::loading::{AudioAssets, BgaAssets};
use crate::game::resources::{LoadedChartRes, PlayResult};
use crate::game::states::GameState;

// Playfield layout (in UI pixels; UI Y grows downward).
const PF_X: f32 = 420.0;
const PF_TOP: f32 = 30.0;
const PF_WIDTH: f32 = 448.0;
const PF_HEIGHT: f32 = 640.0;
const JUDGE_LINE_Y: f32 = 70.0; // distance from bottom of playfield
const NOTE_H: f32 = 16.0;
const HIGH_SPEED: f32 = 700.0; // pixels per second notes fall
const Z_PLAYFIELD: i32 = 0;
const Z_NOTES: i32 = 10;
const Z_JUDGE_LINE: i32 = 20;
const Z_HUD: i32 = 30;

const JUDGE_PERFECT: f64 = 0.045;
const JUDGE_GREAT: f64 = 0.090;
const JUDGE_GOOD: f64 = 0.135;
const JUDGE_BAD: f64 = 0.180;

/// Key bindings for each lane (Scratch, K1..K7).
const LANE_KEYS: [KeyCode; 8] = [
    KeyCode::ShiftLeft, // Scratch
    KeyCode::KeyS,      // K1
    KeyCode::KeyD,      // K2
    KeyCode::KeyF,      // K3
    KeyCode::Space,     // K4
    KeyCode::KeyJ,      // K5
    KeyCode::KeyK,      // K6
    KeyCode::KeyL,      // K7
];

#[derive(Component)]
pub struct PlayUi;

#[derive(Component)]
pub struct NoteNode {
    pub lane: usize,
    pub time: f64,
    pub obj: u32,
    pub judged: bool,
}

#[derive(Component)]
pub struct BgaDisplay;

#[derive(Component)]
pub struct ComboText;

#[derive(Component)]
pub struct JudgmentText;

#[derive(Resource)]
pub struct PlayClock {
    pub stopwatch: Stopwatch,
    pub start_delay: f64,
    pub triggered: std::collections::HashSet<(u32, u32)>,
}

impl Default for PlayClock {
    fn default() -> Self {
        Self {
            stopwatch: Stopwatch::new(),
            start_delay: 2.0,
            triggered: std::collections::HashSet::new(),
        }
    }
}

#[derive(Resource, Default)]
pub struct ComboState {
    pub combo: u32,
    pub max_combo: u32,
}

pub fn enter(
    mut commands: Commands,
    loaded: Res<LoadedChartRes>,
    mut play_result: ResMut<PlayResult>,
    mut play_clock: ResMut<PlayClock>,
    mut combo: ResMut<ComboState>,
) {
    play_clock.stopwatch.reset();
    play_clock.triggered.clear();
    combo.combo = 0;
    combo.max_combo = 0;
    *play_result = PlayResult::default();

    let chart = match &loaded.chart {
        Some(c) => c,
        None => {
            error!("Play entered without a loaded chart.");
            return;
        }
    };
    let bms = &chart.bms;
    let timing = &chart.timing;

    let notes_total = bms
        .notes
        .iter()
        .filter(|n| lane_for_channel(n.channel).is_some())
        .count() as u32;
    play_result.notes_total = notes_total;
    play_result.title = chart.song.title.clone();

    let lane_w = PF_WIDTH / 8.0;
    let judge_top = PF_TOP + PF_HEIGHT - JUDGE_LINE_Y;

    // Root UI container (absolute positioned children).
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            PlayUi,
        ))
        .with_children(|p| {
            // BGA display (top-left).
            p.spawn((
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(300.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(8.0),
                    top: Val::Px(8.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
                ImageNode::new(Handle::<Image>::default()),
                BgaDisplay,
            ));

            // Playfield background.
            p.spawn((
                Node {
                    width: Val::Px(PF_WIDTH),
                    height: Val::Px(PF_HEIGHT),
                    position_type: PositionType::Absolute,
                    left: Val::Px(PF_X),
                    top: Val::Px(PF_TOP),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.04, 0.04, 0.06)),
                BorderColor::all(Color::srgb(0.25, 0.25, 0.25)),
                GlobalZIndex(Z_PLAYFIELD),
            ));

            // Judge line.
            p.spawn((
                Node {
                    width: Val::Px(PF_WIDTH),
                    height: Val::Px(3.0),
                    position_type: PositionType::Absolute,
                    left: Val::Px(PF_X),
                    top: Val::Px(judge_top),
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 0.35, 0.35)),
                GlobalZIndex(Z_JUDGE_LINE),
            ));

            // HUD.
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(PF_X + PF_WIDTH + 24.0),
                    top: Val::Px(20.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                GlobalZIndex(Z_HUD),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Text::new(chart.song.title.clone()),
                    TextFont {
                        font_size: FontSize::Px(26.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                hud.spawn((
                    Text::new("Combo: 0"),
                    TextFont {
                        font_size: FontSize::Px(34.0),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ComboText,
                ));
                hud.spawn((
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Px(30.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    JudgmentText,
                ));
                hud.spawn((
                    Text::new("Keys: S D F Space J K L + LShift(scratch)\nEsc: abort"),
                    TextFont {
                        font_size: FontSize::Px(16.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });
        });

    // Spawn note nodes with absolute positioning.
    let lane_colors = [
        Color::srgb(0.7, 0.7, 0.7), // Scratch
        Color::srgb(0.95, 0.95, 0.95),
        Color::srgb(0.4, 0.6, 1.0),
        Color::srgb(0.95, 0.95, 0.95),
        Color::srgb(0.4, 0.6, 1.0),
        Color::srgb(0.95, 0.95, 0.95),
        Color::srgb(0.4, 0.6, 1.0),
        Color::srgb(0.95, 0.95, 0.95),
    ];

    for (i, n) in bms.notes.iter().enumerate() {
        let lane = match lane_for_channel(n.channel) {
            Some(l) => l,
            None => continue,
        };
        let t = timing.note_times[i];
        let x = PF_X + lane as f32 * lane_w + lane_w * 0.11;
        let w = lane_w * 0.78;
        let y = judge_top - (t as f32) * HIGH_SPEED;
        commands.spawn((
            Node {
                width: Val::Px(w),
                height: Val::Px(NOTE_H),
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                ..default()
            },
            BackgroundColor(lane_colors[lane]),
            GlobalZIndex(Z_NOTES),
            NoteNode {
                lane,
                time: t,
                obj: n.obj.0,
                judged: false,
            },
        ));
    }

    info!("Play scene entered: {} player notes", notes_total);
}

pub fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut play_clock: ResMut<PlayClock>,
    loaded: Res<LoadedChartRes>,
    audio_assets: Res<AudioAssets>,
    bga_assets: Res<BgaAssets>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut notes: Query<(Entity, &mut NoteNode, &mut Node)>,
    mut combo: ResMut<ComboState>,
    mut play_result: ResMut<PlayResult>,
    // Single query over both text markers to avoid B0001 conflicting &mut Text access.
    mut texts: Query<(&mut Text, Option<&ComboText>, Option<&JudgmentText>)>,
    mut bga_display: Query<&mut ImageNode, With<BgaDisplay>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let chart = match &loaded.chart {
        Some(c) => c,
        None => return,
    };
    let bms = &chart.bms;
    let timing = &chart.timing;

    play_clock.stopwatch.tick(time.delta());
    let song_t = play_clock.stopwatch.elapsed_secs_f64() - play_clock.start_delay;

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Score);
        return;
    }

    // Schedule BGM / BGA events.
    for (i, n) in bms.notes.iter().enumerate() {
        let t = timing.note_times[i];
        if t > song_t + 0.050 {
            break;
        }
        let key = (n.channel, n.obj.0);
        if play_clock.triggered.contains(&key) {
            continue;
        }
        match n.channel {
            CHANNEL_BGM => {
                if let Some(handle) = audio_assets.handles.get(&n.obj.0) {
                    spawn_audio(&mut commands, handle.clone());
                }
                play_clock.triggered.insert(key);
            }
            CHANNEL_BGA_BASE | CHANNEL_BGA_LAYER => {
                if let Some(handle) = bga_assets.handles.get(&n.obj.0) {
                    for mut img in bga_display.iter_mut() {
                        img.image = handle.clone();
                    }
                }
                play_clock.triggered.insert(key);
            }
            CHANNEL_BPM_CHANGE | CHANNEL_BPM_CHANGE_EXT | CHANNEL_STOP => {
                // baked into timing
                play_clock.triggered.insert(key);
            }
            _ => {}
        }
    }

    let judge_top = PF_TOP + PF_HEIGHT - JUDGE_LINE_Y;
    let lane_w = PF_WIDTH / 8.0;
    let mut remaining = 0u32;

    for (entity, mut note, mut node) in notes.iter_mut() {
        if note.judged {
            continue;
        }
        let dt = note.time - song_t;
        let y = judge_top - dt as f32 * HIGH_SPEED;
        node.top = Val::Px(y);
        node.left = Val::Px(PF_X + note.lane as f32 * lane_w + lane_w * 0.11);

        // Key press judgment.
        if keyboard.just_pressed(LANE_KEYS[note.lane]) {
            let abs_dt = dt.abs();
            if abs_dt <= JUDGE_BAD {
                let (label, hit) = if abs_dt <= JUDGE_PERFECT {
                    ("PERFECT", 0)
                } else if abs_dt <= JUDGE_GREAT {
                    ("GREAT", 1)
                } else if abs_dt <= JUDGE_GOOD {
                    ("GOOD", 2)
                } else {
                    ("BAD", 3)
                };
                apply_judgment(&mut combo, &mut play_result, hit);
                set_judgment(&mut texts, label);
                if let Some(handle) = audio_assets.handles.get(&note.obj) {
                    spawn_audio(&mut commands, handle.clone());
                }
                note.judged = true;
                commands.entity(entity).despawn();
                continue;
            }
        }

        // Auto-miss.
        if dt < -JUDGE_BAD {
            apply_judgment(&mut combo, &mut play_result, 4);
            set_judgment(&mut texts, "MISS");
            note.judged = true;
            commands.entity(entity).despawn();
            continue;
        }
        remaining += 1;
    }

    for (mut text, is_combo, _) in texts.iter_mut() {
        if is_combo.is_some() {
            text.0 = format!("Combo: {}", combo.combo);
        }
    }

    if remaining == 0 && song_t > timing.total_seconds {
        next_state.set(GameState::Score);
    }
}

fn spawn_audio(commands: &mut Commands, handle: Handle<bevy::audio::AudioSource>) {
    let settings = PlaybackSettings {
        mode: PlaybackMode::Despawn,
        volume: Volume::Linear(1.0),
        ..default()
    };
    commands.spawn((AudioPlayer(handle), settings));
}

fn apply_judgment(combo: &mut ComboState, result: &mut PlayResult, hit: u32) {
    match hit {
        0 => result.perfect += 1,
        1 => result.great += 1,
        2 => result.good += 1,
        3 => result.bad += 1,
        _ => result.poor += 1,
    }
    if hit <= 3 {
        combo.combo += 1;
        if combo.combo > combo.max_combo {
            combo.max_combo = combo.combo;
        }
    } else {
        combo.combo = 0;
    }
}

fn set_judgment(
    texts: &mut Query<(&mut Text, Option<&ComboText>, Option<&JudgmentText>)>,
    label: &str,
) {
    for (mut text, _, is_judgment) in texts.iter_mut() {
        if is_judgment.is_some() {
            text.0 = label.to_string();
        }
    }
}

pub fn exit(
    mut commands: Commands,
    query: Query<Entity, With<PlayUi>>,
    notes: Query<Entity, With<NoteNode>>,
) {
    for e in &query {
        commands.entity(e).despawn();
    }
    for e in &notes {
        commands.entity(e).despawn();
    }
}
