use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player};

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RoomCleared>()
            .add_event::<RoomTransition>()
            .insert_resource(RoomState::default())
            .add_systems(OnEnter(GameState::Playing), spawn_first_room)
            .add_systems(
                Update,
                (
                    check_room_exit,
                    check_room_cleared,
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
    let combat_count = (ROOMS_PER_FLOOR - 2).max(2) as usize;
    for i in 0..combat_count {
        if i == combat_count / 2 {
            layout.push(RoomType::Treasure);
        } else {
            layout.push(RoomType::Combat);
        }
    }
    if floor % 2 == 0 {
        // Insert shop before boss on even floors
        layout.push(RoomType::Shop);
    }
    layout.push(RoomType::Boss);
    layout
}

// ─── Spawn Entry Point ────────────────────────────────────────────────────────

fn spawn_first_room(
    mut commands: Commands,
    mut state: ResMut<RoomState>,
    mut run: ResMut<RunData>,
) {
    *state = RoomState {
        floor: 1,
        seed: 42,
        ..default()
    };
    state.floor_layout = generate_floor_layout(1);
    state.current_type = state.floor_layout[0];
    state.room_cleared = true; // Start room is pre-cleared
    run.current_floor = 1;
    run.current_room = 1;

    spawn_room(&mut commands, &state, 0);
}

// ─── Room Spawning ────────────────────────────────────────────────────────────

fn spawn_room(commands: &mut Commands, state: &RoomState, room_idx: usize) {
    let room_type = state.floor_layout.get(room_idx).copied().unwrap_or(RoomType::Combat);
    let seed = state.seed
        .wrapping_add(room_idx as u64)
        .wrapping_mul(state.floor as u64 + 1);

    // Pick background tint by room type
    let bg_color = match room_type {
        RoomType::Start    => Color::srgb(0.07, 0.09, 0.14),
        RoomType::Combat   => Color::srgb(0.08, 0.06, 0.12),
        RoomType::Treasure => Color::srgb(0.08, 0.07, 0.05),
        RoomType::Shop     => Color::srgb(0.06, 0.08, 0.10),
        RoomType::Boss     => Color::srgb(0.10, 0.04, 0.06),
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

    // Wall / floor colors vary per room
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
    let door_locked = room_type == RoomType::Combat || room_type == RoomType::Boss;
    let door_color = if door_locked {
        Color::srgb(0.6, 0.2, 0.2)
    } else {
        Color::srgb(0.2, 0.6, 0.3)
    };
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

    // Stalactites hanging from ceiling (all non-boss rooms)
    if room_type != RoomType::Boss {
        spawn_stalactites(commands, seed, wall_color);
    }

    // Room-specific content
    match room_type {
        RoomType::Start    => spawn_start_room(commands, seed),
        RoomType::Combat   => spawn_combat_room(commands, seed, state.floor),
        RoomType::Treasure => spawn_treasure_room(commands, seed),
        RoomType::Boss     => spawn_boss_room(commands, state.floor),
        RoomType::Shop     => {} // Shop handled by shop.rs
    }
}

// ─── Tile Color Palette ───────────────────────────────────────────────────────

fn room_tile_colors(room_type: RoomType) -> (Color, Color, Color) {
    match room_type {
        RoomType::Start    => (
            Color::srgb(0.24, 0.22, 0.32),
            Color::srgb(0.20, 0.18, 0.28),
            Color::srgb(0.22, 0.20, 0.30),
        ),
        RoomType::Combat   => (
            Color::srgb(0.22, 0.18, 0.28),
            Color::srgb(0.18, 0.14, 0.22),
            Color::srgb(0.20, 0.16, 0.25),
        ),
        RoomType::Treasure => (
            Color::srgb(0.28, 0.24, 0.16),
            Color::srgb(0.22, 0.19, 0.13),
            Color::srgb(0.26, 0.22, 0.15),
        ),
        RoomType::Shop     => (
            Color::srgb(0.18, 0.22, 0.28),
            Color::srgb(0.14, 0.18, 0.22),
            Color::srgb(0.16, 0.20, 0.26),
        ),
        RoomType::Boss     => (
            Color::srgb(0.28, 0.12, 0.16),
            Color::srgb(0.20, 0.08, 0.12),
            Color::srgb(0.24, 0.10, 0.14),
        ),
    }
}

// ─── Primitive Spawners ───────────────────────────────────────────────────────

fn spawn_tile(commands: &mut Commands, x: f32, y: f32, color: Color) {
    // Main tile
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

/// Spawn a crack/moss overlay on top of a tile for visual variety.
fn spawn_tile_cracked(commands: &mut Commands, x: f32, y: f32, color: Color) {
    spawn_tile(commands, x, y, color);
    // Crack overlay: slightly lighter irregular shape
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
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, y, color);
    }
}

/// Same as spawn_platform but applies cracked overlays every other tile.
fn spawn_platform_worn(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        if (col - col_start) % 3 == 1 {
            spawn_tile_cracked(commands, x, y, color);
        } else {
            spawn_tile(commands, x, y, color);
        }
    }
}

// ─── Decoration Helpers ───────────────────────────────────────────────────────

/// Spawn a torch at a wall position. `facing_right` controls which side of the
/// wall the flame hangs on.
fn spawn_torch(commands: &mut Commands, x: f32, y: f32) {
    // Torch body (dark brown handle)
    commands.spawn((
        Sprite {
            color: Color::srgb(0.35, 0.22, 0.10),
            custom_size: Some(Vec2::new(6.0, 18.0)),
            ..default()
        },
        Transform::from_xyz(x, y - 4.0, Z_TILES + 0.5),
        RoomEntity,
        PlayingEntity,
    ));

    // Flame (orange)
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.55, 0.05, 0.90),
            custom_size: Some(Vec2::new(10.0, 14.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 7.0, Z_TILES + 0.6),
        RoomEntity,
        PlayingEntity,
    ));

    // Glow halo – this is the flickering child
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.70, 0.15, 0.22),
            custom_size: Some(Vec2::new(36.0, 36.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 7.0, Z_TILES + 0.4),
        TorchFlicker {
            timer: 0.0,
            base_alpha: 0.22,
        },
        RoomEntity,
        PlayingEntity,
    ));
}

/// Spawn a small glowing crystal cluster at a corner or on a platform edge.
fn spawn_crystal(commands: &mut Commands, x: f32, y: f32, phase_offset: f32) {
    // Three tiny shards at slightly different positions
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

    // Soft purple glow halo
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

/// Spawn a small mushroom decoration on a floor/platform tile.
fn spawn_mushroom(commands: &mut Commands, x: f32, y: f32) {
    // Stem
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
    // Cap
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
    // White dots on cap
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

/// Spawn stalactites hanging from the ceiling. Uses seed for positioning.
fn spawn_stalactites(commands: &mut Commands, seed: u64, color: Color) {
    // Deterministic pseudo-random placement
    let mut rng = seed.wrapping_mul(2862933555777941757).wrapping_add(3037000493);

    let stalactite_color = Color::srgb(
        (color.to_srgba().red   * 0.85).min(1.0),
        (color.to_srgba().green * 0.85).min(1.0),
        (color.to_srgba().blue  * 0.80).min(1.0),
    );

    // Place 5 to 9 stalactites
    let count = 5 + (rng % 5) as i32;
    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);

    for i in 0..count {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(i as u64 + 1);
        let col = 2 + (rng % (ROOM_COLUMNS as u64 - 4)) as i32;
        let length_tiles = 1 + ((rng >> 16) % 3) as i32; // 1–3 tiles long

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
    }
}

/// Spawn a stone pillar spanning several rows at a given column.
fn spawn_pillar(commands: &mut Commands, col: i32, row_bottom: i32, row_top: i32, color: Color) {
    for row in row_bottom..=row_top {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, y, color);
    }
}

// ─── Start Room ───────────────────────────────────────────────────────────────

fn spawn_start_room(commands: &mut Commands, seed: u64) {
    let plat_color = Color::srgb(0.26, 0.24, 0.34);

    // Three welcoming platforms stepping upward
    spawn_platform(commands, 3, 8,  4, plat_color);
    spawn_platform(commands, 10, 15, 6, plat_color);
    spawn_platform(commands, 16, 21, 4, plat_color);

    // Torches on the left wall, mid wall, and near the door
    spawn_torch(commands, TILE_SIZE + 14.0,              TILE_SIZE * 5.0);
    spawn_torch(commands, TILE_SIZE * 12.0,              TILE_SIZE * 7.5);
    spawn_torch(commands, ROOM_W - TILE_SIZE * 2.0,      TILE_SIZE * 4.5);

    // Crystals in the bottom-left corner
    spawn_crystal(commands, TILE_SIZE * 2.5, TILE_SIZE,       0.0);
    spawn_crystal(commands, TILE_SIZE * 4.0, TILE_SIZE * 5.0, 1.2);

    // A few mushrooms on the first platform
    let mushroom_y = 4.0 * TILE_SIZE + TILE_SIZE;
    spawn_mushroom(commands, TILE_SIZE * 4.5, mushroom_y);
    spawn_mushroom(commands, TILE_SIZE * 7.0, mushroom_y);

    // Crack overlays on some floor tiles for texture
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

    let _ = seed; // reserved for future variation
}

// ─── Combat Room Templates ────────────────────────────────────────────────────

fn spawn_combat_room(commands: &mut Commands, seed: u64, floor: i32) {
    let plat_color = Color::srgb(0.25, 0.22, 0.30);
    let template = (seed % 8) as i32;

    match template {
        // ── 0: Staircase ──────────────────────────────────────────────────────
        0 => {
            spawn_platform_worn(commands, 3,  6,  3, plat_color);
            spawn_platform_worn(commands, 8,  11, 5, plat_color);
            spawn_platform_worn(commands, 13, 16, 7, plat_color);
            spawn_platform_worn(commands, 18, 21, 5, plat_color);

            spawn_torch(commands, TILE_SIZE * 5.0,  TILE_SIZE * 4.0);
            spawn_torch(commands, TILE_SIZE * 15.0, TILE_SIZE * 8.0);
            spawn_crystal(commands, TILE_SIZE * 3.5, TILE_SIZE * 4.0, 0.5);
        }

        // ── 1: Multi-level arena ──────────────────────────────────────────────
        1 => {
            spawn_platform_worn(commands, 2,  7,  4, plat_color);
            spawn_platform_worn(commands, 9,  14, 7, plat_color);
            spawn_platform_worn(commands, 16, 21, 4, plat_color);
            spawn_platform_worn(commands, 5,  10, 10, plat_color);

            spawn_torch(commands, TILE_SIZE * 3.0,  TILE_SIZE * 5.0);
            spawn_torch(commands, TILE_SIZE * 17.0, TILE_SIZE * 5.0);
            spawn_mushroom(commands, TILE_SIZE * 6.5, TILE_SIZE * 4.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 11.0, TILE_SIZE * 7.0 + TILE_SIZE);
        }

        // ── 2: Pits with narrow bridges ───────────────────────────────────────
        2 => {
            spawn_platform(commands, 4,  6,  4, plat_color);
            spawn_platform(commands, 10, 12, 4, plat_color);
            spawn_platform(commands, 16, 18, 4, plat_color);
            // Lower bridging platforms over pits
            spawn_platform(commands, 7,  9,  7, plat_color);
            spawn_platform(commands, 13, 15, 7, plat_color);

            spawn_torch(commands, TILE_SIZE * 5.5,  TILE_SIZE * 5.0);
            spawn_torch(commands, TILE_SIZE * 17.5, TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 8.0, TILE_SIZE * 8.0, 2.1);
            spawn_crystal(commands, TILE_SIZE * 14.0, TILE_SIZE * 8.0, 0.8);
        }

        // ── 3: Tower platforms ────────────────────────────────────────────────
        3 => {
            spawn_platform(commands, 5,  7,  3, plat_color);
            spawn_platform(commands, 5,  7,  6, plat_color);
            spawn_platform(commands, 5,  7,  9, plat_color);
            spawn_platform(commands, 11, 13, 4, plat_color);
            spawn_platform(commands, 11, 13, 7, plat_color);
            spawn_platform(commands, 17, 19, 3, plat_color);
            spawn_platform(commands, 17, 19, 6, plat_color);

            spawn_torch(commands, TILE_SIZE * 6.0,  TILE_SIZE * 7.0);
            spawn_torch(commands, TILE_SIZE * 12.0, TILE_SIZE * 5.0);
            spawn_torch(commands, TILE_SIZE * 18.0, TILE_SIZE * 4.0);
        }

        // ── 4: Zigzag platforms ───────────────────────────────────────────────
        4 => {
            spawn_platform(commands, 2,  5,  3, plat_color);
            spawn_platform(commands, 6,  9,  5, plat_color);
            spawn_platform(commands, 10, 13, 3, plat_color);
            spawn_platform(commands, 14, 17, 6, plat_color);
            spawn_platform(commands, 18, 21, 4, plat_color);
            // A high ledge at center
            spawn_platform(commands, 9, 14, 9, plat_color);

            spawn_torch(commands, TILE_SIZE * 3.5,  TILE_SIZE * 4.0);
            spawn_torch(commands, TILE_SIZE * 19.5, TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 11.5, TILE_SIZE * 10.0, 1.5);
            spawn_mushroom(commands, TILE_SIZE * 7.5, TILE_SIZE * 5.0 + TILE_SIZE);
        }

        // ── 5: Floating islands ───────────────────────────────────────────────
        5 => {
            // Three isolated floating platforms with gaps between them
            spawn_platform(commands, 2,  6,  5, plat_color);
            spawn_platform(commands, 9,  14, 8, plat_color);
            spawn_platform(commands, 16, 21, 5, plat_color);
            // Tiny stepping stones to help traversal
            spawn_platform(commands, 7,  8,  6, plat_color);
            spawn_platform(commands, 15, 15, 6, plat_color);

            spawn_torch(commands, TILE_SIZE * 4.0,  TILE_SIZE * 6.5);
            spawn_torch(commands, TILE_SIZE * 18.0, TILE_SIZE * 6.5);
            spawn_crystal(commands, TILE_SIZE * 11.5, TILE_SIZE * 9.5, 0.3);
            spawn_mushroom(commands, TILE_SIZE * 10.5, TILE_SIZE * 8.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 13.0, TILE_SIZE * 8.0 + TILE_SIZE);
        }

        // ── 6: Elevated walkways / catwalk ────────────────────────────────────
        6 => {
            // Long upper walkway
            spawn_platform_worn(commands, 2, 22, 9, plat_color);
            // Lower ground platforms with a central gap (pit)
            spawn_platform_worn(commands, 2,  8,  4, plat_color);
            spawn_platform_worn(commands, 15, 21, 4, plat_color);
            // Short ledge above the pit for dramatic drops
            spawn_platform(commands, 10, 13, 6, plat_color);

            spawn_torch(commands, TILE_SIZE * 5.0,  TILE_SIZE * 5.0);
            spawn_torch(commands, TILE_SIZE * 11.5, TILE_SIZE * 10.5);
            spawn_torch(commands, TILE_SIZE * 18.0, TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 2.5, TILE_SIZE * 5.0, 1.8);
        }

        // ── 7: Tunnel / low-ceiling corridor ─────────────────────────────────
        _ => {
            // A raised false ceiling that forces crouching hops
            // (represented as a solid ceiling platform at row 10)
            spawn_platform(commands, 4, 19, 10, plat_color);
            // Ground bumps and small platforms inside the tunnel
            spawn_platform(commands, 6,  8,  3, plat_color);
            spawn_platform(commands, 12, 14, 4, plat_color);
            spawn_platform(commands, 17, 19, 3, plat_color);
            // Openings: columns 4-5 and 19-20 give access through top
            // Wall column on each side inside
            spawn_pillar(commands, 4, 1, 10,
                Color::srgb(0.22, 0.18, 0.27));
            spawn_pillar(commands, 19, 1, 10,
                Color::srgb(0.22, 0.18, 0.27));

            spawn_torch(commands, TILE_SIZE * 7.5,  TILE_SIZE * 4.0);
            spawn_torch(commands, TILE_SIZE * 13.5, TILE_SIZE * 5.0);
            spawn_crystal(commands, TILE_SIZE * 15.5, TILE_SIZE * 4.0, 2.6);
            spawn_mushroom(commands, TILE_SIZE * 9.0, TILE_SIZE);
        }
    }

    // Scale difficulty cue: add extra crystals on higher floors
    if floor >= 3 {
        spawn_crystal(commands, TILE_SIZE * 2.0, TILE_SIZE, 3.0);
    }
    let _ = floor; // remainder used via enemy.rs
}

// ─── Treasure Room ────────────────────────────────────────────────────────────

fn spawn_treasure_room(commands: &mut Commands, seed: u64) {
    let plat_color = Color::srgb(0.30, 0.28, 0.18);

    // Raised central altar platform
    spawn_platform(commands, 7, 16, 3, plat_color);
    // Side wings
    spawn_platform(commands, 3, 6,  5, plat_color);
    spawn_platform(commands, 17, 20, 5, plat_color);

    // Treasure chest placeholder – glowing golden box
    let chest_x = ROOM_W / 2.0;
    let chest_y = 3.0 * TILE_SIZE + TILE_SIZE + 14.0;

    // Chest body
    commands.spawn((
        Sprite {
            color: Color::srgb(0.60, 0.42, 0.10),
            custom_size: Some(Vec2::new(28.0, 22.0)),
            ..default()
        },
        Transform::from_xyz(chest_x, chest_y, Z_PICKUPS),
        RoomEntity,
        PlayingEntity,
    ));
    // Chest lid
    commands.spawn((
        Sprite {
            color: Color::srgb(0.80, 0.62, 0.18),
            custom_size: Some(Vec2::new(28.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(chest_x, chest_y + 14.0, Z_PICKUPS + 0.1),
        RoomEntity,
        PlayingEntity,
    ));
    // Gold glow halo
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.85, 0.20, 0.18),
            custom_size: Some(Vec2::new(56.0, 56.0)),
            ..default()
        },
        Transform::from_xyz(chest_x, chest_y + 4.0, Z_PICKUPS - 0.1),
        CrystalGlow {
            timer: 0.0,
            phase: 0.0,
        },
        RoomEntity,
        PlayingEntity,
    ));

    // Crystal decorations on the side wings and corners
    spawn_crystal(commands, TILE_SIZE * 2.5,  TILE_SIZE,            0.0);
    spawn_crystal(commands, TILE_SIZE * 21.5, TILE_SIZE,            1.0);
    spawn_crystal(commands, TILE_SIZE * 4.0,  TILE_SIZE * 6.0,      2.0);
    spawn_crystal(commands, TILE_SIZE * 19.5, TILE_SIZE * 6.0,      3.0);

    // Torches flanking the chest
    spawn_torch(commands, chest_x - TILE_SIZE * 3.0, 3.0 * TILE_SIZE + TILE_SIZE);
    spawn_torch(commands, chest_x + TILE_SIZE * 3.0, 3.0 * TILE_SIZE + TILE_SIZE);

    // Mushrooms on the side platforms
    spawn_mushroom(commands, TILE_SIZE * 4.5, TILE_SIZE * 5.0 + TILE_SIZE);
    spawn_mushroom(commands, TILE_SIZE * 18.5, TILE_SIZE * 5.0 + TILE_SIZE);

    // Crack overlays on the altar for a weathered look
    for col in [8i32, 11, 14] {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        commands.spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.20),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.15)),
                ..default()
            },
            Transform::from_xyz(x, 3.0 * TILE_SIZE + TILE_SIZE / 2.0 + 10.0, Z_TILES + 0.1),
            RoomEntity,
            PlayingEntity,
        ));
    }

    let _ = seed;
}

// ─── Boss Room ────────────────────────────────────────────────────────────────

fn spawn_boss_room(commands: &mut Commands, _floor: i32) {
    let plat_color = Color::srgb(0.32, 0.12, 0.18);
    let pillar_color = Color::srgb(0.26, 0.10, 0.14);

    // Wide ground arena
    spawn_platform(commands, 3, 20, 4, plat_color);
    // Two raised side platforms
    spawn_platform_worn(commands, 4,  7,  7, plat_color);
    spawn_platform_worn(commands, 16, 19, 7, plat_color);

    // Dramatic stone pillars flanking the arena
    spawn_pillar(commands, 4,  5, 8, pillar_color);
    spawn_pillar(commands, 19, 5, 8, pillar_color);
    // Smaller interior pillars
    spawn_pillar(commands, 8,  2, 4, pillar_color);
    spawn_pillar(commands, 15, 2, 4, pillar_color);

    // Torches on pillars
    spawn_torch(commands, TILE_SIZE * 4.5,  TILE_SIZE * 9.5);
    spawn_torch(commands, TILE_SIZE * 19.5, TILE_SIZE * 9.5);
    spawn_torch(commands, TILE_SIZE * 8.5,  TILE_SIZE * 5.5);
    spawn_torch(commands, TILE_SIZE * 15.5, TILE_SIZE * 5.5);

    // Ominous red crystals at the back corners
    spawn_boss_crystal(commands, TILE_SIZE * 2.0,  TILE_SIZE, 0.0);
    spawn_boss_crystal(commands, TILE_SIZE * 21.0, TILE_SIZE, 1.5);

    // Crack overlays all over the arena to show wear
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
}

/// Boss room version of crystal – red/dark red instead of purple.
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
        // Combine two sine waves at different frequencies for organic flicker
        let flicker_val =
            (flicker.timer * 1.0).sin() * 0.5
            + (flicker.timer * 2.7).sin() * 0.3
            + (flicker.timer * 0.4).sin() * 0.2;
        // Map -1..1 to a pulsing alpha range
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
        let pulse = (crystal.timer + crystal.phase).sin() * 0.5 + 0.5; // 0..1
        let srgba = sprite.color.to_srgba();
        // Pulse both alpha and a slight brightness shift
        let new_alpha = (srgba.alpha * 0.6 + pulse * srgba.alpha * 0.8).clamp(0.05, 1.0);
        sprite.color = Color::srgba(srgba.red, srgba.green, srgba.blue, new_alpha);
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

        // Unlock door
        for (mut door, mut sprite) in &mut door_q {
            door.locked = false;
            sprite.color = Color::srgb(0.2, 0.6, 0.3);
        }
    }
}

fn check_room_exit(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    door_q: Query<(&Transform, &ExitDoor), Without<Player>>,
    room_entities: Query<Entity, With<RoomEntity>>,
    mut run: ResMut<RunData>,
    mut room_state: ResMut<RoomState>,
    mut ev_transition: EventWriter<RoomTransition>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };

    for (door_tf, door) in &door_q {
        if door.locked { continue; }

        let dist = (player_tf.translation.xy() - door_tf.translation.xy()).abs();
        if dist.x < 30.0 && dist.y < 60.0 {
            // Transition to next room
            ev_transition.send(RoomTransition);

            // Despawn current room
            for entity in &room_entities {
                commands.entity(entity).despawn_recursive();
            }

            // Advance room
            room_state.room_index += 1;
            run.rooms_cleared += 1;
            run.current_room += 1;

            // Check if floor complete
            if room_state.room_index >= room_state.floor_layout.len() {
                room_state.floor += 1;
                run.current_floor = room_state.floor;
                run.current_room = 1;
                room_state.room_index = 0;
                room_state.floor_layout = generate_floor_layout(room_state.floor);
                room_state.seed = room_state.seed
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1);
            }

            room_state.current_type = room_state.floor_layout[room_state.room_index];
            room_state.room_cleared = match room_state.current_type {
                RoomType::Combat | RoomType::Boss => false,
                _ => true,
            };

            spawn_room(&mut commands, &room_state, room_state.room_index);

            break;
        }
    }
}
