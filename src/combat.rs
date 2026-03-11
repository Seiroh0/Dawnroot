use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, RunData,
    player::{Player, MeleeHitbox, PlayerDamaged, PlayerDied},
    enemy::{Enemy, EnemyDefeated, EnemyProjectile},
    spell::{SpellProjectile, LightningStrike},
    camera::{ScreenShake, trigger_shake},
};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                melee_vs_enemy,
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
) {
    for (h_tf, hitbox) in &hitbox_q {
        for (e_entity, e_tf, mut enemy, sprite) in &mut enemy_q {
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (h_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < MELEE_RANGE / 2.0 + e_size.x / 2.0
                && dist.y < MELEE_WIDTH / 2.0 + e_size.y / 2.0
            {
                enemy.health -= hitbox.damage;

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).despawn_recursive();

                    if let Ok(mut shake) = shake_q.get_single_mut() {
                        trigger_shake(&mut shake, 8.0, 0.15);
                    }
                }
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
) {
    for (p_entity, p_tf, proj) in &proj_q {
        for (e_entity, e_tf, mut enemy, sprite) in &mut enemy_q {
            let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
            let dist = (p_tf.translation.xy() - e_tf.translation.xy()).abs();

            if dist.x < 8.0 + e_size.x / 2.0 && dist.y < 8.0 + e_size.y / 2.0 {
                enemy.health -= proj.damage;
                commands.entity(p_entity).despawn();

                if enemy.health <= 0 {
                    run.score += enemy.score_reward;
                    run.gold += enemy.gold_drop;
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).despawn_recursive();

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
                    ev_defeated.send(EnemyDefeated {
                        position: e_tf.translation,
                        score: enemy.score_reward,
                        gold_drop: enemy.gold_drop,
                    });
                    commands.entity(e_entity).despawn_recursive();
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
    mut shake_q: Query<&mut ScreenShake>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (e_tf, enemy, sprite) in &enemy_q {
        let e_size = sprite.custom_size.unwrap_or(Vec2::new(20.0, 20.0));
        let diff = p_tf.translation.xy() - e_tf.translation.xy();
        let dist = diff.abs();

        if dist.x < 10.0 + e_size.x / 2.0 && dist.y < 16.0 + e_size.y / 2.0 {
            if player.invulnerable <= 0.0 {
                player.health -= enemy.contact_damage;
                player.invulnerable = INVULN_TIME;
                ev_damaged.send(PlayerDamaged {
                    amount: enemy.contact_damage,
                    remaining: player.health,
                });

                if let Ok(mut shake) = shake_q.get_single_mut() {
                    trigger_shake(&mut shake, 14.0, 0.3);
                }

                let kb_dir = if diff.x >= 0.0 { 1.0 } else { -1.0 };
                player.vx = kb_dir * 250.0;
                player.vy = 200.0;

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
    mut shake_q: Query<&mut ScreenShake>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (proj_entity, proj_tf) in &proj_q {
        let dist = (p_tf.translation.xy() - proj_tf.translation.xy()).abs();
        if dist.x < 14.0 && dist.y < 20.0 {
            commands.entity(proj_entity).despawn();
            if player.invulnerable <= 0.0 {
                player.health -= 1;
                player.invulnerable = INVULN_TIME;
                ev_damaged.send(PlayerDamaged {
                    amount: 1,
                    remaining: player.health,
                });
                if let Ok(mut shake) = shake_q.get_single_mut() {
                    trigger_shake(&mut shake, 10.0, 0.2);
                }
                if player.health <= 0 {
                    ev_died.send(PlayerDied);
                }
            }
            break;
        }
    }
}
