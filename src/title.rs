use bevy::prelude::*;
use crate::{GameState, constants::*};

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), setup_title)
            .add_systems(OnExit(GameState::Title), cleanup_title)
            .add_systems(Update, handle_title_input.run_if(in_state(GameState::Title)));
    }
}

#[derive(Component)]
struct TitleEntity;

#[derive(Component)]
struct PromptText;

fn setup_title(mut commands: Commands) {
    commands.spawn((Camera2d, TitleEntity));

    // Dark forest background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.04, 0.06, 0.03),
            custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
        TitleEntity,
    ));

    // Stars
    let mut rng = rand::thread_rng();
    use rand::Rng;
    for _ in 0..40 {
        let x = rng.gen_range(-VIEWPORT_W / 2.0..VIEWPORT_W / 2.0);
        let y = rng.gen_range(50.0..VIEWPORT_H / 2.0);
        let b = rng.gen_range(0.3..0.8_f32);
        let sz = rng.gen_range(1.0..2.5_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(b, b, b * 0.9, b),
                custom_size: Some(Vec2::new(sz, sz)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 1.0),
            TitleEntity,
        ));
    }

    // Ground
    let ground_y = -VIEWPORT_H / 2.0 + 60.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.1, 0.06),
            custom_size: Some(Vec2::new(VIEWPORT_W, 120.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y, Z_BACKGROUND + 2.0),
        TitleEntity,
    ));

    // Grass line
    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.25, 0.1),
            custom_size: Some(Vec2::new(VIEWPORT_W, 4.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y + 62.0, Z_BACKGROUND + 3.0),
        TitleEntity,
    ));

    // Cave entrance (dark arch)
    let cave_y = ground_y + 62.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.02, 0.015, 0.03),
            custom_size: Some(Vec2::new(80.0, 60.0)),
            ..default()
        },
        Transform::from_xyz(0.0, cave_y + 20.0, Z_BACKGROUND + 4.0),
        TitleEntity,
    ));

    // Stone arch left
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.25, 0.2),
            custom_size: Some(Vec2::new(14.0, 70.0)),
            ..default()
        },
        Transform::from_xyz(-42.0, cave_y + 25.0, Z_BACKGROUND + 5.0),
        TitleEntity,
    ));

    // Stone arch right
    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.25, 0.2),
            custom_size: Some(Vec2::new(14.0, 70.0)),
            ..default()
        },
        Transform::from_xyz(42.0, cave_y + 25.0, Z_BACKGROUND + 5.0),
        TitleEntity,
    ));

    // Arch top
    commands.spawn((
        Sprite {
            color: Color::srgb(0.35, 0.28, 0.22),
            custom_size: Some(Vec2::new(98.0, 12.0)),
            ..default()
        },
        Transform::from_xyz(0.0, cave_y + 58.0, Z_BACKGROUND + 5.0),
        TitleEntity,
    ));

    // Trees (silhouettes)
    for x_off in [-200.0_f32, -140.0, 150.0, 210.0] {
        let h = rng.gen_range(80.0..140.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.06, 0.08, 0.04),
                custom_size: Some(Vec2::new(20.0, h)),
                ..default()
            },
            Transform::from_xyz(x_off, cave_y + h / 2.0 - 10.0, Z_BACKGROUND + 1.5),
            TitleEntity,
        ));
        // Canopy
        commands.spawn((
            Sprite {
                color: Color::srgb(0.08, 0.12, 0.06),
                custom_size: Some(Vec2::new(50.0, 40.0)),
                ..default()
            },
            Transform::from_xyz(x_off, cave_y + h - 10.0, Z_BACKGROUND + 1.5),
            TitleEntity,
        ));
    }

    // Title
    commands.spawn((
        Text2d::new("DAWNROOT"),
        TextFont { font_size: 56.0, ..default() },
        TextColor(Color::srgb(0.85, 0.75, 0.5)),
        Transform::from_xyz(0.0, 140.0, Z_HUD),
        TitleEntity,
    ));

    // Subtitle
    commands.spawn((
        Text2d::new("Roguelike Platformer"),
        TextFont { font_size: 18.0, ..default() },
        TextColor(Color::srgb(0.6, 0.55, 0.4)),
        Transform::from_xyz(0.0, 100.0, Z_HUD),
        TitleEntity,
    ));

    // Controls hint
    commands.spawn((
        Text2d::new("A/D: Move  |  Space: Jump  |  J: Attack  |  Shift: Dash  |  1-4: Spells"),
        TextFont { font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.5, 0.45, 0.4)),
        Transform::from_xyz(0.0, -180.0, Z_HUD),
        TitleEntity,
    ));

    // Prompt
    commands.spawn((
        Text2d::new("- Press SPACE to enter -"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgba(0.9, 0.8, 0.5, 1.0)),
        Transform::from_xyz(0.0, -220.0, Z_HUD),
        TitleEntity,
        PromptText,
    ));
}

fn cleanup_title(mut commands: Commands, q: Query<Entity, With<TitleEntity>>) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut prompt_q: Query<&mut TextColor, With<PromptText>>,
    time: Res<Time>,
) {
    if let Ok(mut color) = prompt_q.get_single_mut() {
        let alpha = 0.5 + 0.5 * (time.elapsed_secs() * 2.0).sin();
        color.0 = Color::srgba(0.9, 0.8, 0.5, alpha);
    }

    if keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
