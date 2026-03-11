use bevy::prelude::*;
use crate::{GameState, PlayingEntity, enemy::EnemyDefeated, player::PlayerDamaged, room::RoomCleared};

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                on_enemy_defeated,
                on_player_damaged,
                on_room_cleared,
                update_particles,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
struct Particle {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

fn on_enemy_defeated(
    mut commands: Commands,
    mut ev: EventReader<EnemyDefeated>,
) {
    for event in ev.read() {
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * std::f32::consts::TAU;
            let speed = 80.0 + (i as f32 % 3.0) * 40.0;
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.85, 0.3, 0.35),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                Transform::from_translation(event.position),
                Particle {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: 0.4,
                    max_lifetime: 0.4,
                },
                PlayingEntity,
            ));
        }

        // Gold sparkle
        for i in 0..4 {
            let angle = (i as f32 / 4.0) * std::f32::consts::TAU + 0.3;
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.85, 0.2),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(event.position),
                Particle {
                    vx: angle.cos() * 60.0,
                    vy: angle.sin() * 80.0 + 50.0,
                    lifetime: 0.5,
                    max_lifetime: 0.5,
                },
                PlayingEntity,
            ));
        }
    }
}

fn on_player_damaged(
    mut commands: Commands,
    mut ev: EventReader<PlayerDamaged>,
    player_q: Query<&Transform, With<crate::player::Player>>,
) {
    for _event in ev.read() {
        let Ok(p_tf) = player_q.get_single() else { continue };
        for i in 0..12 {
            let angle = (i as f32 / 12.0) * std::f32::consts::TAU;
            let speed = 100.0 + (i as f32 % 4.0) * 30.0;
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.3, 0.3),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_translation(p_tf.translation),
                Particle {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: 0.5,
                    max_lifetime: 0.5,
                },
                PlayingEntity,
            ));
        }
    }
}

fn on_room_cleared(
    mut commands: Commands,
    mut ev: EventReader<RoomCleared>,
) {
    for _event in ev.read() {
        // Celebration burst at center of room
        let center = Vec3::new(
            crate::constants::ROOM_W / 2.0,
            crate::constants::ROOM_H / 2.0,
            crate::constants::Z_EFFECTS,
        );
        for i in 0..16 {
            let angle = (i as f32 / 16.0) * std::f32::consts::TAU;
            let speed = 120.0 + (i as f32 % 4.0) * 30.0;
            let color = if i % 2 == 0 {
                Color::srgb(0.3, 0.9, 0.4)
            } else {
                Color::srgb(0.9, 0.85, 0.3)
            };
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(5.0, 5.0)),
                    ..default()
                },
                Transform::from_translation(center),
                Particle {
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    lifetime: 0.7,
                    max_lifetime: 0.7,
                },
                PlayingEntity,
            ));
        }
    }
}

fn update_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut particle) in &mut query {
        particle.lifetime -= dt;
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        particle.vy -= 400.0 * dt;
        tf.translation.x += particle.vx * dt;
        tf.translation.y += particle.vy * dt;

        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}
