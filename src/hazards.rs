use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, PlayingEntity,
    player::{Player, PlayerDamaged, PlayerDied},
    enemy::EnemyProjectile,
    room::{Tile, RoomEntity},
};

pub struct HazardsPlugin;

impl Plugin for HazardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                damage_zone_tick,
                movement_modifier_apply,
                kinematic_platform_move,
                kinematic_platform_carry,
                arrow_trap_tick,
                spike_floor_tick,
                poison_cloud_tick,
                animate_lava,
                animate_water,
                animate_poison,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ─── Components ──────────────────────────────────────────────────────────────

/// A zone that deals damage over time to the player on contact (e.g. lava).
#[derive(Component)]
pub struct DamageZone {
    pub damage: i32,
    pub tick_rate: f32,
    pub tick_timer: f32,
}

/// A zone that slows entities inside it (e.g. water, mud).
#[derive(Component)]
pub struct MovementModifier {
    pub speed_multiplier: f32,
}

/// A platform that moves between waypoints.
#[derive(Component)]
pub struct KinematicPlatform {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
    pub speed: f32,
    pub wait_timer: f32,
    pub wait_duration: f32,
    /// Previous frame position, used to compute delta for carrying entities.
    pub prev_position: Vec2,
}

/// Arrow trap: fires a projectile periodically from a wall.
#[derive(Component)]
pub struct ArrowTrap {
    pub fire_interval: f32,
    pub fire_timer: f32,
    pub direction: f32, // 1.0 = right, -1.0 = left
}

/// Spike floor: pops up periodically, dealing damage when raised.
#[derive(Component)]
pub struct SpikeFloor {
    pub cycle_timer: f32,
    pub cycle_duration: f32,
    pub raised: bool,
    pub raised_duration: f32,
}

/// Spike visual child that moves up/down.
#[derive(Component)]
pub struct SpikeVisual;

/// Poison cloud: DOT area that pulses in and out.
#[derive(Component)]
pub struct PoisonCloud {
    pub damage: i32,
    pub tick_rate: f32,
    pub tick_timer: f32,
    pub phase: f32,
}

/// Visual marker for lava animation.
#[derive(Component)]
pub struct LavaVisual {
    pub timer: f32,
}

/// Visual marker for water animation.
#[derive(Component)]
pub struct WaterVisual {
    pub timer: f32,
    pub phase: f32,
}

// ─── Damage Zone System ──────────────────────────────────────────────────────

fn damage_zone_tick(
    mut zone_q: Query<(&Transform, &Sprite, &mut DamageZone, Option<&SpikeFloor>)>,
    mut player_q: Query<(&Transform, &mut Player), Without<DamageZone>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (z_tf, z_sprite, mut zone, spike) in &mut zone_q {
        // Spike floors only damage when raised
        if let Some(sf) = spike {
            if !sf.raised { continue; }
        }

        zone.tick_timer -= dt;

        let z_size = z_sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let dist = (p_tf.translation.xy() - z_tf.translation.xy()).abs();

        // AABB overlap check (player half-size: 10x16)
        if dist.x < 10.0 + z_size.x / 2.0 - 4.0 && dist.y < 16.0 + z_size.y / 2.0 - 4.0 {
            if zone.tick_timer <= 0.0 && player.invulnerable <= 0.0 {
                zone.tick_timer = zone.tick_rate;
                player.health -= zone.damage;
                player.invulnerable = 0.3;
                ev_damaged.send(PlayerDamaged {
                    amount: zone.damage,
                    remaining: player.health,
                });
                player.vy = 300.0;

                if player.health <= 0 {
                    ev_died.send(PlayerDied);
                }
            }
        }
    }
}

// ─── Movement Modifier System ────────────────────────────────────────────────

fn movement_modifier_apply(
    zone_q: Query<(&Transform, &Sprite, &MovementModifier)>,
    mut player_q: Query<(&Transform, &mut Player), Without<MovementModifier>>,
) {
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    let mut in_zone = false;
    for (z_tf, z_sprite, modifier) in &zone_q {
        let z_size = z_sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let dist = (p_tf.translation.xy() - z_tf.translation.xy()).abs();

        if dist.x < 10.0 + z_size.x / 2.0 - 4.0 && dist.y < 16.0 + z_size.y / 2.0 - 4.0 {
            player.vx *= modifier.speed_multiplier;
            in_zone = true;
            break;
        }
    }
    let _ = in_zone;
}

// ─── Kinematic Platform System ───────────────────────────────────────────────

fn kinematic_platform_move(
    mut plat_q: Query<(&mut Transform, &mut KinematicPlatform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut tf, mut plat) in &mut plat_q {
        if plat.waypoints.len() < 2 { continue; }

        plat.prev_position = tf.translation.xy();

        // Waiting at waypoint
        if plat.wait_timer > 0.0 {
            plat.wait_timer -= dt;
            continue;
        }

        let target = plat.waypoints[plat.current_index];
        let current = tf.translation.xy();
        let diff = target - current;
        let dist = diff.length();

        if dist < 2.0 {
            // Arrived at waypoint
            tf.translation.x = target.x;
            tf.translation.y = target.y;
            plat.current_index = (plat.current_index + 1) % plat.waypoints.len();
            plat.wait_timer = plat.wait_duration;
        } else {
            let move_dist = plat.speed * dt;
            let dir = diff.normalize();
            tf.translation.x += dir.x * move_dist.min(dist);
            tf.translation.y += dir.y * move_dist.min(dist);
        }
    }
}

fn kinematic_platform_carry(
    plat_q: Query<(&Transform, &KinematicPlatform, &Sprite)>,
    mut player_q: Query<(&mut Transform, &Player), Without<KinematicPlatform>>,
) {
    let Ok((mut p_tf, player)) = player_q.get_single_mut() else { return };

    // Only carry when player is on floor (standing on something)
    if !player.is_on_floor { return; }

    for (plat_tf, plat, sprite) in &plat_q {
        let plat_size = sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let phw = plat_size.x / 2.0;
        let phh = plat_size.y / 2.0;

        let dx = (p_tf.translation.x - plat_tf.translation.x).abs();
        let dy = p_tf.translation.y - plat_tf.translation.y;

        // Player is on top of platform
        if dx < 10.0 + phw - 2.0 && dy > 0.0 && dy < 16.0 + phh + 4.0 {
            let delta = plat_tf.translation.xy() - plat.prev_position;
            p_tf.translation.x += delta.x;
            p_tf.translation.y += delta.y;
            break;
        }
    }
}

// ─── Visual Animations ───────────────────────────────────────────────────────

fn animate_lava(
    time: Res<Time>,
    mut query: Query<(&mut LavaVisual, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut lava, mut sprite) in &mut query {
        lava.timer += dt * 3.0;
        let pulse = (lava.timer).sin() * 0.15 + 0.85;
        let srgba = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            (srgba.red * pulse).min(1.0),
            srgba.green,
            srgba.blue,
            (0.85 + (lava.timer * 2.0).sin() * 0.1).clamp(0.7, 1.0),
        );
    }
}

fn animate_water(
    time: Res<Time>,
    mut query: Query<(&mut WaterVisual, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut water, mut sprite) in &mut query {
        water.timer += dt * 1.5;
        let wave = ((water.timer + water.phase) * 2.0).sin() * 0.08;
        let srgba = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            srgba.red,
            srgba.green,
            srgba.blue,
            (0.5 + wave).clamp(0.35, 0.65),
        );
    }
}

// ─── Arrow Trap System ──────────────────────────────────────────────────────

fn arrow_trap_tick(
    mut commands: Commands,
    mut trap_q: Query<(&Transform, &mut ArrowTrap)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (tf, mut trap) in &mut trap_q {
        trap.fire_timer -= dt;
        if trap.fire_timer <= 0.0 {
            trap.fire_timer = trap.fire_interval;
            let speed = 280.0;
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.6, 0.4, 0.2),
                    custom_size: Some(Vec2::new(10.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(tf.translation.x + trap.direction * 20.0, tf.translation.y, Z_PROJECTILES),
                EnemyProjectile {
                    vx: trap.direction * speed,
                    vy: 0.0,
                    lifetime: 2.0,
                    damage: 1,
                },
                RoomEntity,
                PlayingEntity,
            ));
        }
    }
}

// ─── Spike Floor System ─────────────────────────────────────────────────────

fn spike_floor_tick(
    spike_q: Query<(&SpikeFloor, &Children)>,
    mut vis_q: Query<&mut Transform, (With<SpikeVisual>, Without<SpikeFloor>)>,
    mut spike_state_q: Query<&mut SpikeFloor, Without<Player>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // Update timers
    for mut spike in &mut spike_state_q {
        spike.cycle_timer -= dt;
        if spike.cycle_timer <= 0.0 {
            spike.raised = !spike.raised;
            spike.cycle_timer = if spike.raised { spike.raised_duration } else { spike.cycle_duration };
        }
    }

    // Animate spike visuals up/down based on parent raised state
    for (spike, children) in &spike_q {
        let target_y = if spike.raised { 6.0 } else { -5.0 };
        for &child in children.iter() {
            if let Ok(mut vtf) = vis_q.get_mut(child) {
                vtf.translation.y += (target_y - vtf.translation.y) * dt * 10.0;
            }
        }
    }
}

// ─── Poison Cloud System ────────────────────────────────────────────────────

fn poison_cloud_tick(
    mut cloud_q: Query<(&Transform, &Sprite, &mut PoisonCloud)>,
    mut player_q: Query<(&Transform, &mut Player), Without<PoisonCloud>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (c_tf, c_sprite, mut cloud) in &mut cloud_q {
        cloud.tick_timer -= dt;
        cloud.phase += dt;

        let c_size = c_sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 2.0));
        let dist = (p_tf.translation.xy() - c_tf.translation.xy()).abs();

        if dist.x < 10.0 + c_size.x / 2.0 - 8.0 && dist.y < 16.0 + c_size.y / 2.0 - 8.0 {
            if cloud.tick_timer <= 0.0 && player.invulnerable <= 0.0 {
                cloud.tick_timer = cloud.tick_rate;
                player.health -= cloud.damage;
                player.invulnerable = 0.2;
                ev_damaged.send(PlayerDamaged { amount: cloud.damage, remaining: player.health });
                if player.health <= 0 {
                    ev_died.send(PlayerDied);
                }
            }
        }
    }
}

fn animate_poison(
    mut query: Query<(&mut PoisonCloud, &mut Sprite, &mut Transform)>,
) {
    for (cloud, mut sprite, mut tf) in &mut query {
        let pulse = (cloud.phase * 1.5).sin() * 0.15 + 0.85;
        tf.scale = Vec3::splat(pulse);
        let c = sprite.color.to_srgba();
        let alpha = 0.3 + (cloud.phase * 2.0).sin().abs() * 0.15;
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ─── Spawn Helpers (used by room.rs) ─────────────────────────────────────────

/// Spawn a lava tile at a grid position. Lava deals 1 damage every 0.5s.
pub fn spawn_lava(commands: &mut Commands, col: i32, row: i32) {
    let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    // Lava base (dark red-orange)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.6, 0.12, 0.02),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.05),
        DamageZone {
            damage: 1,
            tick_rate: 0.5,
            tick_timer: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    ));

    // Lava surface glow (animated)
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.4, 0.05, 0.85),
            custom_size: Some(Vec2::new(TILE_SIZE - 4.0, TILE_SIZE - 4.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.1),
        LavaVisual { timer: col as f32 * 0.7 },
        RoomEntity,
        PlayingEntity,
    ));

    // Bright orange highlight
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.65, 0.1, 0.5),
            custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.3)),
            ..default()
        },
        Transform::from_xyz(x - 4.0, y + 6.0, Z_TILES + 0.15),
        LavaVisual { timer: col as f32 * 1.3 + 2.0 },
        RoomEntity,
        PlayingEntity,
    ));

    // Upward glow halo
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.35, 0.0, 0.12),
            custom_size: Some(Vec2::new(TILE_SIZE + 8.0, TILE_SIZE + 16.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 8.0, Z_TILES + 0.02),
        RoomEntity,
        PlayingEntity,
    ));
}

/// Spawn a strip of lava across multiple columns.
pub fn spawn_lava_strip(commands: &mut Commands, col_start: i32, col_end: i32, row: i32) {
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        spawn_lava(commands, col, row);
    }
}

/// Spawn a water/mud tile at a grid position. Slows player to 40% speed.
pub fn spawn_water(commands: &mut Commands, col: i32, row: i32) {
    let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    // Water body (semi-transparent blue-brown for swampy feel)
    commands.spawn((
        Sprite {
            color: Color::srgba(0.15, 0.25, 0.4, 0.55),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.08),
        MovementModifier {
            speed_multiplier: 0.4,
        },
        RoomEntity,
        PlayingEntity,
    ));

    // Water surface shimmer
    commands.spawn((
        Sprite {
            color: Color::srgba(0.3, 0.5, 0.65, 0.4),
            custom_size: Some(Vec2::new(TILE_SIZE - 6.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(x, y + TILE_SIZE * 0.35, Z_TILES + 0.12),
        WaterVisual { timer: 0.0, phase: col as f32 * 0.5 },
        RoomEntity,
        PlayingEntity,
    ));

    // Secondary shimmer
    commands.spawn((
        Sprite {
            color: Color::srgba(0.25, 0.45, 0.6, 0.3),
            custom_size: Some(Vec2::new(TILE_SIZE * 0.6, 3.0)),
            ..default()
        },
        Transform::from_xyz(x + 5.0, y + TILE_SIZE * 0.15, Z_TILES + 0.11),
        WaterVisual { timer: 0.0, phase: col as f32 * 0.8 + 1.5 },
        RoomEntity,
        PlayingEntity,
    ));
}

/// Spawn a strip of water across multiple columns.
pub fn spawn_water_strip(commands: &mut Commands, col_start: i32, col_end: i32, row: i32) {
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        spawn_water(commands, col, row);
    }
}

/// Spawn a moving platform that travels between waypoints.
/// `cols` is the width in tiles, positioned at `start_col`, `start_row`.
pub fn spawn_moving_platform(
    commands: &mut Commands,
    start_col: i32,
    start_row: i32,
    cols: i32,
    waypoints: Vec<Vec2>,
    speed: f32,
    wait: f32,
) {
    let w = cols as f32 * TILE_SIZE;
    let x = start_col as f32 * TILE_SIZE + w / 2.0;
    let y = start_row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    // Platform body (warm stone, slightly lighter to distinguish from static)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.40, 0.30, 0.18),
            custom_size: Some(Vec2::new(w, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.3),
        Tile,
        KinematicPlatform {
            waypoints,
            current_index: 0,
            speed,
            wait_timer: 0.0,
            wait_duration: wait,
            prev_position: Vec2::new(x, y),
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Top edge highlight
        parent.spawn((
            Sprite {
                color: Color::srgb(0.50, 0.38, 0.22),
                custom_size: Some(Vec2::new(w - 4.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(0.0, TILE_SIZE / 2.0 - 2.0, 0.1),
        ));
        // Bottom shadow
        parent.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.2),
                custom_size: Some(Vec2::new(w, 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -TILE_SIZE / 2.0 + 1.5, 0.1),
        ));
        // Center groove
        parent.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.12),
                custom_size: Some(Vec2::new(w * 0.6, 2.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
    });
}

/// Spawn an arrow trap on a wall. `direction`: 1.0 = shoots right, -1.0 = shoots left.
pub fn spawn_arrow_trap(commands: &mut Commands, col: i32, row: i32, direction: f32) {
    let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.35, 0.25, 0.15),
            custom_size: Some(Vec2::new(TILE_SIZE * 0.6, TILE_SIZE * 0.5)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.4),
        ArrowTrap {
            fire_interval: 2.5,
            fire_timer: 1.0 + row as f32 * 0.3, // stagger
            direction,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Barrel/nozzle
        parent.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.15, 0.1),
                custom_size: Some(Vec2::new(8.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(direction * 10.0, 0.0, 0.1),
        ));
        // Red warning dot
        parent.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.2, 0.1),
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 4.0, 0.2),
        ));
    });
}

/// Spawn a spike floor tile that pops up periodically.
pub fn spawn_spike_floor(commands: &mut Commands, col: i32, row: i32) {
    let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    commands.spawn((
        Sprite {
            color: Color::srgb(0.3, 0.28, 0.25),
            custom_size: Some(Vec2::new(TILE_SIZE - 2.0, TILE_SIZE * 0.4)),
            ..default()
        },
        Transform::from_xyz(x, y - 4.0, Z_TILES + 0.3),
        SpikeFloor {
            cycle_timer: 2.0 + col as f32 * 0.2,
            cycle_duration: 2.5,
            raised: false,
            raised_duration: 1.0,
        },
        DamageZone {
            damage: 1,
            tick_rate: 0.8,
            tick_timer: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Spike tips (3 spikes)
        for i in -1..=1 {
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.5, 0.45, 0.4),
                    custom_size: Some(Vec2::new(4.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * 10.0, -5.0, 0.1), // start retracted
                SpikeVisual,
            ));
        }
    });
}

/// Spawn a poison cloud area.
pub fn spawn_poison_cloud(commands: &mut Commands, x: f32, y: f32) {
    commands.spawn((
        Sprite {
            color: Color::srgba(0.2, 0.6, 0.15, 0.35),
            custom_size: Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 1.5)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 0.5),
        PoisonCloud {
            damage: 1,
            tick_rate: 0.8,
            tick_timer: 0.0,
            phase: x * 0.1,
        },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Inner darker cloud
        parent.spawn((
            Sprite {
                color: Color::srgba(0.15, 0.5, 0.1, 0.25),
                custom_size: Some(Vec2::new(TILE_SIZE * 1.2, TILE_SIZE * 0.8)),
                ..default()
            },
            Transform::from_xyz(5.0, 3.0, 0.1),
        ));
        // Bubble particles
        for i in 0..3 {
            parent.spawn((
                Sprite {
                    color: Color::srgba(0.3, 0.8, 0.2, 0.4),
                    custom_size: Some(Vec2::new(4.0, 4.0)),
                    ..default()
                },
                Transform::from_xyz(
                    (i as f32 - 1.0) * 15.0,
                    (i as f32 * 7.3).sin() * 8.0,
                    0.2,
                ),
            ));
        }
    });
}
