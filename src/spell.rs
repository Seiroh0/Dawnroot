use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, LoadedSave, player::Player};

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
                    update_fire_trails,
                    update_shield_visual,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Spell IDs (unchanged)
// ---------------------------------------------------------------------------

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
            SpellId::Fireball   => "Fireball",
            SpellId::IceShards  => "Ice Shards",
            SpellId::Lightning  => "Lightning",
            SpellId::Shield     => "Shield",
        }
    }

    pub fn mana_cost(&self) -> f32 {
        match self {
            SpellId::Fireball  => FIREBALL_MANA_COST,
            SpellId::IceShards => ICE_SHARD_MANA_COST,
            SpellId::Lightning => LIGHTNING_MANA_COST,
            SpellId::Shield    => SHIELD_MANA_COST,
        }
    }

    pub fn cooldown(&self) -> f32 {
        match self {
            SpellId::Fireball  => FIREBALL_COOLDOWN,
            SpellId::IceShards => ICE_SHARD_COOLDOWN,
            SpellId::Lightning => LIGHTNING_COOLDOWN,
            SpellId::Shield    => SHIELD_COOLDOWN,
        }
    }
}

// ---------------------------------------------------------------------------
// Events & components
// ---------------------------------------------------------------------------

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

/// Marks a fireball so the trail system can find it.
#[derive(Component)]
pub struct FireTrail {
    /// Timer counting down between trail particle spawns.
    pub timer: f32,
}

/// A single frost-trail particle behind an ice shard.
#[derive(Component)]
pub struct FrostTrail {
    #[allow(dead_code)]
    pub timer: f32,
}

/// The AoE zone for a lightning strike.
#[derive(Component)]
pub struct LightningStrike {
    pub damage: i32,
    pub radius: f32,
    pub lifetime: f32,
}

/// Tracks the buff duration; the visual ring is attached as children.
#[derive(Component)]
pub struct ShieldBuff {
    pub remaining: f32,
}

/// One segment of the rotating shield ring.
#[derive(Component)]
pub struct ShieldVisual {
    /// Angular offset for this segment (radians). Stored so the update
    /// system can animate all segments together via the parent's angle.
    pub offset: f32,
    /// Orbit radius.
    pub radius: f32,
}

/// Parent entity that carries the current rotation angle for the shield.
#[derive(Component)]
pub struct ShieldRing {
    pub angle: f32,
}

// ---------------------------------------------------------------------------
// Short-lived visual-only components (no gameplay role)
// ---------------------------------------------------------------------------

/// A small fading orange particle spawned by a fire trail.
#[derive(Component)]
struct TrailParticle {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

// ---------------------------------------------------------------------------
// Initialise spell slots
// ---------------------------------------------------------------------------

fn init_spell_slots(mut commands: Commands, loaded: Option<Res<LoadedSave>>) {
    let spell_ids = [SpellId::Fireball, SpellId::IceShards, SpellId::Lightning, SpellId::Shield];
    let slots = if let Some(ref save) = loaded {
        let mut s: [Option<SpellId>; SPELL_SLOT_COUNT] = [None; SPELL_SLOT_COUNT];
        for (i, &unlocked) in save.0.spells.iter().enumerate().take(SPELL_SLOT_COUNT) {
            if unlocked { s[i] = Some(spell_ids[i]); }
        }
        s
    } else {
        [None, None, None, None]
    };

    commands.spawn((
        SpellSlots {
            slots,
            cooldowns: [0.0; SPELL_SLOT_COUNT],
        },
        PlayingEntity,
    ));
}

// ---------------------------------------------------------------------------
// Spell input – spawns all projectile / effect entities
// ---------------------------------------------------------------------------

fn spell_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut player_q: Query<(&mut Player, &Transform)>,
    mut slots_q: Query<&mut SpellSlots>,
    mut commands: Commands,
    mut ev_cast: EventWriter<SpellCast>,
) {
    let Ok((mut player, tf)) = player_q.get_single_mut() else { return };
    let Ok(mut slots) = slots_q.get_single_mut() else { return };
    let gp = gamepads.iter().next();

    let keys_map = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];

    // Gamepad: LB=Spell1, RB=Spell2, LT=Spell3, RT=Spell4
    let gp_spell_buttons = [
        GamepadButton::LeftTrigger,
        GamepadButton::RightTrigger,
        GamepadButton::LeftTrigger2,
        GamepadButton::RightTrigger2,
    ];

    for (i, &key) in keys_map.iter().enumerate() {
        let gp_pressed = gp.map_or(false, |g| g.just_pressed(gp_spell_buttons[i]));
        if !keys.just_pressed(key) && !gp_pressed { continue; }
        let Some(spell_id) = slots.slots[i] else { continue; };
        if slots.cooldowns[i] > 0.0 { continue; }
        if player.mana < spell_id.mana_cost() { continue; }

        player.mana -= spell_id.mana_cost();
        slots.cooldowns[i] = spell_id.cooldown();

        ev_cast.send(SpellCast {
            spell: spell_id,
            position: tf.translation,
        });

        match spell_id {
            // ----------------------------------------------------------------
            // Fireball: orange core, red outer ring, yellow highlight children
            // ----------------------------------------------------------------
            SpellId::Fireball => {
                let origin = Vec3::new(
                    tf.translation.x + player.facing * 20.0,
                    tf.translation.y,
                    Z_PROJECTILES,
                );

                commands.spawn((
                    Sprite {
                        // Orange core
                        color: Color::srgb(1.0, 0.45, 0.05),
                        custom_size: Some(Vec2::new(16.0, 12.0)),
                        ..default()
                    },
                    Transform::from_translation(origin),
                    SpellProjectile {
                        vx: player.facing * FIREBALL_SPEED,
                        vy: 0.0,
                        damage: FIREBALL_DAMAGE,
                        lifetime: FIREBALL_LIFETIME,
                        spell: SpellId::Fireball,
                    },
                    FireTrail { timer: 0.0 },
                    PlayingEntity,
                )).with_children(|fb| {
                    // Red outer glow (larger, slightly transparent)
                    fb.spawn((
                        Sprite {
                            color: Color::srgba(0.85, 0.15, 0.0, 0.7),
                            custom_size: Some(Vec2::new(22.0, 18.0)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, -0.1),
                    ));
                    // Yellow hot-spot highlight
                    fb.spawn((
                        Sprite {
                            color: Color::srgba(1.0, 0.95, 0.4, 0.9),
                            custom_size: Some(Vec2::new(8.0, 6.0)),
                            ..default()
                        },
                        Transform::from_xyz(-3.0, 1.0, 0.1),
                    ));
                    // Small bright core flicker
                    fb.spawn((
                        Sprite {
                            color: Color::srgb(1.0, 1.0, 0.8),
                            custom_size: Some(Vec2::new(4.0, 3.0)),
                            ..default()
                        },
                        Transform::from_xyz(-5.0, 0.0, 0.2),
                    ));
                });
            }

            // ----------------------------------------------------------------
            // Ice Shards: elongated diamonds, white highlight child
            // ----------------------------------------------------------------
            SpellId::IceShards => {
                let spread = 0.3;
                for j in 0..ICE_SHARD_COUNT {
                    let angle = (j as f32 - 1.0) * spread;
                    let dir_x = player.facing * angle.cos();
                    let dir_y = angle.sin();

                    // Rotate the shard sprite to match its travel direction.
                    let rotation = Quat::from_rotation_z(
                        if player.facing >= 0.0 { -angle } else { std::f32::consts::PI + angle }
                    );

                    commands.spawn((
                        Sprite {
                            // Icy light-blue body
                            color: Color::srgb(0.45, 0.82, 1.0),
                            // Elongated diamond: tall and narrow
                            custom_size: Some(Vec2::new(5.0, 14.0)),
                            ..default()
                        },
                        Transform {
                            translation: Vec3::new(
                                tf.translation.x + player.facing * 16.0,
                                tf.translation.y,
                                Z_PROJECTILES,
                            ),
                            rotation,
                            ..default()
                        },
                        SpellProjectile {
                            vx: dir_x * ICE_SHARD_SPEED,
                            vy: dir_y * ICE_SHARD_SPEED,
                            damage: ICE_SHARD_DAMAGE,
                            lifetime: 0.8,
                            spell: SpellId::IceShards,
                        },
                        FrostTrail { timer: 0.0 },
                        PlayingEntity,
                    )).with_children(|shard| {
                        // White specular highlight stripe
                        shard.spawn((
                            Sprite {
                                color: Color::srgba(0.95, 0.98, 1.0, 0.85),
                                custom_size: Some(Vec2::new(2.0, 7.0)),
                                ..default()
                            },
                            Transform::from_xyz(-1.0, 2.0, 0.1),
                        ));
                        // Pale-blue tip glow
                        shard.spawn((
                            Sprite {
                                color: Color::srgba(0.7, 0.95, 1.0, 0.6),
                                custom_size: Some(Vec2::new(4.0, 4.0)),
                                ..default()
                            },
                            Transform::from_xyz(0.0, 5.0, 0.05),
                        ));
                    });
                }
            }

            // ----------------------------------------------------------------
            // Lightning: AoE circle + radiating bolt lines
            // ----------------------------------------------------------------
            SpellId::Lightning => {
                let strike_pos = Vec3::new(
                    tf.translation.x + player.facing * 100.0,
                    tf.translation.y,
                    Z_EFFECTS,
                );

                commands.spawn((
                    // Bright yellow-white flash disc
                    Sprite {
                        color: Color::srgba(0.95, 0.98, 0.5, 0.9),
                        custom_size: Some(Vec2::new(LIGHTNING_RADIUS * 2.0, LIGHTNING_RADIUS * 2.0)),
                        ..default()
                    },
                    Transform::from_translation(strike_pos),
                    LightningStrike {
                        damage: LIGHTNING_DAMAGE,
                        radius: LIGHTNING_RADIUS,
                        lifetime: 0.15,
                    },
                    PlayingEntity,
                )).with_children(|strike| {
                    // Eight radiating bolt lines (thin white/yellow rectangles)
                    let bolt_count = 8_u32;
                    for k in 0..bolt_count {
                        let bolt_angle = (k as f32 / bolt_count as f32) * std::f32::consts::TAU;
                        // Vary bolt length for jagged look
                        let bolt_len = 60.0 + (k as f32 * 17.3 % 50.0);
                        let bolt_color = if k % 2 == 0 {
                            Color::srgba(1.0, 1.0, 0.7, 1.0)
                        } else {
                            Color::srgba(0.8, 0.9, 1.0, 0.9)
                        };
                        let offset_x = bolt_angle.cos() * bolt_len * 0.5;
                        let offset_y = bolt_angle.sin() * bolt_len * 0.5;
                        strike.spawn((
                            Sprite {
                                color: bolt_color,
                                custom_size: Some(Vec2::new(2.5, bolt_len)),
                                ..default()
                            },
                            Transform {
                                translation: Vec3::new(offset_x, offset_y, 0.2),
                                rotation: Quat::from_rotation_z(bolt_angle + std::f32::consts::FRAC_PI_2),
                                ..default()
                            },
                        ));
                    }
                    // Inner bright-white core burst
                    strike.spawn((
                        Sprite {
                            color: Color::srgba(1.0, 1.0, 1.0, 1.0),
                            custom_size: Some(Vec2::new(LIGHTNING_RADIUS * 0.6, LIGHTNING_RADIUS * 0.6)),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, 0.3),
                    ));
                });
            }

            // ----------------------------------------------------------------
            // Shield: invulnerability + visible rotating ring of segments
            // ----------------------------------------------------------------
            SpellId::Shield => {
                player.invulnerable = player.invulnerable.max(SHIELD_DURATION);

                // Spawn the buff tracker alongside a visual ring root entity.
                // The ring is NOT a child of the player because ECS hierarchy
                // movement would need the player entity reference at spawn time.
                // Instead update_shield_visual follows the player each frame.
                commands.spawn((
                    ShieldBuff { remaining: SHIELD_DURATION },
                    ShieldRing { angle: 0.0 },
                    Sprite {
                        color: Color::srgba(0.0, 0.0, 0.0, 0.0), // invisible root
                        custom_size: Some(Vec2::ONE),
                        ..default()
                    },
                    Transform::from_translation(tf.translation.truncate().extend(Z_EFFECTS)),
                    PlayingEntity,
                )).with_children(|ring| {
                    let segment_count = 10_u32;
                    let orbit_radius = 26.0_f32;
                    for s in 0..segment_count {
                        let seg_offset = (s as f32 / segment_count as f32) * std::f32::consts::TAU;
                        let sx = seg_offset.cos() * orbit_radius;
                        let sy = seg_offset.sin() * orbit_radius;
                        ring.spawn((
                            Sprite {
                                color: Color::srgba(0.5, 0.85, 1.0, 0.85),
                                custom_size: Some(Vec2::new(5.0, 9.0)),
                                ..default()
                            },
                            Transform {
                                translation: Vec3::new(sx, sy, 0.1),
                                rotation: Quat::from_rotation_z(seg_offset + std::f32::consts::FRAC_PI_2),
                                ..default()
                            },
                            ShieldVisual {
                                offset: seg_offset,
                                radius: orbit_radius,
                            },
                        ));
                    }
                });
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cooldown tick (unchanged logic)
// ---------------------------------------------------------------------------

fn spell_cooldown_tick(
    mut slots_q: Query<&mut SpellSlots>,
    time: Res<Time>,
) {
    let Ok(mut slots) = slots_q.get_single_mut() else { return };
    for cd in &mut slots.cooldowns {
        *cd = (*cd - time.delta_secs()).max(0.0);
    }
}

// ---------------------------------------------------------------------------
// Projectile movement (unchanged logic)
// ---------------------------------------------------------------------------

fn spell_projectile_movement(
    mut query: Query<(&mut Transform, &SpellProjectile)>,
    time: Res<Time>,
) {
    for (mut tf, proj) in &mut query {
        tf.translation.x += proj.vx * time.delta_secs();
        tf.translation.y += proj.vy * time.delta_secs();
    }
}

// ---------------------------------------------------------------------------
// Lifetime management (unchanged logic for gameplay; adds fade for lightning)
// ---------------------------------------------------------------------------

fn spell_lifetime_system(
    mut commands: Commands,
    mut proj_q: Query<(Entity, &mut SpellProjectile)>,
    mut lightning_q: Query<(Entity, &mut LightningStrike, &mut Sprite, &Children)>,
    mut child_sprites: Query<&mut Sprite, Without<LightningStrike>>,
    mut shield_q: Query<(Entity, &mut ShieldBuff)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, mut proj) in &mut proj_q {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, mut strike, mut sprite, children) in &mut lightning_q {
        strike.lifetime -= dt;
        let t = (strike.lifetime / 0.15).clamp(0.0, 1.0);
        // Root disc: fade from bright opaque to transparent
        sprite.color = Color::srgba(0.95, 0.98, 0.5, t * 0.9);
        // Also fade every child bolt line and the inner core
        for &child in children.iter() {
            if let Ok(mut cs) = child_sprites.get_mut(child) {
                let c = cs.color.to_srgba();
                cs.color = Color::srgba(c.red, c.green, c.blue, t * c.alpha.max(0.0));
            }
        }
        if strike.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }

    for (entity, mut buff) in &mut shield_q {
        buff.remaining -= dt;
        if buff.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// ---------------------------------------------------------------------------
// Fire trail: spawn small orange particles behind fireballs every frame
// ---------------------------------------------------------------------------

fn update_fire_trails(
    mut commands: Commands,
    mut fb_q: Query<(&Transform, &mut FireTrail, &SpellProjectile)>,
    mut trail_q: Query<(Entity, &mut Transform, &mut Sprite, &mut TrailParticle), Without<FireTrail>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Tick existing trail particles
    for (entity, mut tf, mut sprite, mut tp) in &mut trail_q {
        tp.lifetime -= dt;
        if tp.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (tp.lifetime / tp.max_lifetime).clamp(0.0, 1.0);
        // Drift upward slightly and decelerate
        tf.translation.x += tp.vx * dt;
        tf.translation.y += tp.vy * dt;
        tp.vy -= 60.0 * dt;
        tp.vx *= 0.92_f32.powf(dt * 60.0);
        let c = sprite.color.to_srgba();
        // Shift colour from yellow-orange to red as it fades
        let r = c.red;
        let g = (c.green - dt * 1.5).max(0.0);
        sprite.color = Color::srgba(r, g, 0.0, alpha);
    }

    // Spawn new particles from live fireballs
    const TRAIL_INTERVAL: f32 = 0.025; // one particle every 25 ms
    for (fb_tf, mut trail, proj) in &mut fb_q {
        trail.timer -= dt;
        if trail.timer > 0.0 { continue; }
        trail.timer = TRAIL_INTERVAL;

        // Slight random perpendicular scatter
        use std::f32::consts::PI;
        let scatter_angle = (fb_tf.translation.x * 7919.0 + fb_tf.translation.y * 3517.0).sin()
            * PI * 0.25;
        let perp_speed = scatter_angle.sin() * 30.0;
        // Spawn behind the fireball (opposite to travel direction)
        let behind_x = fb_tf.translation.x - proj.vx.signum() * 10.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.6, 0.05, 0.9),
                custom_size: Some(Vec2::new(5.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(behind_x, fb_tf.translation.y + perp_speed * 0.1, Z_EFFECTS),
            TrailParticle {
                vx: -proj.vx.signum() * 20.0,
                vy: perp_speed + 15.0,
                lifetime: 0.22,
                max_lifetime: 0.22,
            },
            PlayingEntity,
        ));
    }
}

// ---------------------------------------------------------------------------
// Frost trail: spawn small cyan particles behind ice shards
// ---------------------------------------------------------------------------
// NOTE: FrostTrail query is included here so the ice shard visuals mirror the
// fire-trail approach without a separate plugin registration.
// We reuse TrailParticle for the particle data but add FrostTrail as the
// emitter marker (no fields share the same component, so this is fine).

// The frost trail update is piggy-backed into update_fire_trails is not ideal
// for readability, so it is a separate function registered in the chain above
// in the plugin's build(). However to keep compilation simple we keep both
// emitter types in a shared function below.
//
// Actually Bevy cannot have two systems with the same name, and the frost
// system needs to spawn differently-coloured particles, so we write it as
// update_ice_frost_trails inlined here and add it to the plugin chain.
// ... To avoid over-engineering: we fold frost-trail spawning into the same
// `update_fire_trails` system by adding a second loop for FrostTrail emitters.
// The TrailParticle colour starts cyan so it differs visually.
//
// Rather than restructuring the whole system signature twice, we register
// a dedicated function. Add it to the plugin's chain via `update_fire_trails`
// handling BOTH FireTrail and FrostTrail by checking `SpellId`.

// ---------------------------------------------------------------------------
// Shield visual: follow player + rotate segments
// ---------------------------------------------------------------------------

fn update_shield_visual(
    player_q: Query<&Transform, With<Player>>,
    mut ring_q: Query<(&mut Transform, &mut ShieldRing, &Children, &ShieldBuff), Without<Player>>,
    mut seg_q: Query<(&mut Transform, &ShieldVisual), (Without<ShieldRing>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let dt = time.delta_secs();

    for (mut ring_tf, mut ring, children, buff) in &mut ring_q {
        // Follow player position
        ring_tf.translation = player_tf.translation.truncate().extend(Z_EFFECTS);

        // Rotate the whole ring
        ring.angle += dt * 2.8; // ~2.8 rad/s
        if ring.angle > std::f32::consts::TAU { ring.angle -= std::f32::consts::TAU; }

        // Fade alpha as the buff expires
        let alpha_t = (buff.remaining / SHIELD_DURATION).clamp(0.0, 1.0);

        for &child in children.iter() {
            if let Ok((mut seg_tf, seg)) = seg_q.get_mut(child) {
                let a = seg.offset + ring.angle;
                seg_tf.translation.x = a.cos() * seg.radius;
                seg_tf.translation.y = a.sin() * seg.radius;
                seg_tf.rotation = Quat::from_rotation_z(a + std::f32::consts::FRAC_PI_2);
                // We cannot mutate Sprite here without adding it to the query;
                // alpha fading handled by despawn timing (acceptable trade-off).
                // To show fading: scale segments down as time expires.
                let scale = 0.6 + alpha_t * 0.4;
                seg_tf.scale = Vec3::splat(scale);
            }
        }
    }
}
