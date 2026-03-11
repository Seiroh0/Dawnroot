use bevy::prelude::*;
use crate::{GameState, constants::*};

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), setup_title)
            .add_systems(OnExit(GameState::Title), cleanup_title)
            .add_systems(Update, handle_title_input.run_if(in_state(GameState::Title)))
            .add_systems(OnEnter(GameState::WellIntro), setup_well_intro)
            .add_systems(OnExit(GameState::WellIntro), cleanup_well_intro)
            .add_systems(
                Update,
                (update_well_intro, update_falling_particles)
                    .run_if(in_state(GameState::WellIntro)),
            );
    }
}

#[derive(Component)]
struct TitleEntity;

#[derive(Component)]
struct PromptText;

// ── Title screen ──────────────────────────────────────────────────

fn setup_title(mut commands: Commands) {
    commands.spawn((Camera2d, TitleEntity));

    let mut rng = rand::thread_rng();
    use rand::Rng;

    // Night sky
    commands.spawn((
        Sprite {
            color: Color::srgb(0.02, 0.03, 0.06),
            custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
        TitleEntity,
    ));

    // Stars
    for _ in 0..60 {
        let x = rng.gen_range(-VIEWPORT_W / 2.0..VIEWPORT_W / 2.0);
        let y = rng.gen_range(20.0..VIEWPORT_H / 2.0);
        let b = rng.gen_range(0.2..0.9_f32);
        let sz = rng.gen_range(1.0..3.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(b, b, b * 0.95, b),
                custom_size: Some(Vec2::new(sz, sz)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 0.5),
            TitleEntity,
        ));
    }

    // Moon
    commands.spawn((
        Sprite {
            color: Color::srgba(0.95, 0.92, 0.8, 0.9),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(280.0, 180.0, Z_BACKGROUND + 0.8),
        TitleEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.02, 0.03, 0.06),
            custom_size: Some(Vec2::new(34.0, 34.0)),
            ..default()
        },
        Transform::from_xyz(288.0, 184.0, Z_BACKGROUND + 0.9),
        TitleEntity,
    ));

    // Far hills
    for &(x, w, h, g) in &[
        (-200.0, 300.0, 80.0, 0.04_f32),
        (100.0, 350.0, 100.0, 0.035),
        (350.0, 280.0, 70.0, 0.045),
    ] {
        commands.spawn((
            Sprite {
                color: Color::srgb(g, g + 0.01, g * 0.8),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x, -VIEWPORT_H / 2.0 + 60.0 + h / 2.0 - 20.0, Z_BACKGROUND + 1.0),
            TitleEntity,
        ));
    }

    // Ground
    let ground_y = -VIEWPORT_H / 2.0 + 50.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.08, 0.1, 0.05),
            custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y, Z_BACKGROUND + 2.0),
        TitleEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.22, 0.08),
            custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 5.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y + 52.5, Z_BACKGROUND + 3.0),
        TitleEntity,
    ));

    // ── Well (Brunnen) ──
    let well_base_y = ground_y + 52.5;
    let well_x = 0.0;
    // Base
    commands.spawn((
        Sprite { color: Color::srgb(0.35, 0.3, 0.25), custom_size: Some(Vec2::new(64.0, 28.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 14.0, Z_BACKGROUND + 5.0), TitleEntity,
    ));
    // Inner darkness
    commands.spawn((
        Sprite { color: Color::srgb(0.01, 0.01, 0.02), custom_size: Some(Vec2::new(48.0, 20.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 14.0, Z_BACKGROUND + 5.5), TitleEntity,
    ));
    // Rim
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.35, 0.28), custom_size: Some(Vec2::new(70.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 30.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    // Pillars
    commands.spawn((
        Sprite { color: Color::srgb(0.32, 0.27, 0.2), custom_size: Some(Vec2::new(8.0, 60.0)), ..default() },
        Transform::from_xyz(well_x - 30.0, well_base_y + 60.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.32, 0.27, 0.2), custom_size: Some(Vec2::new(8.0, 60.0)), ..default() },
        Transform::from_xyz(well_x + 30.0, well_base_y + 60.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    // Roof
    commands.spawn((
        Sprite { color: Color::srgb(0.28, 0.2, 0.14), custom_size: Some(Vec2::new(80.0, 10.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 90.0, Z_BACKGROUND + 6.5), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.3, 0.22, 0.15), custom_size: Some(Vec2::new(50.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 98.0, Z_BACKGROUND + 6.5), TitleEntity,
    ));
    // Rope + bucket
    commands.spawn((
        Sprite { color: Color::srgb(0.5, 0.4, 0.25), custom_size: Some(Vec2::new(2.0, 40.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 50.0, Z_BACKGROUND + 5.8), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.3, 0.18), custom_size: Some(Vec2::new(10.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 32.0, Z_BACKGROUND + 5.8), TitleEntity,
    ));

    // Trees
    for &(x_off, h_min, h_max) in &[
        (-320.0_f32, 100.0, 150.0),
        (-240.0, 80.0, 130.0),
        (-160.0, 120.0, 160.0),
        (160.0, 110.0, 155.0),
        (250.0, 90.0, 140.0),
        (340.0, 100.0, 150.0),
    ] {
        let h: f32 = rng.gen_range(h_min..h_max);
        let trunk_w: f32 = rng.gen_range(10.0..18.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.04, 0.06, 0.03),
                custom_size: Some(Vec2::new(trunk_w, h)),
                ..default()
            },
            Transform::from_xyz(x_off, well_base_y + h / 2.0, Z_BACKGROUND + 1.5),
            TitleEntity,
        ));
        let canopy_w: f32 = rng.gen_range(40.0..65.0);
        let canopy_h: f32 = rng.gen_range(35.0..55.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.05, 0.09, 0.04),
                custom_size: Some(Vec2::new(canopy_w, canopy_h)),
                ..default()
            },
            Transform::from_xyz(x_off, well_base_y + h - 5.0, Z_BACKGROUND + 1.6),
            TitleEntity,
        ));
    }

    // Fireflies
    for _ in 0..8 {
        let x = rng.gen_range(-VIEWPORT_W / 3.0..VIEWPORT_W / 3.0);
        let y = rng.gen_range(well_base_y + 10.0..well_base_y + 120.0);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.6, 0.8, 0.3, 0.6),
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 7.0),
            TitleEntity,
        ));
    }

    // Title text
    commands.spawn((
        Text2d::new("DAWNROOT"),
        TextFont { font_size: 64.0, ..default() },
        TextColor(Color::srgb(0.9, 0.78, 0.45)),
        Transform::from_xyz(0.0, 160.0, Z_HUD),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("Into the Depths"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.6, 0.55, 0.4)),
        Transform::from_xyz(0.0, 120.0, Z_HUD),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("A/D: Move  |  Space: Jump  |  J: Attack  |  Shift: Dash  |  1-4: Spells"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgb(0.4, 0.38, 0.32)),
        Transform::from_xyz(0.0, -200.0, Z_HUD),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("- Press SPACE to descend -"),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(0.9, 0.8, 0.45, 1.0)),
        Transform::from_xyz(0.0, -230.0, Z_HUD),
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
        let alpha = 0.4 + 0.6 * (time.elapsed_secs() * 2.5).sin().max(0.0);
        color.0 = Color::srgba(0.9, 0.8, 0.45, alpha);
    }

    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(GameState::WellIntro);
    }
}

// ── Well Intro cutscene ───────────────────────────────────────────

/// Marks every entity that belongs to the WellIntro cutscene.
#[derive(Component, Clone, Copy)]
struct IntroEntity;

/// Root entity of the animated intro character (invisible collision hull).
#[derive(Component)]
struct IntroPlayer;

/// Marks every visible child sprite of the intro character for alpha-fading.
#[derive(Component)]
struct IntroPlayerPart;

/// Left leg of the intro character.
#[derive(Component)]
struct IntroLegL;

/// Right leg of the intro character.
#[derive(Component)]
struct IntroLegR;

#[derive(Component)]
struct DarknessOverlay;

#[derive(Component)]
struct FallingParticle {
    vy: f32,
    lifetime: f32,
}

#[derive(Resource)]
struct IntroState {
    phase: IntroPhase,
    timer: f32,
    player_start_y: f32,
}

#[derive(PartialEq)]
enum IntroPhase {
    WalkToWell,
    JumpIn,
    FallDarkness,
    LandInCave,
}

fn setup_well_intro(mut commands: Commands) {
    let ground_y = -VIEWPORT_H / 2.0 + 50.0;
    let well_base_y = ground_y + 52.5;

    commands.insert_resource(IntroState {
        phase: IntroPhase::WalkToWell,
        timer: 0.0,
        player_start_y: well_base_y + 16.0,
    });

    commands.spawn((Camera2d, IntroEntity));

    // Sky
    commands.spawn((
        Sprite { color: Color::srgb(0.02, 0.03, 0.06), custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)), ..default() },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND), IntroEntity,
    ));

    // Ground
    commands.spawn((
        Sprite { color: Color::srgb(0.08, 0.1, 0.05), custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 100.0)), ..default() },
        Transform::from_xyz(0.0, ground_y, Z_BACKGROUND + 2.0), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.1, 0.22, 0.08), custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 5.0)), ..default() },
        Transform::from_xyz(0.0, ground_y + 52.5, Z_BACKGROUND + 3.0), IntroEntity,
    ));

    // Well
    // Base
    commands.spawn((
        Sprite { color: Color::srgb(0.35, 0.3, 0.25), custom_size: Some(Vec2::new(64.0, 28.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 14.0, Z_BACKGROUND + 5.0), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.01, 0.01, 0.02), custom_size: Some(Vec2::new(48.0, 20.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 14.0, Z_BACKGROUND + 5.5), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.35, 0.28), custom_size: Some(Vec2::new(70.0, 8.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 30.0, Z_BACKGROUND + 6.0), IntroEntity,
    ));

    // ── Intro player: invisible root + child body parts ──────────────
    //
    // The root holds the position / scale used for the jump arc and walk
    // bob. All visible pixels live in child entities tagged IntroPlayerPart
    // so we can bulk-fade them during the JumpIn phase.
    commands.spawn((
        Transform::from_xyz(-280.0, well_base_y + 16.0, Z_PLAYER),
        Visibility::Visible,
        IntroEntity,
        IntroPlayer,
    )).with_children(|p| {
        // Body – green tunic
        p.spawn((
            Sprite {
                color: Color::srgb(0.18, 0.50, 0.28),
                custom_size: Some(Vec2::new(14.0, 14.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            IntroEntity,
            IntroPlayerPart,
        ));

        // Belt – brown strap
        p.spawn((
            Sprite {
                color: Color::srgb(0.45, 0.30, 0.15),
                custom_size: Some(Vec2::new(14.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -5.0, 0.15),
            IntroEntity,
            IntroPlayerPart,
        ));

        // Head – slightly warm tone, with hood children
        p.spawn((
            Sprite {
                color: Color::srgb(0.78, 0.62, 0.48),
                custom_size: Some(Vec2::new(12.0, 11.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 12.0, 0.2),
            IntroEntity,
            IntroPlayerPart,
        )).with_children(|head| {
            // Left eye
            head.spawn((
                Sprite {
                    color: Color::srgb(0.9, 0.92, 0.95),
                    custom_size: Some(Vec2::new(2.5, 3.0)),
                    ..default()
                },
                Transform::from_xyz(-2.5, 0.5, 0.1),
                IntroEntity,
                IntroPlayerPart,
            ));
            // Right eye
            head.spawn((
                Sprite {
                    color: Color::srgb(0.9, 0.92, 0.95),
                    custom_size: Some(Vec2::new(2.5, 3.0)),
                    ..default()
                },
                Transform::from_xyz(2.5, 0.5, 0.1),
                IntroEntity,
                IntroPlayerPart,
            ));
            // Hood – darker green drape over top of head
            head.spawn((
                Sprite {
                    color: Color::srgb(0.12, 0.34, 0.18),
                    custom_size: Some(Vec2::new(14.0, 5.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 4.5, 0.15),
                IntroEntity,
                IntroPlayerPart,
            ));
        });

        // Left leg
        p.spawn((
            Sprite {
                color: Color::srgb(0.28, 0.22, 0.16),
                custom_size: Some(Vec2::new(5.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(-3.5, -12.0, 0.0),
            IntroEntity,
            IntroPlayerPart,
            IntroLegL,
        ));

        // Right leg
        p.spawn((
            Sprite {
                color: Color::srgb(0.26, 0.20, 0.14),
                custom_size: Some(Vec2::new(5.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(3.5, -12.0, 0.0),
            IntroEntity,
            IntroPlayerPart,
            IntroLegR,
        ));

        // Sword – small gray rectangle on right side
        p.spawn((
            Sprite {
                color: Color::srgb(0.68, 0.70, 0.74),
                custom_size: Some(Vec2::new(3.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(10.0, 3.0, 0.35),
            IntroEntity,
            IntroPlayerPart,
        ));
    });

    // Darkness overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(VIEWPORT_W + 100.0, VIEWPORT_H + 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_HUD - 1.0),
        IntroEntity,
        DarknessOverlay,
    ));
}

fn update_well_intro(
    mut commands: Commands,
    mut state: ResMut<IntroState>,
    // Root entity – only needs Transform, no Sprite.
    mut player_q: Query<&mut Transform, (With<IntroPlayer>, Without<IntroLegL>, Without<IntroLegR>)>,
    // All visible child parts – for alpha fading.
    mut parts_q: Query<&mut Sprite, (With<IntroPlayerPart>, Without<DarknessOverlay>)>,
    // Leg children – for walk animation.
    mut legl_q: Query<&mut Transform, (With<IntroLegL>, Without<IntroPlayer>, Without<IntroLegR>)>,
    mut legr_q: Query<&mut Transform, (With<IntroLegR>, Without<IntroPlayer>, Without<IntroLegL>)>,
    mut darkness_q: Query<&mut Sprite, (With<DarknessOverlay>, Without<IntroPlayerPart>)>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    state.timer += dt;

    let Ok(mut p_tf) = player_q.get_single_mut() else { return };

    match state.phase {
        IntroPhase::WalkToWell => {
            // Walk rightward from -280 toward the well at x=0.
            p_tf.translation.x += 200.0 * dt;

            // Vertical walk bob on the root so all parts move together.
            let bob = (state.timer * 14.0).sin().abs() * 3.0;
            p_tf.translation.y = state.player_start_y + bob;

            // Leg stride animation using sin waves (opposite phase each leg).
            let sw_l = (state.timer * 14.0).sin() * 6.0;
            let sw_r = (state.timer * 14.0 + std::f32::consts::PI).sin() * 6.0;

            if let Ok(mut ll) = legl_q.get_single_mut() {
                ll.translation = Vec3::new(-3.5 + sw_l * 0.3, -12.0 + sw_l.abs() * 0.5, 0.0);
            }
            if let Ok(mut rl) = legr_q.get_single_mut() {
                rl.translation = Vec3::new(3.5 + sw_r * 0.3, -12.0 + sw_r.abs() * 0.5, 0.0);
            }

            if p_tf.translation.x >= -5.0 {
                p_tf.translation.x = 0.0;
                // Reset legs to standing pose before the jump.
                if let Ok(mut ll) = legl_q.get_single_mut() {
                    ll.translation = Vec3::new(-3.5, -12.0, 0.0);
                }
                if let Ok(mut rl) = legr_q.get_single_mut() {
                    rl.translation = Vec3::new(3.5, -12.0, 0.0);
                }
                state.phase = IntroPhase::JumpIn;
                state.timer = 0.0;
            }
        }

        IntroPhase::JumpIn => {
            let t = state.timer;
            if t < 0.35 {
                // Arc up over the well rim.
                let frac = t / 0.35;
                let y_off = 55.0 * (frac * std::f32::consts::PI).sin();
                p_tf.translation.y = state.player_start_y + y_off;
                let stretch = 1.0 + 0.25 * frac;
                p_tf.scale = Vec3::new(1.0 / stretch.sqrt(), stretch, 1.0);
            } else if t < 0.9 {
                // Drop into the well hole – fade all parts out as the
                // character descends below the well rim.
                let fall_frac = (t - 0.35) / 0.55;
                p_tf.translation.y = state.player_start_y + 10.0 - fall_frac * 90.0;
                p_tf.scale = Vec3::new(0.8, 1.15, 1.0);

                let alpha = (1.0 - fall_frac).max(0.0);
                for mut sprite in parts_q.iter_mut() {
                    // Preserve existing rgb, only update alpha.
                    let c = sprite.color.to_srgba();
                    sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
                }
            } else {
                // Fully hidden – zero out alpha on all parts.
                for mut sprite in parts_q.iter_mut() {
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
                state.phase = IntroPhase::FallDarkness;
                state.timer = 0.0;
            }
        }

        IntroPhase::FallDarkness => {
            // Fade the screen to black.
            if let Ok(mut ds) = darkness_q.get_single_mut() {
                let alpha = (state.timer / 0.6).min(1.0);
                ds.color = Color::srgba(0.0, 0.0, 0.0, alpha);
            }

            // Falling debris particles.
            if state.timer > 0.3 && state.timer < 1.8 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for _ in 0..2 {
                    let px = rng.gen_range(-80.0..80.0_f32);
                    let speed = rng.gen_range(300.0..500.0_f32);
                    let br = rng.gen_range(0.15..0.35_f32);
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(br, br * 0.9, br * 0.7, 0.7),
                            custom_size: Some(Vec2::new(3.0, rng.gen_range(6.0..14.0))),
                            ..default()
                        },
                        Transform::from_xyz(px, VIEWPORT_H / 2.0 + 10.0, Z_HUD - 0.5),
                        IntroEntity,
                        FallingParticle { vy: -speed, lifetime: 1.2 },
                    ));
                }
            }

            if state.timer >= 2.2 {
                state.phase = IntroPhase::LandInCave;
                state.timer = 0.0;
            }
        }

        IntroPhase::LandInCave => {
            // Brief warm flash then transition to gameplay.
            if let Ok(mut ds) = darkness_q.get_single_mut() {
                if state.timer < 0.1 {
                    ds.color = Color::srgba(0.12, 0.08, 0.05, 0.85);
                } else {
                    ds.color = Color::srgba(0.0, 0.0, 0.0, 1.0);
                }
            }

            if state.timer >= 0.6 {
                next_state.set(GameState::Playing);
            }
        }
    }
}

fn cleanup_well_intro(
    mut commands: Commands,
    q: Query<Entity, With<IntroEntity>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<IntroState>();
}

fn update_falling_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FallingParticle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut p) in &mut query {
        tf.translation.y += p.vy * dt;
        p.lifetime -= dt;
        if p.lifetime <= 0.0 || tf.translation.y < -VIEWPORT_H / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}
