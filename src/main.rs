mod bms;
mod channels;
mod game;

use bevy::asset::{AssetPlugin, UnapprovedPathMode};
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy::winit::WinitSettings;

use game::states::GameState;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "rabst".to_string(),
                    resolution: WindowResolution::new(1280u32, 720u32),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            }),
    )
    .insert_resource(WinitSettings::game())
    .add_systems(Startup, setup_camera)
    // Shared resources.
    .init_resource::<game::resources::SelectedSong>()
    .init_resource::<game::resources::LoadedChartRes>()
    .init_resource::<game::resources::PlayResult>()
    .init_resource::<game::SongLibrary>()
    .init_resource::<game::loading::AudioAssets>()
    .init_resource::<game::loading::BgaAssets>()
    .init_resource::<game::play::PlayClock>()
    .init_resource::<game::play::ComboState>()
    // States.
    .init_state::<GameState>()
    .add_systems(OnEnter(GameState::SongSelect), game::song_select::enter)
    .add_systems(
        Update,
        game::song_select::update.run_if(in_state(GameState::SongSelect)),
    )
    .add_systems(OnExit(GameState::SongSelect), game::song_select::exit)
    .add_systems(OnEnter(GameState::Loading), game::loading::enter)
    .add_systems(
        Update,
        game::loading::update.run_if(in_state(GameState::Loading)),
    )
    .add_systems(OnExit(GameState::Loading), game::loading::exit)
    .add_systems(OnEnter(GameState::Playing), game::play::enter)
    .add_systems(
        Update,
        game::play::update.run_if(in_state(GameState::Playing)),
    )
    .add_systems(OnExit(GameState::Playing), game::play::exit)
    .add_systems(OnEnter(GameState::Score), game::score::enter)
    .add_systems(
        Update,
        game::score::update.run_if(in_state(GameState::Score)),
    )
    .add_systems(OnExit(GameState::Score), game::score::exit);

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
