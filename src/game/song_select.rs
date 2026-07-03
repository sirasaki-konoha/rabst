//! Song select scene: lists discovered BMS files and lets the user pick one.

use bevy::prelude::*;

use crate::bms::scan_directory;
use crate::game::SelectedSong;
use crate::game::SongLibrary;
use crate::game::states::GameState;

/// Marker for all UI spawned by the song-select scene.
#[derive(Component)]
pub struct SongSelectUi;

/// Marker for the song-list text node.
#[derive(Component)]
pub struct SongListText;

pub fn enter(mut commands: Commands, mut library: ResMut<SongLibrary>) {
    if library.songs.is_empty() {
        let root = std::env::current_dir().unwrap_or_default();
        let songs_dir = root.join("songs");
        let scan_root = if songs_dir.is_dir() {
            songs_dir
        } else {
            root.clone()
        };
        info!("Scanning {:?} for BMS files...", scan_root);
        match scan_directory(&scan_root) {
            Ok(list) => {
                info!("Found {} BMS file(s).", list.len());
                library.songs = list;
            }
            Err(e) => error!("Failed to scan song directory: {e:?}"),
        }
    }

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            SongSelectUi,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("rabst — Song Select"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.95, 0.6)),
            ));
            p.spawn((
                Text::new("Up/Down: select   Enter: play   Esc: quit"),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    margin: UiRect::px(0.0, 0.0, 16.0, 16.0),
                    ..default()
                },
            ));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                SongListText,
            ));
        });
}

pub fn update(
    mut library: ResMut<SongLibrary>,
    mut selected: ResMut<SelectedSong>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Text, With<SongListText>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::ArrowDown) && !library.songs.is_empty() {
        library.selected_index = (library.selected_index + 1) % library.songs.len();
    }
    if keyboard.just_pressed(KeyCode::ArrowUp) && !library.songs.is_empty() {
        library.selected_index = if library.selected_index == 0 {
            library.songs.len() - 1
        } else {
            library.selected_index - 1
        };
    }
    if keyboard.just_pressed(KeyCode::Enter) {
        if let Some(entry) = library.selected() {
            selected.entry = Some(entry.clone());
            next_state.set(GameState::Loading);
            return;
        }
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }

    for mut text in &mut query {
        let mut buf = String::new();
        if library.songs.is_empty() {
            buf.push_str("No BMS files found. Put .bms files in a `songs/` folder.");
        } else {
            for (i, s) in library.songs.iter().enumerate() {
                let marker = if i == library.selected_index {
                    ">> "
                } else {
                    "   "
                };
                buf.push_str(&format!(
                    "{}[{:>2}] {} (LEVEL {}) - {}\n",
                    marker,
                    i + 1,
                    s.title,
                    s.level,
                    s.artist,
                ));
            }
        }
        text.0 = buf;
    }
}

pub fn exit(mut commands: Commands, query: Query<Entity, With<SongSelectUi>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}
