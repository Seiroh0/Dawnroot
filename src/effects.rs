use bevy::prelude::*;
use crate::{
    GameState, PlayingEntity,
    enemy::EnemyDefeated,
    player::{PlayerDamaged, PlayerLanded, PlayerDashed},
    room::RoomCleared,
};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_enemy_defeated,
                on_player_damaged,
                on_room_cleared,
                on_player_dash,
                on_player_landed,
                update_particles,
                update_confetti,
                update_flash_sprites,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Standard physics particle: affected by gravity, fades out.
#[derive(Component)]
struct Particle {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

/// Confetti particle: has downward gravity, gentle horizontal drift, slow fade.
#[derive(Component)]
struct Confetti {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

/// A sprite that simply fades its alpha to zero over its lifetime.
#[derive(Component)]
struct FlashSprite {
    lifetime: f32,
    max_lifetime: f32,
}

// ---------------------------------------------------------------------------
// Enemy defeated
// ---------------------------------------------------------------------------

fn on_enemy_defeated(
    mut commands: Commands,
    mut ev: EventReader<EnemyDefeated>,
) {
    for event in ev.read() {
        let pos = event.position;

        // --- 12 red/crimson burst particles with varied sizes ---
        for i in 0..12_u32 {
            let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
            let speed = 70.0 + (i as f32 % 4.0) * 45.0;
            // Alternate between small and large fragments
            let size = if i % 3 == 0 { 6.0 } else if i % 3 == 1 { 4.0 } else { 2.5 };
            let red_shade = if i % 2 == 0 {
                Color::srgb(0.9, 0.2, 0.25)
            } else {
                Color::srgb(0.7, 0.1, 0.15)
            };
            commands.spawn((
                Sprite {
                    color: red_shade,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(pos),
                Particle {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: 0.45,
                    max_lifetime: 0.45,
                },
                PlayingEntity,
            ));
        }

        // --- 4 gold sparkles (loot signal) ---
        for i in 0..4_u32 {
            let angle = (i as f32 / 4.0) * std::f32::consts::TAU + 0.3;
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.85, 0.2),
                    custom_size: Some(Vec2::new(3.5, 3.5)),
                    ..default()
                },
                Transform::from_translation(pos),
                Particle {
                    vx: angle.cos() * 60.0,
                    vy: angle.sin() * 80.0 + 50.0,
                    lifetime: 0.55,
                    max_lifetime: 0.55,
                },
                PlayingEntity,
            ));
        }

        // --- Brief white flash disc that fades quickly ---
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.95),
                custom_size: Some(Vec2::new(32.0, 32.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, pos.z + 1.0),
            FlashSprite {
                lifetime: 0.12,
                max_lifetime: 0.12,
            },
            PlayingEntity,
        ));
    }
}

// ---------------------------------------------------------------------------
// Player damaged
// ---------------------------------------------------------------------------

fn on_player_damaged(
    mut commands: Commands,
    mut ev: EventReader<PlayerDamaged>,
    player_q: Query<&Transform, With<crate::player::Player>>,
) {
    for _event in ev.read() {
        let Ok(p_tf) = player_q.get_single() else { continue };
        let pos = p_tf.translation;

        // --- Red particle burst ---
        for i in 0..14_u32 {
            let angle = (i as f32 / 14.0) * std::f32::consts::TAU;
            let speed = 90.0 + (i as f32 % 4.0) * 35.0;
            let size = if i % 2 == 0 { 4.0 } else { 2.5 };
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.15, 0.15),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_translation(pos),
                Particle {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: 0.55,
                    max_lifetime: 0.55,
                },
                PlayingEntity,
            ));
        }

        // --- Screen-edge red vignette: large translucent red rect centred on
        //     player that fades out rapidly. Gives a "blood splatter" feel. ---
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.05, 0.05, 0.55),
                custom_size: Some(Vec2::new(
                    crate::constants::VIEWPORT_W,
                    crate::constants::VIEWPORT_H,
                )),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, crate::constants::Z_HUD - 5.0),
            FlashSprite {
                lifetime: 0.35,
                max_lifetime: 0.35,
            },
            PlayingEntity,
        ));
    }
}

// ---------------------------------------------------------------------------
// Room cleared: confetti celebration
// ---------------------------------------------------------------------------

fn on_room_cleared(
    mut commands: Commands,
    mut ev: EventReader<RoomCleared>,
) {
    for _event in ev.read() {
        let center = Vec3::new(
            crate::constants::ROOM_W / 2.0,
            crate::constants::ROOM_H * 0.75, // spawn near top so they float down
            crate::constants::Z_EFFECTS,
        );

        // 32 confetti pieces in green/gold/white spread across the ceiling area
        for i in 0..32_u32 {
            // Stagger spawn positions horizontally across the room
            let x_offset = (i as f32 / 32.0 - 0.5) * crate::constants::ROOM_W * 0.8;
            let y_jitter = (i as f32 * 31.7 % 60.0) - 30.0;

            let color = match i % 4 {
                0 => Color::srgb(0.25, 0.88, 0.38), // bright green
                1 => Color::srgb(0.95, 0.82, 0.18), // gold
                2 => Color::srgb(0.2, 0.7, 1.0),    // sky blue
                _ => Color::srgb(1.0, 0.95, 0.95),  // near-white
            };

            // Rectangular confetti pieces look more festive than squares
            let (w, h) = if i % 2 == 0 { (6.0, 3.0) } else { (3.0, 7.0) };

            // Spread out initial horizontal speed; mostly fall down
            let vx = (i as f32 * 13.7 % 80.0) - 40.0;
            let vy = -(30.0 + (i as f32 * 17.3 % 60.0)); // start falling

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(w, h)),
                    ..default()
                },
                Transform::from_xyz(center.x + x_offset, center.y + y_jitter, center.z),
                Confetti {
                    vx,
                    vy,
                    lifetime: 1.4 + (i as f32 * 0.03),
                    max_lifetime: 1.4 + (i as f32 * 0.03),
                },
                PlayingEntity,
            ));
        }

        // Extra bright "pop" flash at room centre
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.95, 0.5, 0.7),
                custom_size: Some(Vec2::new(200.0, 200.0)),
                ..default()
            },
            Transform::from_xyz(center.x, crate::constants::ROOM_H / 2.0, crate::constants::Z_EFFECTS + 1.0),
            FlashSprite {
                lifetime: 0.2,
                max_lifetime: 0.2,
            },
            PlayingEntity,
        ));
    }
}

// ---------------------------------------------------------------------------
// Dash afterimage
// ---------------------------------------------------------------------------

fn on_player_dash(
    mut commands: Commands,
    mut ev: EventReader<PlayerDashed>,
) {
    for event in ev.read() {
        // Spawn three translucent player-colour afterimage rectangles
        // staggered slightly behind the dash origin.
        for k in 0..3_u32 {
            let trail_offset = -(event.facing * (k as f32 * 10.0 + 5.0));
            let alpha = 0.55 - k as f32 * 0.15; // 0.55, 0.40, 0.25
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.18, 0.5, 0.28, alpha), // player tunic green
                    custom_size: Some(Vec2::new(14.0, 30.0)),     // approximate player silhouette
                    ..default()
                },
                Transform::from_xyz(
                    event.position.x + trail_offset,
                    event.position.y,
                    crate::constants::Z_PLAYER - 1.0,
                ),
                FlashSprite {
                    lifetime: 0.12,
                    max_lifetime: 0.12,
                },
                PlayingEntity,
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Landing dust puffs
// ---------------------------------------------------------------------------

fn on_player_landed(
    mut commands: Commands,
    mut ev: EventReader<PlayerLanded>,
    player_q: Query<&Transform, With<crate::player::Player>>,
) {
    for _event in ev.read() {
        let Ok(p_tf) = player_q.get_single() else { continue };
        let foot = Vec3::new(p_tf.translation.x, p_tf.translation.y - 16.0, crate::constants::Z_EFFECTS);

        // 6 dust puffs spreading left and right horizontally
        for i in 0..6_u32 {
            // Alternate left/right, fan outward
            let side = if i % 2 == 0 { 1.0_f32 } else { -1.0_f32 };
            let spread_vx = side * (30.0 + i as f32 * 18.0);
            let vy = 25.0 + (i as f32 * 5.0); // small upward kick
            let size = 5.0 - i as f32 * 0.5; // shrink as they fan out
            let gray = 0.55 + (i as f32 * 0.04);

            commands.spawn((
                Sprite {
                    color: Color::srgba(gray, gray * 0.9, gray * 0.8, 0.75),
                    custom_size: Some(Vec2::splat(size.max(2.0))),
                    ..default()
                },
                Transform::from_translation(foot),
                Particle {
                    vx: spread_vx,
                    vy,
                    lifetime: 0.28,
                    max_lifetime: 0.28,
                },
                PlayingEntity,
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Update systems
// ---------------------------------------------------------------------------

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut p) in &mut query {
        p.lifetime -= dt;
        if p.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Gravity
        p.vy -= 380.0 * dt;
        tf.translation.x += p.vx * dt;
        tf.translation.y += p.vy * dt;

        let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

fn update_confetti(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Confetti)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut c) in &mut query {
        c.lifetime -= dt;
        if c.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Light gravity, horizontal air resistance, gentle side-to-side flutter
        c.vy -= 90.0 * dt;
        c.vx += ((tf.translation.y * 0.031).sin() * 15.0) * dt; // flutter
        c.vx *= 0.97_f32.powf(dt * 60.0);

        tf.translation.x += c.vx * dt;
        tf.translation.y += c.vy * dt;

        // Fade only in the last 40 % of lifetime
        let fade_start = 0.4;
        let alpha = if c.lifetime / c.max_lifetime > fade_start {
            1.0
        } else {
            (c.lifetime / (c.max_lifetime * fade_start)).clamp(0.0, 1.0)
        };
        let col = sprite.color.to_srgba();
        sprite.color = Color::srgba(col.red, col.green, col.blue, alpha);
    }
}

fn update_flash_sprites(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut FlashSprite)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut sprite, mut flash) in &mut query {
        flash.lifetime -= dt;
        if flash.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (flash.lifetime / flash.max_lifetime).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}
