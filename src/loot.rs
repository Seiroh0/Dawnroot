use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, enemy::EnemyDefeated};

pub struct LootPlugin;

impl Plugin for LootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_drops,
                pickup_magnet,
                collect_pickups,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
#[allow(dead_code)]
pub struct Pickup {
    pub kind: PickupKind,
    pub magnet_radius: f32,
    pub lifetime: f32,
}

#[derive(Clone, Copy)]
pub enum PickupKind {
    Gold(i32),
    Health,
    Mana(f32),
}

fn spawn_drops(
    mut commands: Commands,
    mut ev: EventReader<EnemyDefeated>,
) {
    for event in ev.read() {
        // Gold drop
        if event.gold_drop > 0 {
            commands.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.85, 0.15),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_translation(event.position + Vec3::new(0.0, 10.0, Z_PICKUPS - Z_ENEMIES)),
                Pickup {
                    kind: PickupKind::Gold(event.gold_drop),
                    magnet_radius: 80.0,
                    lifetime: 10.0,
                },
                PlayingEntity,
            ));
        }

        // Chance for health drop (20%)
        let roll = rand::random::<f32>();
        if roll < 0.2 {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.9, 0.2, 0.3),
                    custom_size: Some(Vec2::new(10.0, 10.0)),
                    ..default()
                },
                Transform::from_translation(event.position + Vec3::new(12.0, 10.0, Z_PICKUPS - Z_ENEMIES)),
                Pickup {
                    kind: PickupKind::Health,
                    magnet_radius: 60.0,
                    lifetime: 10.0,
                },
                PlayingEntity,
            ));
        }

        // Chance for mana drop (30%)
        if roll >= 0.2 && roll < 0.5 {
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.3, 0.4, 0.9),
                    custom_size: Some(Vec2::new(8.0, 8.0)),
                    ..default()
                },
                Transform::from_translation(event.position + Vec3::new(-10.0, 10.0, Z_PICKUPS - Z_ENEMIES)),
                Pickup {
                    kind: PickupKind::Mana(20.0),
                    magnet_radius: 60.0,
                    lifetime: 10.0,
                },
                PlayingEntity,
            ));
        }
    }
}

fn pickup_magnet(
    mut pickup_q: Query<(&mut Transform, &Pickup)>,
    player_q: Query<&Transform, (With<Player>, Without<Pickup>)>,
    time: Res<Time>,
) {
    let Ok(p_tf) = player_q.get_single() else { return };

    for (mut tf, pickup) in &mut pickup_q {
        let diff = p_tf.translation.xy() - tf.translation.xy();
        let dist = diff.length();

        if dist < pickup.magnet_radius && dist > 5.0 {
            let dir = diff.normalize();
            let speed = 200.0 * (1.0 - dist / pickup.magnet_radius);
            tf.translation.x += dir.x * speed * time.delta_secs();
            tf.translation.y += dir.y * speed * time.delta_secs();
        }
    }
}

fn collect_pickups(
    mut commands: Commands,
    pickup_q: Query<(Entity, &Transform, &Pickup)>,
    player_q: Query<&Transform, With<Player>>,
    mut player_mut: Query<&mut Player>,
    mut run: ResMut<RunData>,
    time: Res<Time>,
) {
    let Ok(p_tf) = player_q.get_single() else { return };

    for (entity, tf, pickup) in &pickup_q {
        let dist = (p_tf.translation.xy() - tf.translation.xy()).length();

        if dist < 18.0 {
            match pickup.kind {
                PickupKind::Gold(amount) => {
                    run.gold += amount;
                    run.score += amount * 10;
                }
                PickupKind::Health => {
                    if let Ok(mut player) = player_mut.get_single_mut() {
                        player.health = (player.health + 1).min(player.max_health);
                    }
                }
                PickupKind::Mana(amount) => {
                    if let Ok(mut player) = player_mut.get_single_mut() {
                        player.mana = (player.mana + amount).min(player.max_mana);
                    }
                }
            }
            commands.entity(entity).despawn();
        }
    }

    // Lifetime despawn
    // Note: We can't mutably borrow Pickup in the loop above due to the Query constraints,
    // so handle lifetime separately. For now pickups persist until collected.
    let _ = time;
}
