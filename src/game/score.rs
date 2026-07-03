//! Score scene: shows the result of the last play and returns to song select.

use bevy::prelude::*;

use crate::game::resources::PlayResult;
use crate::game::states::GameState;

#[derive(Component)]
pub struct ScoreUi;

pub fn enter(mut commands: Commands, result: Res<PlayResult>) {
    let score = result.score();
    let grade = result.grade();
    info!(
        "Score scene: grade={} score={} perfect={} great={} good={} bad={} poor={} max_combo={}",
        grade,
        score,
        result.perfect,
        result.great,
        result.good,
        result.bad,
        result.poor,
        result.max_combo
    );

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            ScoreUi,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(format!("RESULT — {}", result.title)),
                TextFont { font_size: FontSize::Px(40.0), ..default() },
                TextColor(Color::srgb(1.0, 0.95, 0.6)),
            ));
            p.spawn((
                Text::new(format!("GRADE: {}   ({})", grade, score)),
                TextFont { font_size: FontSize::Px(56.0), ..default() },
                TextColor(Color::srgb(1.0, 0.8, 0.2)),
            ));
            p.spawn((
                Text::new(format!(
                    "PERFECT {:>4}\nGREAT   {:>4}\nGOOD    {:>4}\nBAD     {:>4}\nPOOR    {:>4}\n\nMAX COMBO {}",
                    result.perfect,
                    result.great,
                    result.good,
                    result.bad,
                    result.poor,
                    result.max_combo,
                )),
                TextFont { font_size: FontSize::Px(26.0), ..default() },
                TextColor(Color::WHITE),
            ));
            p.spawn((
                Text::new("Enter: back to song select   Esc: quit"),
                TextFont { font_size: FontSize::Px(18.0), ..default() },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

pub fn update(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
) {
    if keyboard.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::SongSelect);
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn exit(mut commands: Commands, query: Query<Entity, With<ScoreUi>>) {
    for e in &query {
        commands.entity(e).despawn();
    }
}
