use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, floor_complete::FloorCompleteState,
    hazards::{spawn_lava_strip, spawn_water_strip, spawn_moving_platform}};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RoomCleared>()
            .add_event::<RoomTransition>()
            .add_event::<AdvanceFloor>()
            .insert_resource(RoomState::default())
            .insert_resource(StartRoomUnlockTimer::default())
            .add_systems(OnEnter(GameState::Playing), spawn_first_room)
            .add_systems(
                Update,
                (
                    check_room_exit,
                    check_room_cleared,
                    tick_start_door_timer,
                    handle_advance_floor,
                    animate_torches,
                    animate_crystals,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ─── Events ──────────────────────────────────────────────────────────────────

#[derive(Event)]
pub struct RoomCleared;

#[derive(Event)]
pub struct RoomTransition;

#[derive(Event)]
pub struct AdvanceFloor;

// ─── Components ──────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct ExitDoor {
    pub locked: bool,
}

#[derive(Component)]
pub struct RoomEntity;

/// Animated torch glow child sprite.
#[derive(Component)]
pub struct TorchFlicker {
    pub timer: f32,
    pub base_alpha: f32,
}

/// Animated crystal glow sprite.
#[derive(Component)]
pub struct CrystalGlow {
    pub timer: f32,
    pub phase: f32,
}

/// Marker for treasure chests that auto-open on player contact.
#[derive(Component)]
pub struct TreasureChest {
    pub opened: bool,
}

// ─── Start-room door delay ────────────────────────────────────────────────────

/// Counts down before the Start room door automatically unlocks.
/// `active` is true only while we are in a Start room and waiting.
#[derive(Resource, Default)]
pub struct StartRoomUnlockTimer {
    pub active: bool,
    pub elapsed: f32,
    pub duration: f32,
}

// ─── Room State ───────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomType {
    Combat,
    Treasure,
    Shop,
    Boss,
    Start,
}

#[derive(Resource)]
pub struct RoomState {
    pub current_type: RoomType,
    pub room_cleared: bool,
    pub floor_layout: Vec<RoomType>,
    pub room_index: usize,
    pub floor: i32,
    pub seed: u64,
}

impl Default for RoomState {
    fn default() -> Self {
        Self {
            current_type: RoomType::Start,
            room_cleared: false,
            floor_layout: Vec::new(),
            room_index: 0,
            floor: 1,
            seed: 42,
        }
    }
}

// ─── Floor Layout ─────────────────────────────────────────────────────────────

fn generate_floor_layout(floor: i32) -> Vec<RoomType> {
    let mut layout = vec![RoomType::Start];
    // More combat rooms on higher floors (4 base, +1 per floor up to +3)
    let base_rooms = (ROOMS_PER_FLOOR - 2).max(2) as usize;
    let combat_count = base_rooms + (floor as usize).saturating_sub(1).min(3);
    for i in 0..combat_count {
        if i == combat_count / 2 {
            layout.push(RoomType::Treasure);
        } else {
            layout.push(RoomType::Combat);
        }
    }
    // Shop available every floor
    layout.push(RoomType::Shop);
    layout.push(RoomType::Boss);
    layout
}

// ─── Spawn Entry Point ────────────────────────────────────────────────────────

fn spawn_first_room(
    mut commands: Commands,
    mut state: ResMut<RoomState>,
    mut run: ResMut<RunData>,
    mut start_timer: ResMut<StartRoomUnlockTimer>,
    loaded: Option<Res<crate::LoadedSave>>,
    mut floor_complete: ResMut<FloorCompleteState>,
) {
    // Reset floor complete overlay
    *floor_complete = FloorCompleteState::default();

    let floor = loaded.as_ref().map(|s| s.0.floor).unwrap_or(1);

    *state = RoomState {
        floor,
        seed: rand::random::<u64>(),
        ..default()
    };
    state.floor_layout = generate_floor_layout(floor);
    state.current_type = state.floor_layout[0];
    state.room_cleared = true;
    run.current_floor = floor;
    run.current_room = 1;

    start_timer.active = false;

    spawn_room(&mut commands, &state, 0);
}

// ─── Room Spawning ────────────────────────────────────────────────────────────

fn spawn_room(commands: &mut Commands, state: &RoomState, room_idx: usize) {
    let room_type = state.floor_layout.get(room_idx).copied().unwrap_or(RoomType::Combat);
    let seed = state.seed
        .wrapping_add(room_idx as u64)
        .wrapping_mul(state.floor as u64 + 1);

    let bg_color = match room_type {
        RoomType::Start    => Color::srgb(0.08, 0.06, 0.04),
        RoomType::Combat   => Color::srgb(0.07, 0.05, 0.03),
        RoomType::Treasure => Color::srgb(0.08, 0.07, 0.05),
        RoomType::Shop     => Color::srgb(0.08, 0.07, 0.05),
        RoomType::Boss     => Color::srgb(0.10, 0.04, 0.03),
    };

    // Background
    commands.spawn((
        Sprite {
            color: bg_color,
            custom_size: Some(Vec2::new(ROOM_W, ROOM_H)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0, ROOM_H / 2.0, Z_BACKGROUND),
        RoomEntity,
        PlayingEntity,
    ));

    let (floor_color, ceil_color, wall_color) = room_tile_colors(room_type);

    // Floor row
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, TILE_SIZE / 2.0, floor_color);
    }

    // Ceiling row
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, ROOM_H - TILE_SIZE / 2.0, ceil_color);
    }

    // Left wall
    for row in 0..ROOM_ROWS {
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, TILE_SIZE / 2.0, y, wall_color);
    }

    // Right wall (gap at rows 1-3 for door)
    for row in 0..ROOM_ROWS {
        if row >= 1 && row <= 3 { continue; }
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, ROOM_W - TILE_SIZE / 2.0, y, wall_color);
    }

    // Exit door
    let door_locked = room_type == RoomType::Combat
        || room_type == RoomType::Boss;
    let door_color = if door_locked {
        Color::srgb(0.60, 0.20, 0.08)
    } else {
        Color::srgb(0.65, 0.45, 0.15)
    };

    // Door body
    commands.spawn((
        Sprite {
            color: door_color,
            custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 3.0)),
            ..default()
        },
        Transform::from_xyz(ROOM_W - TILE_SIZE / 2.0, TILE_SIZE * 2.5, Z_TILES + 1.0),
        ExitDoor { locked: door_locked },
        RoomEntity,
        PlayingEntity,
    ));

    // Red cross pattern overlay on locked door (two thin bars)
    if door_locked {
        let door_x = ROOM_W - TILE_SIZE / 2.0;
        let door_y = TILE_SIZE * 2.5;
        // Horizontal bar
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.25, 0.08, 0.75),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.70, 5.0)),
                ..default()
            },
            Transform::from_xyz(door_x, door_y, Z_TILES + 1.2),
            RoomEntity,
            PlayingEntity,
        ));
        // Vertical bar
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.25, 0.08, 0.75),
                custom_size: Some(Vec2::new(5.0, TILE_SIZE * 2.8)),
                ..default()
            },
            Transform::from_xyz(door_x, door_y, Z_TILES + 1.2),
            RoomEntity,
            PlayingEntity,
        ));
    }

    // Stalactites hanging from ceiling (all non-boss rooms)
    if room_type != RoomType::Boss {
        spawn_stalactites(commands, seed, wall_color);
    }

    // Chains hanging from ceiling in non-boss, non-start rooms
    if room_type == RoomType::Combat || room_type == RoomType::Treasure {
        spawn_ceiling_chains(commands, seed);
    }

    // Small rubble near base of walls
    spawn_wall_rubble(commands, seed, floor_color);

    // Room-specific content
    match room_type {
        RoomType::Start    => spawn_start_room(commands, seed),
        RoomType::Combat   => spawn_combat_room(commands, seed, state.floor),
        RoomType::Treasure => spawn_treasure_room(commands, seed),
        RoomType::Boss     => spawn_boss_room(commands, state.floor),
        RoomType::Shop     => {}
    }
}

// ─── Tile Color Palette ───────────────────────────────────────────────────────

fn room_tile_colors(room_type: RoomType) -> (Color, Color, Color) {
    match room_type {
        RoomType::Start    => (
            Color::srgb(0.28, 0.22, 0.15),  // floor: warm dark stone
            Color::srgb(0.22, 0.17, 0.12),  // ceiling: darker stone
            Color::srgb(0.24, 0.19, 0.14),  // walls: neutral stone
        ),
        RoomType::Combat   => (
            Color::srgb(0.26, 0.20, 0.14),  // floor: dark stone
            Color::srgb(0.20, 0.15, 0.10),  // ceiling: deep dark
            Color::srgb(0.23, 0.18, 0.12),  // walls: stone gray
        ),
        RoomType::Treasure => (
            Color::srgb(0.28, 0.24, 0.16),
            Color::srgb(0.22, 0.19, 0.13),
            Color::srgb(0.26, 0.22, 0.15),
        ),
        RoomType::Shop     => (
            Color::srgb(0.24, 0.20, 0.14),
            Color::srgb(0.20, 0.16, 0.10),
            Color::srgb(0.22, 0.18, 0.12),
        ),
        RoomType::Boss     => (
            Color::srgb(0.28, 0.14, 0.08),
            Color::srgb(0.22, 0.10, 0.06),
            Color::srgb(0.25, 0.12, 0.07),
        ),
    }
}

// ─── Primitive Spawners ───────────────────────────────────────────────────────

fn spawn_tile(commands: &mut Commands, x: f32, y: f32, color: Color) {
    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES),
        Tile,
        RoomEntity,
        PlayingEntity,
    ));
}

fn spawn_tile_cracked(commands: &mut Commands, x: f32, y: f32, color: Color) {
    spawn_tile(commands, x, y, color);
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.06),
            custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.3)),
            ..default()
        },
        Transform::from_xyz(x - 3.0, y + 6.0, Z_TILES + 0.1),
        RoomEntity,
        PlayingEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.18),
            custom_size: Some(Vec2::new(TILE_SIZE * 0.4, TILE_SIZE * 0.15)),
            ..default()
        },
        Transform::from_xyz(x + 5.0, y - 4.0, Z_TILES + 0.1),
        RoomEntity,
        PlayingEntity,
    ));
}

fn spawn_platform(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let len = col_end - col_start + 1;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let is_left  = col == col_start;
        let is_right = col == col_end;
        spawn_tile(commands, x, y, color);
        // 3-part visual: left cap, right cap, and surface highlight
        if is_left {
            spawn_platform_cap(commands, x, y, color, true);
        }
        if is_right {
            spawn_platform_cap(commands, x, y, color, false);
        }
        // Surface highlight on middle tiles
        if !is_left && !is_right && len > 2 {
            spawn_platform_surface(commands, x, y, color);
        }
    }
}

fn spawn_platform_worn(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let len = col_end - col_start + 1;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let is_left  = col == col_start;
        let is_right = col == col_end;
        if (col - col_start) % 3 == 1 {
            spawn_tile_cracked(commands, x, y, color);
        } else {
            spawn_tile(commands, x, y, color);
        }
        if is_left {
            spawn_platform_cap(commands, x, y, color, true);
        }
        if is_right {
            spawn_platform_cap(commands, x, y, color, false);
        }
        if !is_left && !is_right && len > 2 {
            spawn_platform_surface(commands, x, y, color);
        }
    }
}

/// Left or right rounded cap for platform edges.
fn spawn_platform_cap(commands: &mut Commands, x: f32, y: f32, base_color: Color, is_left: bool) {
    let srgb = base_color.to_srgba();

    // Edge bevel — a slightly darker strip on the outer side
    let bevel_color = Color::srgb(
        (srgb.red - 0.06).max(0.0),
        (srgb.green - 0.05).max(0.0),
        (srgb.blue - 0.04).max(0.0),
    );
    let bevel_x = if is_left { x - TILE_SIZE * 0.35 } else { x + TILE_SIZE * 0.35 };
    commands.spawn((
        Sprite {
            color: bevel_color,
            custom_size: Some(Vec2::new(TILE_SIZE * 0.3, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(bevel_x, y, Z_TILES + 0.05),
        Tile, RoomEntity, PlayingEntity,
    ));

    // Top edge highlight — lighter strip across the top of the cap
    let highlight_color = Color::srgba(1.0, 1.0, 0.95, 0.08);
    commands.spawn((
        Sprite {
            color: highlight_color,
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE * 0.15)),
            ..default()
        },
        Transform::from_xyz(x, y + TILE_SIZE * 0.42, Z_TILES + 0.08),
        RoomEntity, PlayingEntity,
    ));

    // Bottom shadow on the cap edge
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.12);
    commands.spawn((
        Sprite {
            color: shadow_color,
            custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.12)),
            ..default()
        },
        Transform::from_xyz(
            if is_left { x - TILE_SIZE * 0.25 } else { x + TILE_SIZE * 0.25 },
            y - TILE_SIZE * 0.44,
            Z_TILES + 0.06,
        ),
        RoomEntity, PlayingEntity,
    ));
}

/// Surface highlight on middle platform tiles — subtle top-of-platform shine.
fn spawn_platform_surface(commands: &mut Commands, x: f32, y: f32, _color: Color) {
    // Thin light strip across the top
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.9, 0.06),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE * 0.12)),
            ..default()
        },
        Transform::from_xyz(x, y + TILE_SIZE * 0.44, Z_TILES + 0.08),
        RoomEntity, PlayingEntity,
    ));
}

// ─── Decoration Helpers ───────────────────────────────────────────────────────

/// Spawn a wall-mounted torch at an exact pixel position.
/// The handle is a thin vertical rectangle; the flame is a small diamond (two
/// overlapping rotated squares drawn as slim tall/wide rects to approximate a
/// diamond without needing mesh rotation). The glow halo flickers.
fn spawn_wall_torch(commands: &mut Commands, x: f32, y: f32) {
    // Thin handle – 4 px wide, 16 px tall
    commands.spawn((
        Sprite {
            color: Color::srgb(0.30, 0.18, 0.08),
            custom_size: Some(Vec2::new(4.0, 16.0)),
            ..default()
        },
        Transform::from_xyz(x, y - 6.0, Z_TILES + 0.5),
        RoomEntity,
        PlayingEntity,
    ));

    // Flame – tall thin rect (vertical diamond approximation)
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.60, 0.05, 0.92),
            custom_size: Some(Vec2::new(5.0, 11.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 4.0, Z_TILES + 0.6),
        RoomEntity,
        PlayingEntity,
    ));

    // Flame – wide thin rect (horizontal diamond approximation, slightly lower)
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.80, 0.20, 0.75),
            custom_size: Some(Vec2::new(8.0, 5.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 2.0, Z_TILES + 0.65),
        RoomEntity,
        PlayingEntity,
    ));

    // Glow halo (flickering)
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.65, 0.15, 0.20),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 3.0, Z_TILES + 0.4),
        TorchFlicker {
            timer: 0.0,
            base_alpha: 0.20,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

/// Spawn a small glowing crystal cluster.
fn spawn_crystal(commands: &mut Commands, x: f32, y: f32, phase_offset: f32) {
    let offsets: [(f32, f32, f32, f32); 3] = [
        (0.0,  8.0, 5.0, 14.0),
        (-6.0, 2.0, 4.0, 10.0),
        ( 6.0, 4.0, 4.0, 12.0),
    ];
    for (dx, dy, w, h) in offsets {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.45, 0.20, 0.90, 0.85),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x + dx, y + dy, Z_TILES + 0.5),
            CrystalGlow {
                timer: phase_offset,
                phase: phase_offset,
            },
            RoomEntity,
            PlayingEntity,
        ));
    }

    commands.spawn((
        Sprite {
            color: Color::srgba(0.55, 0.10, 0.95, 0.12),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 6.0, Z_TILES + 0.3),
        CrystalGlow {
            timer: phase_offset + 0.3,
            phase: phase_offset + 0.3,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

/// Spawn a small mushroom decoration.
fn spawn_mushroom(commands: &mut Commands, x: f32, y: f32) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.80, 0.75, 0.68),
            custom_size: Some(Vec2::new(5.0, 9.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 5.0, Z_TILES + 0.5),
        RoomEntity,
        PlayingEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.70, 0.22, 0.22),
            custom_size: Some(Vec2::new(13.0, 8.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 12.0, Z_TILES + 0.6),
        RoomEntity,
        PlayingEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.70),
            custom_size: Some(Vec2::new(3.0, 3.0)),
            ..default()
        },
        Transform::from_xyz(x - 3.0, y + 14.0, Z_TILES + 0.7),
        RoomEntity,
        PlayingEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.70),
            custom_size: Some(Vec2::new(2.0, 2.0)),
            ..default()
        },
        Transform::from_xyz(x + 3.0, y + 12.0, Z_TILES + 0.7),
        RoomEntity,
        PlayingEntity,
    ));
}

/// Stalactites hanging from ceiling.
fn spawn_stalactites(commands: &mut Commands, seed: u64, color: Color) {
    let mut rng = seed.wrapping_mul(2862933555777941757).wrapping_add(3037000493);

    let stalactite_color = Color::srgb(
        (color.to_srgba().red   * 0.85).min(1.0),
        (color.to_srgba().green * 0.85).min(1.0),
        (color.to_srgba().blue  * 0.80).min(1.0),
    );

    let count = 5 + (rng % 5) as i32;
    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);

    for i in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 1);
        let col = 2 + (rng % (ROOM_COLUMNS as u64 - 4)) as i32;
        let length_tiles = 1 + ((rng >> 16) % 3) as i32;

        for seg in 0..length_tiles {
            let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let y = ROOM_H - TILE_SIZE * 1.5 - seg as f32 * TILE_SIZE;
            let width = TILE_SIZE * 0.4 * (1.0 - seg as f32 * 0.25);
            commands.spawn((
                Sprite {
                    color: stalactite_color,
                    custom_size: Some(Vec2::new(width, TILE_SIZE * 0.9)),
                    ..default()
                },
                Transform::from_xyz(x, y, Z_TILES + 0.2),
                RoomEntity,
                PlayingEntity,
            ));
        }

        // Water drip near the tip of each stalactite (tiny blue teardrop)
        let tip_y = ROOM_H - TILE_SIZE * 1.5 - (length_tiles - 1) as f32 * TILE_SIZE - TILE_SIZE * 0.5;
        let drip_x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.40, 0.65, 1.0, 0.55),
                custom_size: Some(Vec2::new(3.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(drip_x, tip_y, Z_TILES + 0.25),
            RoomEntity,
            PlayingEntity,
        ));
    }
}

/// Chain segments hanging from ceiling tiles. Placed at a few deterministic
/// columns. Each chain is 2-3 links (thin gray rectangles).
fn spawn_ceiling_chains(commands: &mut Commands, seed: u64) {
    let mut rng = seed.wrapping_mul(1442695040888963407).wrapping_add(6364136223846793005);
    let chain_count = 2 + (rng % 3) as i32;
    let chain_color = Color::srgba(0.45, 0.45, 0.48, 0.80);

    for i in 0..chain_count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 7);
        let col = 3 + (rng % (ROOM_COLUMNS as u64 - 6)) as i32;
        let links = 2 + (rng >> 20) % 2; // 2-3 links
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;

        for link in 0..links {
            // Alternate tall/wide rects to simulate chain links
            let (w, h) = if link % 2 == 0 { (3.0, 8.0) } else { (6.0, 3.0) };
            let y = ROOM_H - TILE_SIZE - 6.0 - link as f32 * 9.0;
            commands.spawn((
                Sprite {
                    color: chain_color,
                    custom_size: Some(Vec2::new(w, h)),
                    ..default()
                },
                Transform::from_xyz(x, y, Z_TILES + 0.3),
                RoomEntity,
                PlayingEntity,
            ));
        }
    }
}

/// Small stone rubble chunks near the base of each wall.
fn spawn_wall_rubble(commands: &mut Commands, seed: u64, floor_color: Color) {
    let mut rng = seed.wrapping_mul(1103515245).wrapping_add(12345);
    let rubble_color = Color::srgb(
        (floor_color.to_srgba().red   * 0.70).min(1.0),
        (floor_color.to_srgba().green * 0.70).min(1.0),
        (floor_color.to_srgba().blue  * 0.70).min(1.0),
    );

    // Left wall – 2-3 chunks
    let left_count = 2 + (rng % 2) as i32;
    for i in 0..left_count {
        rng = rng.wrapping_mul(1103515245).wrapping_add(i as u64 + 1);
        let x_off = 4.0 + (rng % 12) as f32;
        let w = 5.0 + (rng % 8) as f32;
        let h = 3.0 + (rng % 5) as f32;
        let x = TILE_SIZE + x_off + w / 2.0;
        let y = TILE_SIZE + h / 2.0 + 1.0;
        commands.spawn((
            Sprite {
                color: rubble_color,
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_TILES + 0.12),
            RoomEntity,
            PlayingEntity,
        ));
    }

    // Right wall – 2-3 chunks
    let right_count = 2 + (rng % 2) as i32;
    for i in 0..right_count {
        rng = rng.wrapping_mul(1103515245).wrapping_add(i as u64 + 50);
        let x_off = 4.0 + (rng % 12) as f32;
        let w = 5.0 + (rng % 8) as f32;
        let h = 3.0 + (rng % 5) as f32;
        let x = ROOM_W - TILE_SIZE - x_off - w / 2.0;
        let y = TILE_SIZE + h / 2.0 + 1.0;
        commands.spawn((
            Sprite {
                color: rubble_color,
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_TILES + 0.12),
            RoomEntity,
            PlayingEntity,
        ));
    }
}

/// Stone pillar spanning several rows at a given column.
fn spawn_pillar(commands: &mut Commands, col: i32, row_bottom: i32, row_top: i32, color: Color) {
    for row in row_bottom..=row_top {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, y, color);
    }
}

// ─── Wall X helpers ──────────────────────────────────────────────────────────

/// X coordinate for a torch mounted on the left wall (just inside, column 1).
const LEFT_WALL_TORCH_X: f32 = TILE_SIZE + 10.0;

/// X coordinate for a torch mounted on the right wall (just inside, column 22).
fn right_wall_torch_x() -> f32 {
    ROOM_W - TILE_SIZE - 10.0
}

// ─── Start Room ───────────────────────────────────────────────────────────────

fn spawn_start_room(commands: &mut Commands, seed: u64) {
    let plat_color = Color::srgb(0.34, 0.26, 0.16);

    // Three low, welcoming stepping-stone platforms (rows 2-4, spans 3-4 tiles)
    spawn_platform(commands, 3,  6,  2, plat_color); // left, very low
    spawn_platform(commands, 9,  13, 3, plat_color); // center, one step up
    spawn_platform(commands, 16, 19, 2, plat_color); // right, back low

    // Torches ONLY on left and right walls
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);

    // Crystals in the bottom-left corner
    spawn_crystal(commands, TILE_SIZE * 2.5, TILE_SIZE,       0.0);
    spawn_crystal(commands, TILE_SIZE * 3.5, TILE_SIZE * 3.0, 1.2);

    // A few mushrooms near the base of the center platform
    let mushroom_y = 2.0 * TILE_SIZE + TILE_SIZE;
    spawn_mushroom(commands, TILE_SIZE * 4.5, mushroom_y);
    spawn_mushroom(commands, TILE_SIZE * 6.0, mushroom_y);

    // Crack overlays on some floor tiles
    for col in [2i32, 6, 11, 18] {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.15),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.6, TILE_SIZE * 0.2)),
                ..default()
            },
            Transform::from_xyz(x + 4.0, TILE_SIZE / 2.0 + 8.0, Z_TILES + 0.1),
            RoomEntity,
            PlayingEntity,
        ));
    }

    let _ = seed;
}

// ─── Combat Room Templates ────────────────────────────────────────────────────

fn spawn_combat_room(commands: &mut Commands, seed: u64, floor: i32) {
    let plat_color = Color::srgb(0.32, 0.24, 0.14);
    // Floor 1: only safe templates (no hazards). Floor 2+: all 16 templates.
    let safe_templates: &[i32] = &[0, 1, 3, 4, 7];
    let template = if floor <= 1 {
        safe_templates[(seed % safe_templates.len() as u64) as usize]
    } else {
        (seed % 16) as i32
    };

    match template {
        // ── 0: Low staircase – left to right flow ─────────────────────────────
        0 => {
            spawn_platform_worn(commands, 2,  5,  2, plat_color); // very low left
            spawn_platform_worn(commands, 7,  10, 3, plat_color); // step up
            spawn_platform_worn(commands, 12, 15, 4, plat_color); // center
            spawn_platform_worn(commands, 17, 20, 2, plat_color); // low right

            // Torches on left and right walls only
            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.5);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 3.5);
            spawn_crystal(commands, TILE_SIZE * 3.5, TILE_SIZE * 3.0, 0.5);
        }

        // ── 1: Multi-level arena ─────────────────────────────────────────────
        1 => {
            spawn_platform_worn(commands, 2,  5,  2, plat_color);
            spawn_platform_worn(commands, 8,  12, 4, plat_color);
            spawn_platform_worn(commands, 15, 18, 2, plat_color);
            spawn_platform_worn(commands, 5,  9,  5, plat_color); // medium high (was row 6)

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_mushroom(commands, TILE_SIZE * 6.5, TILE_SIZE * 2.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 10.0, TILE_SIZE * 4.0 + TILE_SIZE);
        }

        // ── 2: Lava pits with narrow bridges ──────────────────────────────────
        2 => {
            spawn_platform(commands, 3,  5,  3, plat_color);
            spawn_platform(commands, 9,  11, 3, plat_color);
            spawn_platform(commands, 15, 17, 3, plat_color);
            // Lower bridge tiles over lava pits
            spawn_platform(commands, 6,  8,  2, plat_color);
            spawn_platform(commands, 12, 14, 2, plat_color);
            // Lava in the gaps below the bridges
            spawn_lava_strip(commands, 6, 8, 1);
            spawn_lava_strip(commands, 12, 14, 1);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 7.5, TILE_SIZE * 3.0, 2.1);
            spawn_crystal(commands, TILE_SIZE * 13.5, TILE_SIZE * 3.0, 0.8);
        }

        // ── 3: Scattered low platforms ───────────────────────────────────────
        3 => {
            spawn_platform(commands, 2,  4,  2, plat_color);
            spawn_platform(commands, 6,  8,  4, plat_color);
            spawn_platform(commands, 10, 12, 2, plat_color);
            spawn_platform(commands, 14, 16, 5, plat_color);
            spawn_platform(commands, 18, 21, 3, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.5);
        }

        // ── 4: Zigzag platforms ──────────────────────────────────────────────
        4 => {
            spawn_platform(commands, 2,  4,  2, plat_color);
            spawn_platform(commands, 6,  8,  4, plat_color);
            spawn_platform(commands, 10, 13, 2, plat_color);
            spawn_platform(commands, 14, 16, 4, plat_color);
            spawn_platform(commands, 18, 21, 3, plat_color);
            // A medium-high ledge for optional routing (was row 7)
            spawn_platform(commands, 9, 13, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 11.0, TILE_SIZE * 8.0, 1.5);
            spawn_mushroom(commands, TILE_SIZE * 7.5, TILE_SIZE * 4.0 + TILE_SIZE);
        }

        // ── 5: Floating stones + moving platform ─────────────────────────────
        5 => {
            spawn_platform(commands, 2,  4,  3, plat_color);
            spawn_platform(commands, 7,  10, 5, plat_color);
            spawn_platform(commands, 13, 16, 3, plat_color);
            spawn_platform(commands, 18, 21, 4, plat_color);
            // Tiny single-tile mid-gap
            spawn_platform(commands, 5,  6,  4, plat_color);
            // Moving platform instead of static bridge at col 11-12
            let mp_x = 11.0 * TILE_SIZE + TILE_SIZE;
            let mp_y = 4.0 * TILE_SIZE + TILE_SIZE / 2.0;
            spawn_moving_platform(
                commands, 11, 4, 2,
                vec![
                    Vec2::new(mp_x, mp_y),
                    Vec2::new(mp_x, mp_y + TILE_SIZE * 3.0),
                ],
                40.0, 1.0,
            );

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.5);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 9.0, TILE_SIZE * 6.0, 0.3);
            spawn_mushroom(commands, TILE_SIZE * 8.5, TILE_SIZE * 5.0 + TILE_SIZE);
        }

        // ── 6: Low walkways with swampy water gap ─────────────────────────────
        6 => {
            // Long walkways left and right, gap in middle
            spawn_platform_worn(commands, 2,  9,  3, plat_color);
            spawn_platform_worn(commands, 13, 21, 3, plat_color);
            // Small mid platform to bridge the gap
            spawn_platform(commands, 10, 12, 4, plat_color);
            // Optional high ledge (was row 6)
            spawn_platform(commands, 7, 10, 5, plat_color);
            // Swampy water in the central gap (floor level)
            spawn_water_strip(commands, 10, 12, 1);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 2.5, TILE_SIZE * 4.0, 1.8);
        }

        // ── 7: Tunnel / open corridor with overhangs ────────────────────────
        7 => {
            // Partial overhangs instead of sealed ceiling — gaps allow flight
            spawn_platform(commands, 4,  9,  8, plat_color);  // left overhang
            spawn_platform(commands, 14, 19, 8, plat_color);  // right overhang
            // Ground bumps inside the corridor
            spawn_platform(commands, 5,  7,  2, plat_color);
            spawn_platform(commands, 11, 13, 3, plat_color);
            spawn_platform(commands, 16, 18, 2, plat_color);
            // Mid-height stepping stone in the gap for player to reach flyers
            spawn_platform(commands, 10, 13, 6, plat_color);
            // Shorter pillars — decorative, not sealing
            spawn_pillar(commands, 4,  1, 5, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 19, 1, 5, Color::srgb(0.25, 0.18, 0.10));

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 3.0);
            spawn_crystal(commands, TILE_SIZE * 14.0, TILE_SIZE * 3.0, 2.6);
            spawn_mushroom(commands, TILE_SIZE * 9.0, TILE_SIZE);
        }

        // ── 8: Lava gauntlet – lava floor with island platforms ─────────────
        8 => {
            // Lava covering most of the floor
            spawn_lava_strip(commands, 2, 21, 1);
            // Island platforms above the lava
            spawn_platform(commands, 2,  4,  2, plat_color);
            spawn_platform(commands, 7,  9,  3, plat_color);
            spawn_platform(commands, 11, 14, 2, plat_color);
            spawn_platform(commands, 17, 20, 3, plat_color);
            // High escape route
            spawn_platform(commands, 5,  6,  5, plat_color);
            spawn_platform(commands, 15, 16, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
        }

        // ── 9: Swamp marsh – water-filled lower area ────────────────────────
        9 => {
            // Water across the floor
            spawn_water_strip(commands, 3, 20, 1);
            // Raised dry platforms
            spawn_platform_worn(commands, 2,  5,  3, plat_color);
            spawn_platform_worn(commands, 8,  11, 2, plat_color);
            spawn_platform_worn(commands, 14, 17, 3, plat_color);
            spawn_platform_worn(commands, 19, 21, 2, plat_color);
            // Upper catwalk
            spawn_platform(commands, 6, 8, 5, plat_color);
            spawn_platform(commands, 12, 14, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_mushroom(commands, TILE_SIZE * 3.5, TILE_SIZE * 3.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 15.5, TILE_SIZE * 3.0 + TILE_SIZE);
        }

        // ── 10: Vertical elevator shaft – multiple moving platforms ─────────
        10 => {
            // Ground-level platforms on sides
            spawn_platform(commands, 2,  5,  2, plat_color);
            spawn_platform(commands, 18, 21, 2, plat_color);
            // High ledges on sides
            spawn_platform(commands, 2,  4,  5, plat_color);
            spawn_platform(commands, 19, 21, 5, plat_color);
            // Two moving platforms in the center
            let mp1_x = 9.0 * TILE_SIZE + TILE_SIZE;
            let mp1_y = 2.0 * TILE_SIZE + TILE_SIZE / 2.0;
            spawn_moving_platform(
                commands, 9, 2, 2,
                vec![
                    Vec2::new(mp1_x, mp1_y),
                    Vec2::new(mp1_x, mp1_y + TILE_SIZE * 4.0),
                ],
                35.0, 0.8,
            );
            let mp2_x = 14.0 * TILE_SIZE + TILE_SIZE;
            let mp2_y = 5.0 * TILE_SIZE + TILE_SIZE / 2.0;
            spawn_moving_platform(
                commands, 14, 5, 2,
                vec![
                    Vec2::new(mp2_x, mp2_y),
                    Vec2::new(mp2_x, mp2_y - TILE_SIZE * 3.0),
                ],
                35.0, 0.8,
            );

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 5.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 11.5, TILE_SIZE, 1.0);
        }

        // ── 11: Split path – high road vs low road ──────────────────────────
        11 => {
            // Low road (ground level with water hazard)
            spawn_platform(commands, 2, 6, 2, plat_color);
            spawn_water_strip(commands, 7, 16, 1);
            spawn_platform(commands, 17, 21, 2, plat_color);
            // High road (upper platforms)
            spawn_platform(commands, 3,  5,  4, plat_color);
            spawn_platform(commands, 7,  10, 5, plat_color);
            spawn_platform(commands, 12, 15, 5, plat_color);
            spawn_platform(commands, 17, 20, 4, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 11.0, TILE_SIZE * 6.0, 0.7);
            spawn_mushroom(commands, TILE_SIZE * 4.0, TILE_SIZE * 2.0 + TILE_SIZE);
        }

        // ── 12: Pillared hall – dense columns creating corridors ─────────────
        12 => {
            // Ground platforms between pillars
            spawn_platform(commands, 2, 21, 2, plat_color);
            // Pillars creating corridors
            spawn_pillar(commands, 5,  3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 10, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 15, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 20, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            // Mid-height walkways between pillars
            spawn_platform(commands, 6,  9,  5, plat_color);
            spawn_platform(commands, 11, 14, 5, plat_color);
            spawn_platform(commands, 16, 19, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 5.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 7.5, TILE_SIZE * 2.0 + TILE_SIZE, 1.2);
            spawn_crystal(commands, TILE_SIZE * 17.5, TILE_SIZE * 2.0 + TILE_SIZE, 2.4);
        }

        // ── 13: Crumbling ruins – worn platforms over lava ──────────────────
        13 => {
            // Lava base
            spawn_lava_strip(commands, 4, 19, 1);
            // Worn platforms as safe ground
            spawn_platform_worn(commands, 2,  5,  3, plat_color);
            spawn_platform_worn(commands, 7,  9,  4, plat_color);
            spawn_platform_worn(commands, 11, 13, 3, plat_color);
            spawn_platform_worn(commands, 15, 18, 5, plat_color);
            spawn_platform_worn(commands, 19, 21, 3, plat_color);
            // Collapsed pillar (decorative, acts as platform)
            spawn_platform(commands, 10, 10, 2, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 6.0, TILE_SIZE * 4.0, 3.1);
        }

        // ── 14: The pit – deep central chasm with bridges ───────────────────
        14 => {
            // Solid ground on sides
            spawn_platform(commands, 2,  7,  2, plat_color);
            spawn_platform(commands, 16, 21, 2, plat_color);
            // Lava in the pit
            spawn_lava_strip(commands, 8, 15, 1);
            // Narrow bridge across
            spawn_platform(commands, 9,  10, 3, plat_color);
            spawn_platform(commands, 13, 14, 3, plat_color);
            // Moving platform in the gap
            let mp_x = 11.0 * TILE_SIZE + TILE_SIZE;
            let mp_y = 3.0 * TILE_SIZE + TILE_SIZE / 2.0;
            spawn_moving_platform(
                commands, 11, 3, 2,
                vec![
                    Vec2::new(mp_x, mp_y),
                    Vec2::new(mp_x, mp_y + TILE_SIZE * 2.5),
                ],
                30.0, 1.5,
            );
            // Upper platforms for alternate routing
            spawn_platform(commands, 5,  7,  5, plat_color);
            spawn_platform(commands, 16, 18, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 4.0, TILE_SIZE * 3.0, 0.5);
            spawn_crystal(commands, TILE_SIZE * 19.0, TILE_SIZE * 3.0, 1.8);
        }

        // ── 15: Alternating hazards – lava and water patchwork ──────────────
        _ => {
            // Alternating lava and water patches on the floor
            spawn_lava_strip(commands, 3, 5, 1);
            spawn_water_strip(commands, 7, 9, 1);
            spawn_lava_strip(commands, 11, 13, 1);
            spawn_water_strip(commands, 15, 17, 1);
            spawn_lava_strip(commands, 19, 20, 1);
            // Platforms above the hazards
            spawn_platform(commands, 2,  4,  3, plat_color);
            spawn_platform(commands, 6,  8,  2, plat_color);
            spawn_platform(commands, 10, 12, 4, plat_color);
            spawn_platform(commands, 14, 16, 2, plat_color);
            spawn_platform(commands, 18, 21, 3, plat_color);
            // High bridge
            spawn_platform(commands, 8, 14, 5, plat_color);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.5);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 3.5);
            spawn_mushroom(commands, TILE_SIZE * 7.0, TILE_SIZE * 2.0 + TILE_SIZE);
            spawn_crystal(commands, TILE_SIZE * 18.0, TILE_SIZE * 3.0 + TILE_SIZE, 2.0);
        }
    }

    if floor >= 3 {
        spawn_crystal(commands, TILE_SIZE * 2.0, TILE_SIZE, 3.0);
    }
    let _ = floor;
}

// ─── Treasure Room ────────────────────────────────────────────────────────────

fn spawn_treasure_room(commands: &mut Commands, seed: u64) {
    let plat_color = Color::srgb(0.34, 0.30, 0.22);
    let gold_accent = Color::srgb(0.85, 0.70, 0.20);
    let gold_dark = Color::srgb(0.65, 0.50, 0.12);

    // Raised central altar – lower than before (row 2 instead of 3)
    spawn_platform(commands, 7, 16, 2, plat_color);
    // Side wings at row 4
    spawn_platform(commands, 3, 6,  4, plat_color);
    spawn_platform(commands, 17, 20, 4, plat_color);

    // ── Golden carpet/runner on the altar ──
    for col in 8..=15 {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.75, 0.55, 0.10, 0.25),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.9, TILE_SIZE * 0.15)),
                ..default()
            },
            Transform::from_xyz(x, 2.0 * TILE_SIZE + TILE_SIZE, Z_TILES + 0.08),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Gold coin piles on the sides ──
    let pile_positions = [
        (TILE_SIZE * 4.0, 4.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 5.5, 4.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 18.0, 4.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 19.5, 4.0 * TILE_SIZE + TILE_SIZE),
    ];
    for (px, py) in pile_positions {
        // Pile base
        commands.spawn((
            Sprite { color: gold_dark, custom_size: Some(Vec2::new(14.0, 6.0)), ..default() },
            Transform::from_xyz(px, py + 3.0, Z_PICKUPS - 0.2),
            RoomEntity, PlayingEntity,
        ));
        // Individual coins (scattered on top)
        commands.spawn((
            Sprite { color: gold_accent, custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(px - 3.0, py + 7.0, Z_PICKUPS - 0.15),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: gold_accent, custom_size: Some(Vec2::new(4.0, 4.0)), ..default() },
            Transform::from_xyz(px + 4.0, py + 6.0, Z_PICKUPS - 0.15),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgb(0.95, 0.80, 0.25), custom_size: Some(Vec2::new(3.0, 3.0)), ..default() },
            Transform::from_xyz(px + 1.0, py + 9.0, Z_PICKUPS - 0.1),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Golden goblets on altar edges ──
    for gx in [TILE_SIZE * 8.0, TILE_SIZE * 15.0] {
        let gy = 2.0 * TILE_SIZE + TILE_SIZE;
        // Cup base
        commands.spawn((
            Sprite { color: gold_dark, custom_size: Some(Vec2::new(8.0, 4.0)), ..default() },
            Transform::from_xyz(gx, gy + 2.0, Z_PICKUPS - 0.1),
            RoomEntity, PlayingEntity,
        ));
        // Cup body
        commands.spawn((
            Sprite { color: gold_accent, custom_size: Some(Vec2::new(6.0, 10.0)), ..default() },
            Transform::from_xyz(gx, gy + 8.0, Z_PICKUPS - 0.08),
            RoomEntity, PlayingEntity,
        ));
        // Cup rim
        commands.spawn((
            Sprite { color: Color::srgb(0.95, 0.82, 0.30), custom_size: Some(Vec2::new(9.0, 3.0)), ..default() },
            Transform::from_xyz(gx, gy + 13.0, Z_PICKUPS - 0.05),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Golden wall banners on left and right walls ──
    for bx in [TILE_SIZE * 1.5, ROOM_W - TILE_SIZE * 1.5] {
        // Banner pole
        commands.spawn((
            Sprite { color: Color::srgb(0.45, 0.35, 0.15), custom_size: Some(Vec2::new(3.0, 60.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 5.0, Z_TILES + 0.3),
            RoomEntity, PlayingEntity,
        ));
        // Banner fabric
        commands.spawn((
            Sprite { color: Color::srgba(0.70, 0.50, 0.08, 0.7), custom_size: Some(Vec2::new(18.0, 50.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 4.5, Z_TILES + 0.28),
            RoomEntity, PlayingEntity,
        ));
        // Gold emblem on banner
        commands.spawn((
            Sprite { color: Color::srgba(0.95, 0.80, 0.20, 0.8), custom_size: Some(Vec2::new(8.0, 8.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 5.0, Z_TILES + 0.32),
            RoomEntity, PlayingEntity,
        ));
    }

    // Treasure chest
    let chest_x = ROOM_W / 2.0;
    let chest_y = 2.0 * TILE_SIZE + TILE_SIZE + 14.0;

    // Main body (dark wood) — has TreasureChest for auto-open
    commands.spawn((
        Sprite { color: Color::srgb(0.50, 0.32, 0.08), custom_size: Some(Vec2::new(32.0, 20.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y, Z_PICKUPS),
        TreasureChest { opened: false },
        RoomEntity, PlayingEntity,
    ));
    // Metal band bottom
    commands.spawn((
        Sprite { color: Color::srgb(0.40, 0.38, 0.30), custom_size: Some(Vec2::new(34.0, 3.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y - 6.0, Z_PICKUPS + 0.05),
        RoomEntity, PlayingEntity,
    ));
    // Metal band middle
    commands.spawn((
        Sprite { color: Color::srgb(0.40, 0.38, 0.30), custom_size: Some(Vec2::new(34.0, 3.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 2.0, Z_PICKUPS + 0.05),
        RoomEntity, PlayingEntity,
    ));
    // Lid (lighter wood, slightly wider)
    commands.spawn((
        Sprite { color: Color::srgb(0.65, 0.45, 0.12), custom_size: Some(Vec2::new(34.0, 10.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 14.0, Z_PICKUPS + 0.1),
        RoomEntity, PlayingEntity,
    ));
    // Lid dome top (narrower for curved look)
    commands.spawn((
        Sprite { color: Color::srgb(0.58, 0.40, 0.10), custom_size: Some(Vec2::new(28.0, 5.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 20.0, Z_PICKUPS + 0.12),
        RoomEntity, PlayingEntity,
    ));
    // Metal band on lid
    commands.spawn((
        Sprite { color: Color::srgb(0.40, 0.38, 0.30), custom_size: Some(Vec2::new(36.0, 2.5)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 12.0, Z_PICKUPS + 0.15),
        RoomEntity, PlayingEntity,
    ));
    // Lock clasp (gold)
    commands.spawn((
        Sprite { color: Color::srgb(0.90, 0.75, 0.15), custom_size: Some(Vec2::new(6.0, 8.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 8.0, Z_PICKUPS + 0.2),
        RoomEntity, PlayingEntity,
    ));
    // Keyhole (dark)
    commands.spawn((
        Sprite { color: Color::srgb(0.15, 0.10, 0.05), custom_size: Some(Vec2::new(2.5, 3.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 7.0, Z_PICKUPS + 0.22),
        RoomEntity, PlayingEntity,
    ));
    // Gold glow halo (animated, larger)
    commands.spawn((
        Sprite { color: Color::srgba(1.0, 0.85, 0.20, 0.22), custom_size: Some(Vec2::new(80.0, 80.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y + 8.0, Z_PICKUPS - 0.1),
        CrystalGlow { timer: 0.0, phase: 0.0 },
        RoomEntity, PlayingEntity,
    ));
    // Secondary glow ring
    commands.spawn((
        Sprite { color: Color::srgba(0.95, 0.75, 0.15, 0.10), custom_size: Some(Vec2::new(120.0, 50.0)), ..default() },
        Transform::from_xyz(chest_x, chest_y, Z_PICKUPS - 0.15),
        CrystalGlow { timer: 0.5, phase: 0.5 },
        RoomEntity, PlayingEntity,
    ));

    // ── Scattered gems near chest ──
    let gem_colors = [
        Color::srgb(0.9, 0.2, 0.2),   // ruby
        Color::srgb(0.2, 0.7, 0.9),   // sapphire
        Color::srgb(0.3, 0.9, 0.3),   // emerald
    ];
    let gem_positions = [
        (chest_x - 30.0, chest_y - 8.0),
        (chest_x + 35.0, chest_y - 6.0),
        (chest_x - 20.0, chest_y - 10.0),
    ];
    for (i, (gx, gy)) in gem_positions.iter().enumerate() {
        commands.spawn((
            Sprite { color: gem_colors[i], custom_size: Some(Vec2::new(5.0, 5.0)), ..default() },
            Transform::from_xyz(*gx, *gy, Z_PICKUPS - 0.05),
            RoomEntity, PlayingEntity,
        ));
        // Gem sparkle
        commands.spawn((
            Sprite { color: Color::srgba(1.0, 1.0, 1.0, 0.3), custom_size: Some(Vec2::new(2.0, 2.0)), ..default() },
            Transform::from_xyz(*gx + 1.5, *gy + 1.5, Z_PICKUPS - 0.03),
            RoomEntity, PlayingEntity,
        ));
    }

    // Crystal decorations on corners
    spawn_crystal(commands, TILE_SIZE * 2.5,  TILE_SIZE,       0.0);
    spawn_crystal(commands, TILE_SIZE * 21.5, TILE_SIZE,       1.0);
    spawn_crystal(commands, TILE_SIZE * 4.0,  TILE_SIZE * 5.0, 2.0);
    spawn_crystal(commands, TILE_SIZE * 19.5, TILE_SIZE * 5.0, 3.0);

    // Torches on left and right walls
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 5.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 5.0);
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 8.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 8.0);

    // Mushrooms on the side platforms (anchored to platform top)
    spawn_mushroom(commands, TILE_SIZE * 4.5, TILE_SIZE * 4.0 + TILE_SIZE);
    spawn_mushroom(commands, TILE_SIZE * 18.5, TILE_SIZE * 4.0 + TILE_SIZE);

    // Crack overlays on the altar
    for col in [8i32, 11, 14] {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.20),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.15)),
                ..default()
            },
            Transform::from_xyz(x, 2.0 * TILE_SIZE + TILE_SIZE / 2.0 + 10.0, Z_TILES + 0.1),
            RoomEntity,
            PlayingEntity,
        ));
    }

    let _ = seed;
}

// ─── Boss Room ────────────────────────────────────────────────────────────────

fn spawn_boss_room(commands: &mut Commands, floor: i32) {
    let plat_color = Color::srgb(0.32, 0.16, 0.08);
    let pillar_color = Color::srgb(0.28, 0.14, 0.07);
    let arena_top = 4.0 * TILE_SIZE + TILE_SIZE;
    let center_x = ROOM_W / 2.0;

    // Wide ground arena
    spawn_platform(commands, 3, 20, 4, plat_color);
    // Two raised side platforms
    spawn_platform_worn(commands, 4,  7,  6, plat_color);
    spawn_platform_worn(commands, 16, 19, 6, plat_color);
    // Small stepping stones to side platforms
    spawn_platform(commands, 3, 4, 5, plat_color);
    spawn_platform(commands, 19, 20, 5, plat_color);

    // ── Dramatic stone pillars with capitals ──
    spawn_pillar(commands, 4,  5, 9, pillar_color);
    spawn_pillar(commands, 19, 5, 9, pillar_color);
    // Pillar capitals (wider tops)
    for px in [4i32, 19] {
        let cx = px as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite { color: Color::srgb(0.32, 0.18, 0.09), custom_size: Some(Vec2::new(TILE_SIZE * 1.4, TILE_SIZE * 0.6)), ..default() },
            Transform::from_xyz(cx, 9.0 * TILE_SIZE + TILE_SIZE * 0.3, Z_TILES + 0.15),
            RoomEntity, PlayingEntity,
        ));
    }
    // Inner shorter pillars
    spawn_pillar(commands, 8,  2, 5, pillar_color);
    spawn_pillar(commands, 15, 2, 5, pillar_color);

    // ── Torches on walls and pillars ──
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 6.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 6.0);
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 9.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 9.0);
    // Pillar torches
    spawn_wall_torch(commands, 4.0 * TILE_SIZE + TILE_SIZE / 2.0 + 8.0, TILE_SIZE * 7.5);
    spawn_wall_torch(commands, 19.0 * TILE_SIZE + TILE_SIZE / 2.0 - 8.0, TILE_SIZE * 7.5);

    // Lava pits flanking the arena
    spawn_lava_strip(commands, 2, 3, 1);
    spawn_lava_strip(commands, 20, 21, 1);
    if floor >= 2 {
        spawn_lava_strip(commands, 2, 2, 2);
        spawn_lava_strip(commands, 21, 21, 2);
    }

    // ── Ominous red crystals (more on higher floors) ──
    spawn_boss_crystal(commands, TILE_SIZE * 2.0,  TILE_SIZE, 0.0);
    spawn_boss_crystal(commands, TILE_SIZE * 21.0, TILE_SIZE, 1.5);
    if floor >= 2 {
        spawn_boss_crystal(commands, TILE_SIZE * 5.0, TILE_SIZE * 6.0 + TILE_SIZE, 0.8);
        spawn_boss_crystal(commands, TILE_SIZE * 18.0, TILE_SIZE * 6.0 + TILE_SIZE, 2.3);
    }

    // ── Arena ritual circle (dark rune pattern on floor) ──
    // Outer line
    commands.spawn((
        Sprite { color: Color::srgba(0.6, 0.08, 0.05, 0.15), custom_size: Some(Vec2::new(TILE_SIZE * 12.0, TILE_SIZE * 0.15)), ..default() },
        Transform::from_xyz(center_x, arena_top + 0.1, Z_TILES + 0.03),
        RoomEntity, PlayingEntity,
    ));
    // Inner cross pattern
    commands.spawn((
        Sprite { color: Color::srgba(0.5, 0.06, 0.04, 0.12), custom_size: Some(Vec2::new(TILE_SIZE * 0.15, TILE_SIZE * 2.5)), ..default() },
        Transform::from_xyz(center_x, arena_top - TILE_SIZE, Z_TILES + 0.04),
        RoomEntity, PlayingEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgba(0.5, 0.06, 0.04, 0.12), custom_size: Some(Vec2::new(TILE_SIZE * 6.0, TILE_SIZE * 0.15)), ..default() },
        Transform::from_xyz(center_x, arena_top - TILE_SIZE, Z_TILES + 0.04),
        RoomEntity, PlayingEntity,
    ));

    // ── Skull decorations near inner pillars ──
    for sx in [TILE_SIZE * 8.5, TILE_SIZE * 14.5] {
        commands.spawn((
            Sprite { color: Color::srgb(0.75, 0.70, 0.60), custom_size: Some(Vec2::new(8.0, 7.0)), ..default() },
            Transform::from_xyz(sx, arena_top + 4.0, Z_TILES + 0.12),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgb(0.15, 0.08, 0.05), custom_size: Some(Vec2::new(2.5, 2.5)), ..default() },
            Transform::from_xyz(sx - 2.0, arena_top + 5.0, Z_TILES + 0.14),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgb(0.15, 0.08, 0.05), custom_size: Some(Vec2::new(2.5, 2.5)), ..default() },
            Transform::from_xyz(sx + 2.0, arena_top + 5.0, Z_TILES + 0.14),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Dark crimson banners on walls ──
    for bx in [TILE_SIZE * 1.5, ROOM_W - TILE_SIZE * 1.5] {
        commands.spawn((
            Sprite { color: Color::srgb(0.35, 0.18, 0.08), custom_size: Some(Vec2::new(3.0, 70.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 7.0, Z_TILES + 0.3),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgba(0.45, 0.08, 0.05, 0.7), custom_size: Some(Vec2::new(20.0, 60.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 6.5, Z_TILES + 0.28),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgba(0.8, 0.15, 0.10, 0.8), custom_size: Some(Vec2::new(10.0, 10.0)), ..default() },
            Transform::from_xyz(bx, TILE_SIZE * 7.0, Z_TILES + 0.32),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Chains hanging from ceiling ──
    for chain_col in [6i32, 10, 13, 17] {
        let cx = chain_col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let chain_len = 3 + (chain_col % 3) as i32;
        for link in 0..chain_len {
            let ly = ROOM_H - TILE_SIZE - (link as f32 * 12.0);
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.35, 0.30, 0.22),
                    custom_size: Some(Vec2::new(4.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(cx, ly, Z_TILES + 0.2),
                RoomEntity, PlayingEntity,
            ));
        }
    }

    // Crack overlays on arena
    for col in [4i32, 7, 10, 13, 16, 19] {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.22),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.55, TILE_SIZE * 0.18)),
                ..default()
            },
            Transform::from_xyz(x, TILE_SIZE * 4.0 + TILE_SIZE / 2.0 + 10.0, Z_TILES + 0.1),
            RoomEntity,
            PlayingEntity,
        ));
    }

    // Arena border: darkened edge tiles
    for col in [3i32, 20] {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.15),
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE * 0.3)),
                ..default()
            },
            Transform::from_xyz(x, TILE_SIZE * 4.0 + TILE_SIZE * 0.15, Z_TILES + 0.12),
            RoomEntity, PlayingEntity,
        ));
    }

    // Ominous floor glow (pulsing)
    commands.spawn((
        Sprite {
            color: Color::srgba(0.7, 0.1, 0.05, 0.10),
            custom_size: Some(Vec2::new(TILE_SIZE * 12.0, TILE_SIZE * 2.5)),
            ..default()
        },
        Transform::from_xyz(center_x, TILE_SIZE * 4.5, Z_TILES + 0.02),
        CrystalGlow { timer: 0.0, phase: 0.0 },
        RoomEntity, PlayingEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgba(0.8, 0.05, 0.02, 0.06),
            custom_size: Some(Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 1.5)),
            ..default()
        },
        Transform::from_xyz(center_x, TILE_SIZE * 4.5, Z_TILES + 0.025),
        CrystalGlow { timer: 1.5, phase: 1.5 },
        RoomEntity, PlayingEntity,
    ));
}

fn spawn_boss_crystal(commands: &mut Commands, x: f32, y: f32, phase_offset: f32) {
    let offsets: [(f32, f32, f32, f32); 3] = [
        (0.0,  8.0, 5.0, 14.0),
        (-6.0, 2.0, 4.0, 10.0),
        ( 6.0, 4.0, 4.0, 12.0),
    ];
    for (dx, dy, w, h) in offsets {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.80, 0.10, 0.15, 0.85),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x + dx, y + dy, Z_TILES + 0.5),
            CrystalGlow {
                timer: phase_offset,
                phase: phase_offset,
            },
            RoomEntity,
            PlayingEntity,
        ));
    }
    commands.spawn((
        Sprite {
            color: Color::srgba(0.90, 0.05, 0.10, 0.14),
            custom_size: Some(Vec2::new(28.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 6.0, Z_TILES + 0.3),
        CrystalGlow {
            timer: phase_offset + 0.3,
            phase: phase_offset + 0.3,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

// ─── Animation Systems ────────────────────────────────────────────────────────

pub fn animate_torches(
    time: Res<Time>,
    mut query: Query<(&mut TorchFlicker, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut flicker, mut sprite) in &mut query {
        flicker.timer += dt * 4.5;
        let flicker_val =
            (flicker.timer * 1.0).sin() * 0.5
            + (flicker.timer * 2.7).sin() * 0.3
            + (flicker.timer * 0.4).sin() * 0.2;
        let alpha = (flicker.base_alpha + flicker_val * 0.10).clamp(0.08, 0.40);
        let srgba = sprite.color.to_srgba();
        sprite.color = Color::srgba(srgba.red, srgba.green, srgba.blue, alpha);
    }
}

pub fn animate_crystals(
    time: Res<Time>,
    mut query: Query<(&mut CrystalGlow, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut crystal, mut sprite) in &mut query {
        crystal.timer += dt * 1.8;
        let pulse = (crystal.timer + crystal.phase).sin() * 0.5 + 0.5;
        let srgba = sprite.color.to_srgba();
        let new_alpha = (srgba.alpha * 0.6 + pulse * srgba.alpha * 0.8).clamp(0.05, 1.0);
        sprite.color = Color::srgba(srgba.red, srgba.green, srgba.blue, new_alpha);
    }
}

// ─── Start-room door delay timer ─────────────────────────────────────────────

fn tick_start_door_timer(
    time: Res<Time>,
    mut timer: ResMut<StartRoomUnlockTimer>,
    mut room_state: ResMut<RoomState>,
    mut door_q: Query<(&mut ExitDoor, &mut Sprite)>,
    mut ev_cleared: EventWriter<RoomCleared>,
) {
    if !timer.active { return; }
    if room_state.current_type != RoomType::Start { timer.active = false; return; }

    timer.elapsed += time.delta_secs();
    if timer.elapsed >= timer.duration {
        timer.active = false;
        room_state.room_cleared = true;
        ev_cleared.send(RoomCleared);

        for (mut door, mut sprite) in &mut door_q {
            door.locked = false;
            sprite.color = Color::srgb(0.65, 0.45, 0.15);
        }
    }
}

// ─── Room Cleared / Exit Logic ────────────────────────────────────────────────

fn check_room_cleared(
    mut state: ResMut<RoomState>,
    run: Res<RunData>,
    mut door_q: Query<(&mut ExitDoor, &mut Sprite)>,
    mut ev_cleared: EventWriter<RoomCleared>,
) {
    if state.room_cleared { return; }

    let is_cleared = match state.current_type {
        RoomType::Combat | RoomType::Boss => run.enemies_alive <= 0,
        _ => true,
    };

    if is_cleared {
        state.room_cleared = true;
        ev_cleared.send(RoomCleared);

        for (mut door, mut sprite) in &mut door_q {
            door.locked = false;
            sprite.color = Color::srgb(0.65, 0.45, 0.15);
        }
    }
}

fn check_room_exit(
    mut commands: Commands,
    mut player_q: Query<(&mut Player, &mut Transform), Without<ExitDoor>>,
    door_q: Query<(&Transform, &ExitDoor), Without<Player>>,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut run: ResMut<RunData>,
    mut room_state: ResMut<RoomState>,
    mut start_timer: ResMut<StartRoomUnlockTimer>,
    mut ev_transition: EventWriter<RoomTransition>,
    mut floor_complete: ResMut<FloorCompleteState>,
) {
    // Block room transitions while floor complete overlay is active
    if floor_complete.active { return; }

    let Ok((mut player, mut player_tf)) = player_q.get_single_mut() else { return };

    for (door_tf, door) in &door_q {
        if door.locked { continue; }

        let dist = (player_tf.translation.xy() - door_tf.translation.xy()).abs();
        if dist.x < 30.0 && dist.y < 60.0 {
            // Check if this is the last room of the floor (boss cleared)
            let is_floor_end = room_state.room_index + 1 >= room_state.floor_layout.len();

            if is_floor_end {
                // Show floor complete overlay (keep room visible behind it)
                floor_complete.active = true;
                floor_complete.floor_completed = room_state.floor;
                floor_complete.ui_spawned = false;
                break;
            }

            // Normal room transition
            ev_transition.send(RoomTransition);

            for entity in &room_entities {
                commands.entity(entity).try_despawn_recursive();
            }

            room_state.room_index += 1;
            run.rooms_cleared += 1;
            run.current_room += 1;

            room_state.current_type = room_state.floor_layout[room_state.room_index];
            room_state.room_cleared = match room_state.current_type {
                RoomType::Combat | RoomType::Boss => false,
                _ => true,
            };

            // Re-arm the start-room timer if transitioning into a Start room
            if room_state.current_type == RoomType::Start {
                *start_timer = StartRoomUnlockTimer {
                    active: true,
                    elapsed: 0.0,
                    duration: 2.0,
                };
            } else {
                start_timer.active = false;
            }

            spawn_room(&mut commands, &room_state, room_state.room_index);

            // Teleport player to safe spawn (left side, above floor)
            player_tf.translation.x = TILE_SIZE * 2.5;
            player_tf.translation.y = TILE_SIZE * 2.0;
            player.vx = 0.0;
            player.vy = 0.0;

            break;
        }
    }
}

/// Handles the AdvanceFloor event sent from FloorComplete when player chooses to descend.
fn handle_advance_floor(
    mut commands: Commands,
    mut ev: EventReader<AdvanceFloor>,
    mut room_state: ResMut<RoomState>,
    mut run: ResMut<RunData>,
    mut start_timer: ResMut<StartRoomUnlockTimer>,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut player_q: Query<(&mut Player, &mut Transform), Without<ExitDoor>>,
    mut ev_transition: EventWriter<RoomTransition>,
) {
    for _ in ev.read() {
        // Despawn old room
        for entity in &room_entities {
            commands.entity(entity).try_despawn_recursive();
        }

        ev_transition.send(RoomTransition);

        // Advance floor
        room_state.floor += 1;
        run.current_floor = room_state.floor;
        run.current_room = 1;
        room_state.room_index = 0;
        room_state.floor_layout = generate_floor_layout(room_state.floor);
        room_state.seed = room_state.seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1);
        room_state.current_type = room_state.floor_layout[0];
        room_state.room_cleared = true;
        start_timer.active = false;

        spawn_room(&mut commands, &room_state, 0);

        // Teleport player
        if let Ok((mut player, mut tf)) = player_q.get_single_mut() {
            tf.translation.x = TILE_SIZE * 2.5;
            tf.translation.y = TILE_SIZE * 2.0;
            player.vx = 0.0;
            player.vy = 0.0;
        }

        break;
    }
}
