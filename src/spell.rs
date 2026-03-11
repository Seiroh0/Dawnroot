use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, player::Player};

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpellCast>()
            .add_systems(OnEnter(GameState::Playing), init_spell_slots)
            .add_systems(
                Update,
                (
                    spell_input,
                    spell_cooldown_tick,
                    spell_projectile_movement,
                    spell_lifetime_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpellId {
    Fireball,
    IceShards,
    Lightning,
    Shield,
}

impl SpellId {
    pub fn name(&self) -> &str {
        match self {
            SpellId::Fireball => "Fireball",
            SpellId::IceShards => "Ice Shards",
            SpellId::Lightning => "Lightning",
            SpellId::Shield => "Shield",
        }
    }

    pub fn mana_cost(&self) -> f32 {
        match self {
            SpellId::Fireball => FIREBALL_MANA_COST,
            SpellId::IceShards => ICE_SHARD_MANA_COST,
            SpellId::Lightning => LIGHTNING_MANA_COST,
            SpellId::Shield => SHIELD_MANA_COST,
        }
    }

    pub fn cooldown(&self) -> f32 {
        match self {
            SpellId::Fireball => FIREBALL_COOLDOWN,
            SpellId::IceShards => ICE_SHARD_COOLDOWN,
            SpellId::Lightning => LIGHTNING_COOLDOWN,
            SpellId::Shield => SHIELD_COOLDOWN,
        }
    }
}

#[derive(Event)]
#[allow(dead_code)]
pub struct SpellCast {
    pub spell: SpellId,
    pub position: Vec3,
}

#[derive(Component)]
pub struct SpellSlots {
    pub slots: [Option<SpellId>; SPELL_SLOT_COUNT],
    pub cooldowns: [f32; SPELL_SLOT_COUNT],
}

#[derive(Component)]
#[allow(dead_code)]
pub struct SpellProjectile {
    pub vx: f32,
    pub vy: f32,
    pub damage: i32,
    pub lifetime: f32,
    pub spell: SpellId,
}

#[derive(Component)]
pub struct ShieldBuff {
    pub remaining: f32,
}

fn init_spell_slots(mut commands: Commands) {
    commands.spawn((
        SpellSlots {
            slots: [
                Some(SpellId::Fireball),
                Some(SpellId::IceShards),
                Some(SpellId::Lightning),
                Some(SpellId::Shield),
            ],
            cooldowns: [0.0; SPELL_SLOT_COUNT],
        },
        PlayingEntity,
    ));
}

fn spell_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&mut Player, &Transform)>,
    mut slots_q: Query<&mut SpellSlots>,
    mut commands: Commands,
    mut ev_cast: EventWriter<SpellCast>,
) {
    let Ok((mut player, tf)) = player_q.get_single_mut() else { return };
    let Ok(mut slots) = slots_q.get_single_mut() else { return };

    let keys_map = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];

    for (i, &key) in keys_map.iter().enumerate() {
        if !keys.just_pressed(key) { continue; }
        let Some(spell_id) = slots.slots[i] else { continue; };
        if slots.cooldowns[i] > 0.0 { continue; }
        if player.mana < spell_id.mana_cost() { continue; }

        // Cast spell
        player.mana -= spell_id.mana_cost();
        slots.cooldowns[i] = spell_id.cooldown();

        ev_cast.send(SpellCast {
            spell: spell_id,
            position: tf.translation,
        });

        match spell_id {
            SpellId::Fireball => {
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 0.4, 0.1),
                        custom_size: Some(Vec2::new(16.0, 12.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        tf.translation.x + player.facing * 20.0,
                        tf.translation.y,
                        Z_PROJECTILES,
                    ),
                    SpellProjectile {
                        vx: player.facing * FIREBALL_SPEED,
                        vy: 0.0,
                        damage: FIREBALL_DAMAGE,
                        lifetime: FIREBALL_LIFETIME,
                        spell: SpellId::Fireball,
                    },
                    PlayingEntity,
                ));
            }
            SpellId::IceShards => {
                let spread = 0.3;
                for j in 0..ICE_SHARD_COUNT {
                    let angle = (j as f32 - 1.0) * spread;
                    let dir_x = player.facing * angle.cos();
                    let dir_y = angle.sin();
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.5, 0.8, 1.0),
                            custom_size: Some(Vec2::new(8.0, 8.0)),
                            ..default()
                        },
                        Transform::from_xyz(
                            tf.translation.x + player.facing * 16.0,
                            tf.translation.y,
                            Z_PROJECTILES,
                        ),
                        SpellProjectile {
                            vx: dir_x * ICE_SHARD_SPEED,
                            vy: dir_y * ICE_SHARD_SPEED,
                            damage: ICE_SHARD_DAMAGE,
                            lifetime: 0.8,
                            spell: SpellId::IceShards,
                        },
                        PlayingEntity,
                    ));
                }
            }
            SpellId::Lightning => {
                // Lightning: AoE damage around cursor position (use player pos for now)
                // The actual damage is handled in combat.rs via LightningStrike
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.9, 0.9, 0.3, 0.8),
                        custom_size: Some(Vec2::new(LIGHTNING_RADIUS * 2.0, LIGHTNING_RADIUS * 2.0)),
                        ..default()
                    },
                    Transform::from_xyz(
                        tf.translation.x + player.facing * 100.0,
                        tf.translation.y,
                        Z_EFFECTS,
                    ),
                    LightningStrike {
                        damage: LIGHTNING_DAMAGE,
                        radius: LIGHTNING_RADIUS,
                        lifetime: 0.15,
                    },
                    PlayingEntity,
                ));
            }
            SpellId::Shield => {
                player.invulnerable = player.invulnerable.max(SHIELD_DURATION);
                commands.spawn((
                    ShieldBuff { remaining: SHIELD_DURATION },
                    PlayingEntity,
                ));
            }
        }
    }
}

#[derive(Component)]
pub struct LightningStrike {
    pub damage: i32,
    pub radius: f32,
    pub lifetime: f32,
}

fn spell_cooldown_tick(
    mut slots_q: Query<&mut SpellSlots>,
    time: Res<Time>,
) {
    let Ok(mut slots) = slots_q.get_single_mut() else { return };
    for cd in &mut slots.cooldowns {
        *cd = (*cd - time.delta_secs()).max(0.0);
    }
}

fn spell_projectile_movement(
    mut query: Query<(&mut Transform, &SpellProjectile)>,
    time: Res<Time>,
) {
    for (mut tf, proj) in &mut query {
        tf.translation.x += proj.vx * time.delta_secs();
        tf.translation.y += proj.vy * time.delta_secs();
    }
}

fn spell_lifetime_system(
    mut commands: Commands,
    mut proj_q: Query<(Entity, &mut SpellProjectile)>,
    mut lightning_q: Query<(Entity, &mut LightningStrike, &mut Sprite)>,
    mut shield_q: Query<(Entity, &mut ShieldBuff)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut proj) in &mut proj_q {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut strike, mut sprite) in &mut lightning_q {
        strike.lifetime -= dt;
        let alpha = (strike.lifetime / 0.15).clamp(0.0, 0.8);
        sprite.color = Color::srgba(0.9, 0.9, 0.3, alpha);
        if strike.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut buff) in &mut shield_q {
        buff.remaining -= dt;
        if buff.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
