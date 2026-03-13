use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, PlayingEntity,
    player::{Player, PlayerDamaged, PlayerDied},
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
                animate_lava,
                animate_water,
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
    mut zone_q: Query<(&Transform, &Sprite, &mut DamageZone)>,
    mut player_q: Query<(&Transform, &mut Player), Without<DamageZone>>,
    mut ev_damaged: EventWriter<PlayerDamaged>,
    mut ev_died: EventWriter<PlayerDied>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let Ok((p_tf, mut player)) = player_q.get_single_mut() else { return };

    for (z_tf, z_sprite, mut zone) in &mut zone_q {
        zone.tick_timer -= dt;

        let z_size = z_sprite.custom_size.unwrap_or(Vec2::new(TILE_SIZE, TILE_SIZE));
        let dist = (p_tf.translation.xy() - z_tf.translation.xy()).abs();

        // AABB overlap check (player half-size: 10x16)
        if dist.x < 10.0 + z_size.x / 2.0 - 4.0 && dist.y < 16.0 + z_size.y / 2.0 - 4.0 {
            if zone.tick_timer <= 0.0 && player.invulnerable <= 0.0 {
                zone.tick_timer = zone.tick_rate;
                player.health -= zone.damage;
                player.invulnerable = 0.3; // Short invuln between lava ticks
                ev_damaged.send(PlayerDamaged {
                    amount: zone.damage,
                    remaining: player.health,
                });
                // Knockback upward (escape lava)
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
