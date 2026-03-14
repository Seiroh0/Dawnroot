use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, RunData,
    player::{Player, MeleeHitbox, PlayerProjectile, PlayerDamaged, PlayerDied, PlayerBlocked},
    enemy::{Enemy, EnemyDefeated, EnemyProjectile},
    spell::{SpellProjectile, LightningStrike},
    camera::{ScreenShake, trigger_shake},
    equipment::PlayerStats,
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                melee_vs_enemy,
                ranged_vs_enemy,
                spell_vs_enemy,
                lightning_vs_enemy,
                player_vs_enemy,
                enemy_projectile_vs_player,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn melee_vs_enemy(
    mut commands: Commands,
    hitbox_q: Query<(&Transform, &MeleeHitbox)>,
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    for (h_tf, hitbox) in &hitbox_q {
        for (e_entity, e_tf, mut enemy, sprite) in &mut enemy_q {
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (h_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < MELEE_RANGE / 2.0 + e_size.x / 2.0
                && dist.y < MELEE_WIDTH / 2.0 + e_size.y / 2.0
            {
                let bonus = stats.attack + ((hitbox.damage as f32 * stats.crit_chance) as i32);
                enemy.health -= hitbox.damage + bonus;

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    run.enemies_killed += 1;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).try_despawn_recursive();

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
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    for (p_entity, p_tf, proj) in &proj_q {
        for (e_entity, e_tf, mut enemy, sprite) in &mut enemy_q {
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (p_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < 5.0 + e_size.x / 2.0 && dist.y < 3.0 + e_size.y / 2.0 {
                enemy.health -= proj.damage + stats.attack;
                commands.entity(p_entity).try_despawn_recursive();

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    run.enemies_killed += 1;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).try_despawn_recursive();

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
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy, &Sprite)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    for (p_entity, p_tf, proj) in &proj_q {
        for (e_entity, e_tf, mut enemy, sprite) in &mut enemy_q {
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (p_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < 8.0 + e_size.x / 2.0 && dist.y < 8.0 + e_size.y / 2.0 {
                enemy.health -= proj.damage + stats.attack;
                commands.entity(p_entity).despawn();

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    run.enemies_killed += 1;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).try_despawn_recursive();

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
    mut enemy_q: Query<(Entity, &Transform, &mut Enemy)>,
    mut ev_defeated: EventWriter<EnemyDefeated>,
    mut run: ResMut<RunData>,
    mut shake_q: Query<&mut ScreenShake>,
) {
    for (s_tf, strike) in &strike_q {
        if strike.lifetime < 0.12 { continue; }

        for (e_entity, e_tf, mut enemy) in &mut enemy_q {
            let dist = (s_tf.translation.xy() - e_tf.translation.xy()).length();

            if dist < strike.radius {
                enemy.health -= strike.damage;

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    run.enemies_killed += 1;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).try_despawn_recursive();
                }
            }
        }

        if let Ok(mut shake) = shake_q.get_single_mut() {
            trigger_shake(&mut shake, 12.0, 0.25);
        }
    }
}

fn player_vs_enemy(
    mut player_q: Query<(&Transform, &mut Player)>,
    enemy_q: Query<(&Transform, &Enemy, &Sprite), Without<Player>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    mut ev_blocked: EventWriter<PlayerBlocked>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (e_tf, enemy, sprite) in &enemy_q {
        let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
        let diff = p_tf.translation.xy() - e_tf.translation.xy();
        let dist = diff.abs();

        if dist.x < 10.0 + e_size.x / 2.0 && dist.y < 16.0 + e_size.y / 2.0 {
            if player.invulnerable <= 0.0 {
                let raw_dmg = (enemy.contact_damage - stats.defense).max(1);
                let reduced = if player.is_blocking {
                    ev_blocked.send(PlayerBlocked { position: p_tf.translation });
                    ((raw_dmg as f32) * (1.0 - BLOCK_DAMAGE_REDUCTION)).ceil() as i32
                } else {
                    raw_dmg
                }.max(0);
                player.health -= reduced;
                player.invulnerable = INVULN_TIME;
                ev_damaged.send(PlayerDamaged {
                    amount: reduced,
                    remaining: player.health,
                });

                if let Ok(mut shake) = shake_q.get_single_mut() {
                    let intensity = if player.is_blocking { 6.0 } else { 14.0 };
                    trigger_shake(&mut shake, intensity, 0.3);
                }

                let kb_dir = if diff.x >= 0.0 { 1.0 } else { -1.0 };
                let kb_mult = if player.is_blocking { 0.5 } else { 1.0 };
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
    proj_q: Query<(Entity, &Transform), With<EnemyProjectile>>,
    mut player_q: Query<(&Transform, &mut Player), Without<EnemyProjectile>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    mut ev_blocked: EventWriter<PlayerBlocked>,
    mut shake_q: Query<&mut ScreenShake>,
    stats: Res<PlayerStats>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (proj_entity, proj_tf) in &proj_q {
        let dist = (p_tf.translation.xy() - proj_tf.translation.xy()).abs();
        if dist.x < 14.0 && dist.y < 20.0 {
            commands.entity(proj_entity).despawn();
            if player.invulnerable <= 0.0 {
                let raw_dmg = (1 - stats.defense).max(1);
                let dmg = if player.is_blocking {
                    ev_blocked.send(PlayerBlocked { position: p_tf.translation });
                    ((raw_dmg as f32) * (1.0 - BLOCK_DAMAGE_REDUCTION)).ceil() as i32
                } else {
                    raw_dmg
                }.max(0);
                player.health -= dmg;
                player.invulnerable = INVULN_TIME;
                ev_damaged.send(PlayerDamaged {
                    amount: dmg,
                    remaining: player.health,
                });
                if let Ok(mut shake) = shake_q.get_single_mut() {
                    let intensity = if player.is_blocking { 4.0 } else { 10.0 };
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
