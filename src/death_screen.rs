use bevy::prelude::*;
use crate::{GameState, RunData};

pub struct DeathScreenPlugin;

impl Plugin for DeathScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameOver), setup_death_screen)
            .add_systems(
                Update,
                death_screen_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup_death_screen);
    }
}

#[derive(Component)]
struct DeathScreenUI;

fn setup_death_screen(mut commands: Commands, run: Res<RunData>) {
    let minutes = (run.time / 60.0) as i32;
    let seconds = (run.time % 60.0) as i32;

    // The playing camera was despawned by cleanup_run, so we need our own.
    commands.spawn((Camera2d, DeathScreenUI));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.03, 0.02, 0.94)),
            DeathScreenUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("YOU HAVE FALLEN"),
                TextFont { font_size: 38.0, ..default() },
                TextColor(Color::srgb(0.9, 0.25, 0.08)),
            ));

            parent.spawn(Node { height: Val::Px(24.0), ..default() });

            // Location
            parent.spawn((
                Text::new(format!(
                    "Floor {} - Room {}",
                    run.current_floor, run.current_room,
                )),
                TextFont { font_size: 22.0, ..default() },
                TextColor(Color::srgb(0.8, 0.65, 0.45)),
            ));

            // Time
            parent.spawn((
                Text::new(format!("Time Survived: {}:{:02}", minutes, seconds)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.65, 0.55, 0.4)),
            ));

            // Enemies
            parent.spawn((
                Text::new(format!("Enemies Defeated: {}", run.enemies_killed)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.65, 0.55, 0.4)),
            ));

            // Gold
            parent.spawn((
                Text::new(format!("Gold Earned: {}", run.gold)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            // Score
            parent.spawn((
                Text::new(format!("Score: {:06}", run.score)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));

            parent.spawn(Node { height: Val::Px(32.0), ..default() });

            // Prompt
            parent.spawn((
                Text::new("Press SPACE to return"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.5, 0.4, 0.3)),
            ));
        });
}

fn death_screen_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Title);
    }
}

fn cleanup_death_screen(
    mut commands: Commands,
    q: Query<Entity, With<DeathScreenUI>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}
