use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, RunData, PlayingEntity,
    player::{Player, MeleeHitbox, PlayerProjectile, PlayerDamaged, PlayerDied, PlayerBlocked},
    enemy::{Enemy, EnemyDefeated, EnemyProjectile, Intangible, SlimeEnemy},
    spell::{SpellProjectile, LightningStrike},
    camera::{ScreenShake, trigger_shake},
    equipment::PlayerStats,
    room::{RoomState, DestructibleWall},
    loot::{Pickup, PickupKind},
};

#[derive(Event)]
pub struct DamageNumberEvent {
    pub position: Vec3,
    pub amount: i32,
    pub kind: DamageNumberKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DamageNumberKind {
    PlayerHit,
    EnemyHit,
    CritHit,
    Blocked,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageNumberEvent>()
        .add_systems(
            Update,
            (
                melee_vs_enemy,
                ranged_vs_enemy,
                spell_vs_enemy,
                lightning_vs_enemy,
                player_vs_enemy,
                enemy_projectile_vs_player,
                melee_vs_wall,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Shared kill-enemy bookkeeping: update RunData, fire event, despawn entity.
fn kill_enemy(
    commands: &mut Commands,
    run: &mut RunData,
    ev_defeated: &mut EventWriter<EnemyDefeated>,
    entity: Entity,
    position: Vec3,
    enemy: &Enemy,
) {
    run.score += enemy.score_reward;
    run.gold += enemy.gold_drop;
    run.enemies_killed += 1;
    ev_defeated.send(EnemyDefeated {
        position,
        score: enemy.score_reward,
        gold_drop: enemy.gold_drop,
    });
    commands.entity(entity).try_despawn_recursive();
}

fn melee_vs_enemy(
    mut commands: Commands,
    hitbox_q: Query<(&Transform, &MeleeHitbox)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite, Option<&Intangible>, Option<&SlimeEnemy>)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
    room_state: Res<RoomState>,
) {
    for (h_tf, hitbox) in &hitbox_q {
        for (e_entity, e_tf, mut enemy, sprite, intangible, slime) in &mut enemy_q {
            if intangible.is_some() { continue; }
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (h_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < MELEE_RANGE / 2.0 + e_size.x / 2.0
                && dist.y < MELEE_WIDTH / 2.0 + e_size.y / 2.0
            {
                let crit_bonus = (hitbox.damage as f32 * stats.crit_chance) as i32;
                let total_dmg = hitbox.damage + stats.attack + crit_bonus;
                enemy.health -= total_dmg;
                let kind = if crit_bonus > 0 { DamageNumberKind::CritHit } else { DamageNumberKind::EnemyHit };
                ev_dmg.send(DamageNumberEvent { position: e_tf.translation, amount: total_dmg, kind });

                if enemy.health <= 0 {
                    // Slime split on death
                    if let Some(se) = slime {
                        crate::enemy::slime_split_on_death(&mut commands, e_tf.translation, se.size, room_state.floor, &mut run);
                    }
                    kill_enemy(&mut commands, &mut run, &mut ev_defeated, e_entity, e_tf.translation, &enemy);
                    if let Ok(mut shake) = shake_q.get_single_mut() {
                        trigger_shake(&mut shake, 8.0, 0.15);
                    }
                }
            }
        }
    }
}

fn ranged_vs_enemy(
    mut commands: Commands,
    proj_q: Query<(Entity, &Transform, &PlayerProjectile)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite, Option<&Intangible>, Option<&SlimeEnemy>)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
    room_state: Res<RoomState>,
) {
    for (p_entity, p_tf, proj) in &proj_q {
        for (e_entity, e_tf, mut enemy, sprite, intangible, slime) in &mut enemy_q {
            if intangible.is_some() { continue; }
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (p_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < 5.0 + e_size.x / 2.0 && dist.y < 3.0 + e_size.y / 2.0 {
                let total_dmg = proj.damage + stats.attack;
                enemy.health -= total_dmg;
                ev_dmg.send(DamageNumberEvent { position: e_tf.translation, amount: total_dmg, kind: DamageNumberKind::EnemyHit });
                commands.entity(p_entity).try_despawn_recursive();

                if enemy.health <= 0 {
                    if let Some(se) = slime {
                        crate::enemy::slime_split_on_death(&mut commands, e_tf.translation, se.size, room_state.floor, &mut run);
                    }
                    kill_enemy(&mut commands, &mut run, &mut ev_defeated, e_entity, e_tf.translation, &enemy);
                    if let Ok(mut shake) = shake_q.get_single_mut() {
                        trigger_shake(&mut shake, 6.0, 0.12);
                    }
                }
                break;
            }
        }
    }
}

fn spell_vs_enemy(
    mut commands: Commands,
    proj_q: Query<(Entity, &Transform, &SpellProjectile)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite, Option<&Intangible>, Option<&SlimeEnemy>)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
    room_state: Res<RoomState>,
) {
    for (p_entity, p_tf, proj) in &proj_q {
        for (e_entity, e_tf, mut enemy, sprite, intangible, slime) in &mut enemy_q {
            if intangible.is_some() { continue; }
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (p_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < 8.0 + e_size.x / 2.0 && dist.y < 8.0 + e_size.y / 2.0 {
                let total_dmg = proj.damage + stats.attack;
                enemy.health -= total_dmg;
                ev_dmg.send(DamageNumberEvent { position: e_tf.translation, amount: total_dmg, kind: DamageNumberKind::EnemyHit });
                commands.entity(p_entity).despawn();

                if enemy.health <= 0 {
                    if let Some(se) = slime {
                        crate::enemy::slime_split_on_death(&mut commands, e_tf.translation, se.size, room_state.floor, &mut run);
                    }
                    kill_enemy(&mut commands, &mut run, &mut ev_defeated, e_entity, e_tf.translation, &enemy);
                    if let Ok(mut shake) = shake_q.get_single_mut() {
                        trigger_shake(&mut shake, 10.0, 0.2);
                    }
                }
                break;
            }
        }
    }
}

fn lightning_vs_enemy(
    mut commands: Commands,
    strike_q: Query<(&Transform, &LightningStrike)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, Option<&Intangible>, Option<&SlimeEnemy>)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    room_state: Res<RoomState>,
) {
    for (s_tf, strike) in &strike_q {
        if strike.lifetime < 0.12 { continue; }

        for (e_entity, e_tf, mut enemy, intangible, slime) in &mut enemy_q {
            if intangible.is_some() { continue; }
            let dist = (s_tf.translation.xy() - e_tf.translation.xy()).length();

            if dist < strike.radius {
                enemy.health -= strike.damage;
                ev_dmg.send(DamageNumberEvent { position: e_tf.translation, amount: strike.damage, kind: DamageNumberKind::EnemyHit });

                if enemy.health <= 0 {
                    if let Some(se) = slime {
                        crate::enemy::slime_split_on_death(&mut commands, e_tf.translation, se.size, room_state.floor, &mut run);
                    }
                    kill_enemy(&mut commands, &mut run, &mut ev_defeated, e_entity, e_tf.translation, &enemy);
                }
            }
        }

        if let Ok(mut shake) = shake_q.get_single_mut() {
            trigger_shake(&mut shake, 12.0, 0.25);
        }
    }
}

/// Apply block damage reduction: returns reduced damage amount.
fn apply_block_reduction(raw_dmg: i32, is_blocking: bool) -> i32 {
    if is_blocking {
        ((raw_dmg as f32) * (1.0 - BLOCK_DAMAGE_REDUCTION)).ceil() as i32
    } else {
        raw_dmg
    }.max(0)
}

fn player_vs_enemy(
    mut player_q: Query<(&Transform, &mut Player)>,
    enemy_q: Query<(&Transform, &Enemy, &Sprite, Option<&Intangible>), Without<Player>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    mut ev_blocked: EventWriter<PlayerBlocked>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (e_tf, enemy, sprite, intangible) in &enemy_q {
        if intangible.is_some() { continue; }
        let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
        let diff = p_tf.translation.xy() - e_tf.translation.xy();
        let dist = diff.abs();

        if dist.x < 10.0 + e_size.x / 2.0 && dist.y < 16.0 + e_size.y / 2.0 {
            if player.invulnerable <= 0.0 {
                let raw_dmg = (enemy.contact_damage - stats.defense).max(1);
                let is_blocked = player.is_blocking;
                if is_blocked {
                    ev_blocked.send(PlayerBlocked { position: p_tf.translation });
                }
                let reduced = apply_block_reduction(raw_dmg, is_blocked);
                player.health -= reduced;
                player.invulnerable = INVULN_TIME;
                let kind = if is_blocked { DamageNumberKind::Blocked } else { DamageNumberKind::PlayerHit };
                ev_dmg.send(DamageNumberEvent { position: p_tf.translation, amount: reduced, kind });
                ev_damaged.send(PlayerDamaged {
                    amount: reduced,
                    remaining: player.health,
                });

                if let Ok(mut shake) = shake_q.get_single_mut() {
                    let intensity = if is_blocked { 6.0 } else { 14.0 };
                    trigger_shake(&mut shake, intensity, 0.3);
                }

                let kb_dir = if diff.x >= 0.0 { 1.0 } else { -1.0 };
                let kb_mult = if is_blocked { 0.5 } else { 1.0 };
                player.vx = kb_dir * 250.0 * kb_mult;
                player.vy = 200.0 * kb_mult;

                if player.health <= 0 {
                    ev_died.send(PlayerDied);
                }
            }
            break;
        }
    }
}

fn enemy_projectile_vs_player(
    mut commands: Commands,
    proj_q: Query<(Entity, &Transform, &EnemyProjectile)>,
    mut player_q: Query<(&Transform, &mut Player), Without<EnemyProjectile>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    mut ev_blocked: EventWriter<PlayerBlocked>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (proj_entity, proj_tf, proj) in &proj_q {
        let dist = (p_tf.translation.xy() - proj_tf.translation.xy()).abs();
        if dist.x < 14.0 && dist.y < 20.0 {
            commands.entity(proj_entity).despawn();
            if player.invulnerable <= 0.0 {
                let raw_dmg = (proj.damage - stats.defense).max(1);
                let is_blocked = player.is_blocking;
                if is_blocked {
                    ev_blocked.send(PlayerBlocked { position: p_tf.translation });
                }
                let dmg = apply_block_reduction(raw_dmg, is_blocked);
                player.health -= dmg;
                player.invulnerable = INVULN_TIME;
                let kind = if is_blocked { DamageNumberKind::Blocked } else { DamageNumberKind::PlayerHit };
                ev_dmg.send(DamageNumberEvent { position: p_tf.translation, amount: dmg, kind });
                ev_damaged.send(PlayerDamaged {
                    amount: dmg,
                    remaining: player.health,
                });
                if let Ok(mut shake) = shake_q.get_single_mut() {
                    let intensity = if is_blocked { 4.0 } else { 10.0 };
                    trigger_shake(&mut shake, intensity, 0.2);
                }
                if player.health <= 0 {
                    ev_died.send(PlayerDied);
                }
            }
            break;
        }
    }
}

/// Melee attacks can break destructible walls to reveal secret loot.
fn melee_vs_wall(
    mut commands: Commands,
    hitbox_q: Query<&Transform, With<MeleeHitbox>>,
    mut wall_q: Query<(Entity, &Transform, &mut DestructibleWall)>,
    mut ev_dmg: EventWriter<DamageNumberEvent>,
    mut shake_q: Query<&mut ScreenShake>,
) {
    for h_tf in &hitbox_q {
        for (w_entity, w_tf, mut wall) in &mut wall_q {
            let dist = (h_tf.translation.xy() - w_tf.translation.xy()).abs();

            if dist.x < MELEE_RANGE / 2.0 + TILE_SIZE / 2.0
                && dist.y < MELEE_WIDTH / 2.0 + TILE_SIZE / 2.0
            {
                wall.health -= 1;
                ev_dmg.send(DamageNumberEvent {
                    position: w_tf.translation,
                    amount: 1,
                    kind: DamageNumberKind::EnemyHit,
                });

                if wall.health <= 0 {
                    let pos = w_tf.translation;
                    commands.entity(w_entity).try_despawn_recursive();

                    // Spawn secret loot: gold + health + mana burst
                    for i in 0..4_u32 {
                        let angle = (i as f32 / 4.0) * std::f32::consts::TAU;
                        let spread = 15.0 + (i as f32 * 8.0);
                        commands.spawn((
                            Sprite {
                                color: Color::srgb(1.0, 0.85, 0.15),
                                custom_size: Some(Vec2::new(10.0, 10.0)),
                                ..default()
                            },
                            Transform::from_xyz(
                                pos.x + angle.cos() * spread,
                                pos.y + angle.sin() * spread,
                                Z_EFFECTS,
                            ),
                            Pickup {
                                kind: PickupKind::Gold(5 + (i as i32 * 3)),
                                magnet_radius: 100.0,
                                lifetime: 10.0,
                            },
                            PlayingEntity,
                        ));
                    }
                    // Health pickup
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.9, 0.3, 0.15),
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y + 20.0, Z_EFFECTS),
                        Pickup {
                            kind: PickupKind::Health,
                            magnet_radius: 80.0,
                            lifetime: 10.0,
                        },
                        PlayingEntity,
                    ));
                    // Mana pickup
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.7, 0.4, 0.85),
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y - 10.0, Z_EFFECTS),
                        Pickup {
                            kind: PickupKind::Mana(25.0),
                            magnet_radius: 80.0,
                            lifetime: 10.0,
                        },
                        PlayingEntity,
                    ));

                    if let Ok(mut shake) = shake_q.get_single_mut() {
                        trigger_shake(&mut shake, 10.0, 0.2);
                    }
                }
            }
        }
    }
}
