use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, RunData, player::Player, floor_complete::FloorCompleteState,
    hazards::{spawn_lava_strip, spawn_water_strip, spawn_moving_platform, spawn_arrow_trap, spawn_spike_floor, spawn_poison_cloud},
    relic::{RelicChoiceState, RelicInventory, start_relic_choice},
    altar::{AltarState, AltarEntity, spawn_altar, check_altar_interaction},
    audio::{PlaySfxEvent, SfxType},
    well::{WellAssets, spawn_well}};

// ─── Tileset Assets ──────────────────────────────────────────────────────────

const TILESET_PATH: &str = "tilesets/0x72_DungeonTilesetII_v1.7/0x72_DungeonTilesetII_v1.7/frames";

/// Pre-loaded tile texture handles from the 0x72 Dungeon Tileset.
#[derive(Resource)]
pub struct TilesetAssets {
    /// 8 floor tile variants (floor_1 .. floor_8).
    pub floors: Vec<Handle<Image>>,
    /// Wall body tile (wall_mid).
    pub wall_mid: Handle<Image>,
    /// Wall top tile (wall_top_mid).
    pub wall_top: Handle<Image>,
}

pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        // Load tileset images.
        let asset_server = app.world().resource::<AssetServer>();
        let floors: Vec<Handle<Image>> = (1..=8)
            .map(|i| asset_server.load(format!("{TILESET_PATH}/floor_{i}.png")))
            .collect();
        let wall_mid = asset_server.load(format!("{TILESET_PATH}/wall_mid.png"));
        let wall_top = asset_server.load(format!("{TILESET_PATH}/wall_top_mid.png"));
        app.insert_resource(TilesetAssets { floors, wall_mid, wall_top });

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
                    check_altar_proximity,
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

/// Destructible wall that can be broken by melee attacks.
#[derive(Component)]
pub struct DestructibleWall {
    pub health: i32,
}

/// Hidden loot spawned when a destructible wall breaks.
#[derive(Component)]
#[allow(dead_code)]
pub struct SecretLootMarker;

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
    Altar,
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
        } else if i == combat_count - 1 && floor >= 2 {
            // Altar room before shop on floor 2+
            layout.push(RoomType::Altar);
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
    resuming: Option<Res<crate::ResumingFromPause>>,
    tiles: Res<TilesetAssets>,
    well_assets: Res<WellAssets>,
) {
    if resuming.is_some() { return; }
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

    spawn_room(&mut commands, &state, 0, &tiles, &well_assets);
}

// ─── Room Spawning ────────────────────────────────────────────────────────────

fn spawn_room(commands: &mut Commands, state: &RoomState, room_idx: usize, tiles: &TilesetAssets, well_assets: &WellAssets) {
    let room_type = state.floor_layout.get(room_idx).copied().unwrap_or(RoomType::Combat);
    let seed = state.seed
        .wrapping_add(room_idx as u64)
        .wrapping_mul(state.floor as u64 + 1);

    let bg_color = biome_bg_color(room_type, state.floor);

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

    // Keep biome colors for decorations (stalactites, rubble, etc.)
    let (floor_color, _ceil_color, wall_color) = room_tile_colors(room_type, state.floor);

    // Floor row — tileset floor tiles (randomized variant per column)
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile_sprite(commands, x, TILE_SIZE / 2.0, pick_floor_tile(tiles, col, 0));
    }

    // Ceiling row — wall_top tiles
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile_sprite(commands, x, ROOM_H - TILE_SIZE / 2.0, tiles.wall_top.clone());
    }

    // Left wall — wall_mid tiles
    for row in 0..ROOM_ROWS {
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile_sprite(commands, TILE_SIZE / 2.0, y, tiles.wall_mid.clone());
    }

    // Right wall (gap at rows 1-3 for door) — wall_mid tiles
    for row in 0..ROOM_ROWS {
        if row >= 1 && row <= 3 { continue; }
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile_sprite(commands, ROOM_W - TILE_SIZE / 2.0, y, tiles.wall_mid.clone());
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

    // Boss room: dramatic reddish tint overlay on all tiles
    if room_type == RoomType::Boss {
        commands.spawn((
            Sprite {
                color: Color::srgba(0.4, 0.0, 0.0, 0.15),
                custom_size: Some(Vec2::new(ROOM_W, ROOM_H)),
                ..default()
            },
            Transform::from_xyz(ROOM_W / 2.0, ROOM_H / 2.0, Z_TILES + 0.05),
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

    // Biome-specific ambient decorations
    spawn_biome_decorations(commands, seed, state.floor, room_type);

    // Room-specific content
    match room_type {
        RoomType::Start    => spawn_start_room(commands, seed, tiles),
        RoomType::Combat   => spawn_combat_room(commands, seed, state.floor, tiles),
        RoomType::Treasure => spawn_treasure_room(commands, seed, tiles),
        RoomType::Boss     => spawn_boss_room(commands, state.floor, tiles),
        RoomType::Shop     => spawn_shop_room(commands, seed, tiles),
        RoomType::Altar    => spawn_altar_room(commands, seed, tiles),
    }

    // Healing well: 30% chance in Combat and Treasure rooms (not Boss/Shop)
    if matches!(room_type, RoomType::Combat | RoomType::Treasure) {
        let well_roll = seed.wrapping_mul(48271).wrapping_add(room_idx as u64 * 101);
        if well_roll % 100 < 30 {
            let well_col = 5 + (well_roll % 14) as i32; // cols 5-18
            let well_x = well_col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
            let well_y = TILE_SIZE; // on the floor
            spawn_well(commands, well_x, well_y, well_assets);
        }
    }
}

// ─── Tile Color Palette ───────────────────────────────────────────────────────

/// Floor-based biome background colors.
fn biome_bg_color(room_type: RoomType, floor: i32) -> Color {
    let biome = (floor - 1).clamp(0, 3);
    match room_type {
        RoomType::Boss => match biome {
            1 => Color::srgb(0.06, 0.08, 0.04), // mushroom boss
            2 => Color::srgb(0.12, 0.04, 0.02), // lava boss
            3 => Color::srgb(0.04, 0.06, 0.04), // root boss
            _ => Color::srgb(0.10, 0.04, 0.03), // stone boss
        },
        RoomType::Altar => match biome {
            1 => Color::srgb(0.04, 0.06, 0.08),
            2 => Color::srgb(0.08, 0.03, 0.06),
            3 => Color::srgb(0.03, 0.05, 0.06),
            _ => Color::srgb(0.06, 0.04, 0.08),
        },
        RoomType::Shop => Color::srgb(0.10, 0.06, 0.03), // warm campfire amber
        _ => match biome {
            1 => Color::srgb(0.05, 0.07, 0.04), // mushroom cave: earthy green
            2 => Color::srgb(0.09, 0.04, 0.02), // lava depths: dark red
            3 => Color::srgb(0.03, 0.05, 0.04), // root heart: dark organic
            _ => Color::srgb(0.08, 0.06, 0.04), // stone ruins: default
        },
    }
}

/// Floor-based biome tile colors: (floor, ceiling, walls).
fn room_tile_colors(room_type: RoomType, floor: i32) -> (Color, Color, Color) {
    let biome = (floor - 1).clamp(0, 3);

    // Altar always uses its mystic palette
    if room_type == RoomType::Altar {
        return match biome {
            1 => (Color::srgb(0.18, 0.22, 0.18), Color::srgb(0.14, 0.18, 0.14), Color::srgb(0.16, 0.20, 0.16)),
            2 => (Color::srgb(0.25, 0.15, 0.22), Color::srgb(0.20, 0.10, 0.18), Color::srgb(0.22, 0.12, 0.20)),
            3 => (Color::srgb(0.15, 0.20, 0.18), Color::srgb(0.12, 0.16, 0.14), Color::srgb(0.14, 0.18, 0.16)),
            _ => (Color::srgb(0.22, 0.18, 0.28), Color::srgb(0.18, 0.14, 0.24), Color::srgb(0.20, 0.16, 0.26)),
        };
    }

    // Boss rooms use dramatic biome colors
    if room_type == RoomType::Boss {
        return match biome {
            1 => (Color::srgb(0.20, 0.25, 0.12), Color::srgb(0.16, 0.20, 0.08), Color::srgb(0.18, 0.22, 0.10)),
            2 => (Color::srgb(0.32, 0.12, 0.06), Color::srgb(0.26, 0.08, 0.04), Color::srgb(0.28, 0.10, 0.05)),
            3 => (Color::srgb(0.14, 0.22, 0.14), Color::srgb(0.10, 0.18, 0.10), Color::srgb(0.12, 0.20, 0.12)),
            _ => (Color::srgb(0.28, 0.14, 0.08), Color::srgb(0.22, 0.10, 0.06), Color::srgb(0.25, 0.12, 0.07)),
        };
    }

    // All other room types use biome-based stone palette
    match biome {
        // Floor 2: Mushroom Cave — earthy greens and browns
        1 => (
            Color::srgb(0.22, 0.24, 0.14),  // floor: mossy stone
            Color::srgb(0.16, 0.18, 0.10),  // ceiling: dark green stone
            Color::srgb(0.19, 0.21, 0.12),  // walls: earthy
        ),
        // Floor 3: Lava Depths — reds and dark oranges
        2 => (
            Color::srgb(0.28, 0.16, 0.10),  // floor: volcanic rock
            Color::srgb(0.22, 0.10, 0.06),  // ceiling: charred
            Color::srgb(0.25, 0.13, 0.08),  // walls: basalt
        ),
        // Floor 4: Root Heart — dark organic greens and purples
        3 => (
            Color::srgb(0.16, 0.22, 0.16),  // floor: root-covered
            Color::srgb(0.10, 0.16, 0.12),  // ceiling: organic canopy
            Color::srgb(0.13, 0.19, 0.14),  // walls: root-woven
        ),
        // Floor 1: Stone Ruins (default)
        _ => match room_type {
            RoomType::Start    => (Color::srgb(0.28, 0.22, 0.15), Color::srgb(0.22, 0.17, 0.12), Color::srgb(0.24, 0.19, 0.14)),
            RoomType::Combat   => (Color::srgb(0.26, 0.20, 0.14), Color::srgb(0.20, 0.15, 0.10), Color::srgb(0.23, 0.18, 0.12)),
            RoomType::Treasure => (Color::srgb(0.28, 0.24, 0.16), Color::srgb(0.22, 0.19, 0.13), Color::srgb(0.26, 0.22, 0.15)),
            RoomType::Shop     => (Color::srgb(0.30, 0.18, 0.12), Color::srgb(0.22, 0.13, 0.08), Color::srgb(0.26, 0.15, 0.10)),
            _ => (Color::srgb(0.26, 0.20, 0.14), Color::srgb(0.20, 0.15, 0.10), Color::srgb(0.23, 0.18, 0.12)),
        },
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

/// Spawn a tile using a tileset texture image instead of a colored rectangle.
fn spawn_tile_sprite(commands: &mut Commands, x: f32, y: f32, texture: Handle<Image>) {
    commands.spawn((
        Sprite {
            image: texture,
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES),
        Tile,
        RoomEntity,
        PlayingEntity,
    ));
}

/// Pick a deterministic floor tile variant based on position.
fn pick_floor_tile(tiles: &TilesetAssets, col: i32, row: i32) -> Handle<Image> {
    let hash = ((col as u32).wrapping_mul(7) ^ (row as u32).wrapping_mul(13)) as usize;
    tiles.floors[hash % tiles.floors.len()].clone()
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

fn spawn_platform(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color, tiles: &TilesetAssets) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let len = col_end - col_start + 1;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let is_left  = col == col_start;
        let is_right = col == col_end;
        spawn_tile_sprite(commands, x, y, pick_floor_tile(tiles, col, row));
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

fn spawn_platform_worn(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color, tiles: &TilesetAssets) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    let len = col_end - col_start + 1;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let is_left  = col == col_start;
        let is_right = col == col_end;
        spawn_tile_sprite(commands, x, y, pick_floor_tile(tiles, col, row));
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
/// Spawn a destructible wall section (cracked wall with secret behind it).
fn spawn_secret_wall(commands: &mut Commands, _floor: i32, seed: u64) {
    // Decide which wall (left or right) based on seed
    let on_right = seed % 2 == 0;
    let wall_x = if on_right { ROOM_W - TILE_SIZE / 2.0 } else { TILE_SIZE / 2.0 };
    let row = 4 + (seed % 3) as i32; // Row 4-6 (above floor, below ceiling)
    let wall_y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

    // Cracked wall tile (replaces normal wall at this position)
    let crack_color = Color::srgb(0.30, 0.24, 0.16);
    let crack_line = Color::srgba(0.0, 0.0, 0.0, 0.3);

    commands.spawn((
        Sprite {
            color: crack_color,
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        },
        Transform::from_xyz(wall_x, wall_y, Z_TILES + 0.01),
        DestructibleWall { health: 3 },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Crack pattern overlay
        parent.spawn((
            Sprite { color: crack_line, custom_size: Some(Vec2::new(TILE_SIZE * 0.6, 2.0)), ..default() },
            Transform::from_xyz(-2.0, 4.0, 0.1),
        ));
        parent.spawn((
            Sprite { color: crack_line, custom_size: Some(Vec2::new(2.0, TILE_SIZE * 0.5)), ..default() },
            Transform::from_xyz(4.0, -3.0, 0.1),
        ));
        parent.spawn((
            Sprite { color: crack_line, custom_size: Some(Vec2::new(TILE_SIZE * 0.4, 2.0)), ..default() },
            Transform::from_xyz(3.0, -6.0, 0.1),
        ));
        // Glowing hint behind the cracks
        parent.spawn((
            Sprite {
                color: Color::srgba(1.0, 0.85, 0.2, 0.08),
                custom_size: Some(Vec2::new(TILE_SIZE * 0.8, TILE_SIZE * 0.8)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.1),
            CrystalGlow { timer: 0.0, phase: seed as f32 * 0.1 },
        ));
    });
}

/// Spawn biome-specific decorations based on floor number.
fn spawn_biome_decorations(commands: &mut Commands, seed: u64, floor: i32, room_type: RoomType) {
    let biome = (floor - 1).clamp(0, 3);
    if room_type == RoomType::Boss || room_type == RoomType::Shop { return; }

    match biome {
        1 => {
            // Mushroom Cave: extra mushrooms, glowing moss patches
            let moss_color = Color::srgba(0.3, 0.6, 0.2, 0.2);
            let positions = [(3.0, 1.0), (8.0, 1.0), (14.0, 1.0), (19.0, 1.0)];
            for (i, &(col, _row)) in positions.iter().enumerate() {
                let hash = seed.wrapping_add(i as u64 * 37);
                if hash % 3 != 0 { continue; }
                let x = col * TILE_SIZE + TILE_SIZE / 2.0;
                spawn_mushroom(commands, x, TILE_SIZE);
            }
            // Glowing moss patches on floor
            for i in 0..3 {
                let hash = seed.wrapping_add(i * 53 + 7);
                let x = (2.0 + (hash % 18) as f32) * TILE_SIZE + TILE_SIZE / 2.0;
                commands.spawn((
                    Sprite {
                        color: moss_color,
                        custom_size: Some(Vec2::new(TILE_SIZE * 1.5, TILE_SIZE * 0.3)),
                        ..default()
                    },
                    Transform::from_xyz(x, TILE_SIZE + 2.0, Z_TILES + 0.2),
                    RoomEntity,
                    PlayingEntity,
                ));
            }
        }
        2 => {
            // Lava Depths: ember particles, lava glow on floor edges
            let ember_color = Color::srgba(1.0, 0.5, 0.1, 0.25);
            let glow_color = Color::srgba(0.9, 0.3, 0.05, 0.12);
            // Floor lava glow
            for i in 0..4 {
                let hash = seed.wrapping_add(i * 41);
                let x = (3.0 + (hash % 16) as f32) * TILE_SIZE + TILE_SIZE / 2.0;
                commands.spawn((
                    Sprite {
                        color: glow_color,
                        custom_size: Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 0.5)),
                        ..default()
                    },
                    Transform::from_xyz(x, TILE_SIZE + 4.0, Z_TILES + 0.15),
                    RoomEntity,
                    PlayingEntity,
                ));
            }
            // Floating embers
            for i in 0..5 {
                let hash = seed.wrapping_add(i * 67);
                let x = (2.0 + (hash % 20) as f32) * TILE_SIZE;
                let y = TILE_SIZE * 3.0 + (hash % 200) as f32;
                commands.spawn((
                    Sprite {
                        color: ember_color,
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    Transform::from_xyz(x, y, Z_TILES + 0.3),
                    RoomEntity,
                    PlayingEntity,
                ));
            }
        }
        3 => {
            // Root Heart: root tendrils from ceiling and floor
            let root_color = Color::srgb(0.15, 0.25, 0.12);
            let root_hi = Color::srgba(0.3, 0.5, 0.2, 0.3);
            // Ceiling roots hanging down
            for i in 0..5 {
                let hash = seed.wrapping_add(i * 79);
                let x = (3.0 + (hash % 18) as f32) * TILE_SIZE;
                let h = 20.0 + (hash % 40) as f32;
                commands.spawn((
                    Sprite {
                        color: root_color,
                        custom_size: Some(Vec2::new(4.0, h)),
                        ..default()
                    },
                    Transform::from_xyz(x, ROOM_H - TILE_SIZE - h / 2.0, Z_TILES + 0.25),
                    RoomEntity,
                    PlayingEntity,
                ));
            }
            // Floor roots growing up
            for i in 0..3 {
                let hash = seed.wrapping_add(i * 101 + 33);
                let x = (4.0 + (hash % 16) as f32) * TILE_SIZE;
                let h = 15.0 + (hash % 25) as f32;
                commands.spawn((
                    Sprite {
                        color: root_color,
                        custom_size: Some(Vec2::new(3.0, h)),
                        ..default()
                    },
                    Transform::from_xyz(x, TILE_SIZE + h / 2.0, Z_TILES + 0.25),
                    RoomEntity,
                    PlayingEntity,
                ));
            }
            // Pulsing organic glow patches
            for i in 0..2 {
                let hash = seed.wrapping_add(i * 59);
                let x = (5.0 + (hash % 14) as f32) * TILE_SIZE;
                commands.spawn((
                    Sprite {
                        color: root_hi,
                        custom_size: Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE * 0.5)),
                        ..default()
                    },
                    Transform::from_xyz(x, TILE_SIZE + 3.0, Z_TILES + 0.2),
                    CrystalGlow { timer: i as f32 * 1.5, phase: i as f32 * 1.5 },
                    RoomEntity,
                    PlayingEntity,
                ));
            }
        }
        _ => {} // Floor 1 Stone Ruins uses existing decorations
    }
}

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

fn spawn_start_room(commands: &mut Commands, seed: u64, tiles: &TilesetAssets) {
    let plat_color = Color::srgb(0.34, 0.26, 0.16);

    // Three low, welcoming stepping-stone platforms (rows 2-4, spans 3-4 tiles)
    spawn_platform(commands, 3,  6,  2, plat_color, tiles); // left, very low
    spawn_platform(commands, 9,  13, 3, plat_color, tiles); // center, one step up
    spawn_platform(commands, 16, 19, 2, plat_color, tiles); // right, back low

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

fn spawn_combat_room(commands: &mut Commands, seed: u64, floor: i32, tiles: &TilesetAssets) {
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
            spawn_platform_worn(commands, 2,  5,  2, plat_color, tiles); // very low left
            spawn_platform_worn(commands, 7,  10, 3, plat_color, tiles); // step up
            spawn_platform_worn(commands, 12, 15, 4, plat_color, tiles); // center
            spawn_platform_worn(commands, 17, 20, 2, plat_color, tiles); // low right

            // Torches on left and right walls only
            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.5);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 3.5);
            spawn_crystal(commands, TILE_SIZE * 3.5, TILE_SIZE * 3.0, 0.5);
        }

        // ── 1: Multi-level arena ─────────────────────────────────────────────
        1 => {
            spawn_platform_worn(commands, 2,  5,  2, plat_color, tiles);
            spawn_platform_worn(commands, 8,  12, 4, plat_color, tiles);
            spawn_platform_worn(commands, 15, 18, 2, plat_color, tiles);
            spawn_platform_worn(commands, 5,  9,  5, plat_color, tiles); // medium high (was row 6)

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_mushroom(commands, TILE_SIZE * 6.5, TILE_SIZE * 2.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 10.0, TILE_SIZE * 4.0 + TILE_SIZE);
        }

        // ── 2: Lava pits with narrow bridges ──────────────────────────────────
        2 => {
            spawn_platform(commands, 3,  5,  3, plat_color, tiles);
            spawn_platform(commands, 9,  11, 3, plat_color, tiles);
            spawn_platform(commands, 15, 17, 3, plat_color, tiles);
            // Lower bridge tiles over lava pits
            spawn_platform(commands, 6,  8,  2, plat_color, tiles);
            spawn_platform(commands, 12, 14, 2, plat_color, tiles);
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
            spawn_platform(commands, 2,  4,  2, plat_color, tiles);
            spawn_platform(commands, 6,  8,  4, plat_color, tiles);
            spawn_platform(commands, 10, 12, 2, plat_color, tiles);
            spawn_platform(commands, 14, 16, 5, plat_color, tiles);
            spawn_platform(commands, 18, 21, 3, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.5);
        }

        // ── 4: Zigzag platforms ──────────────────────────────────────────────
        4 => {
            spawn_platform(commands, 2,  4,  2, plat_color, tiles);
            spawn_platform(commands, 6,  8,  4, plat_color, tiles);
            spawn_platform(commands, 10, 13, 2, plat_color, tiles);
            spawn_platform(commands, 14, 16, 4, plat_color, tiles);
            spawn_platform(commands, 18, 21, 3, plat_color, tiles);
            // A medium-high ledge for optional routing (was row 7)
            spawn_platform(commands, 9, 13, 5, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 11.0, TILE_SIZE * 8.0, 1.5);
            spawn_mushroom(commands, TILE_SIZE * 7.5, TILE_SIZE * 4.0 + TILE_SIZE);
        }

        // ── 5: Floating stones + moving platform ─────────────────────────────
        5 => {
            spawn_platform(commands, 2,  4,  3, plat_color, tiles);
            spawn_platform(commands, 7,  10, 5, plat_color, tiles);
            spawn_platform(commands, 13, 16, 3, plat_color, tiles);
            spawn_platform(commands, 18, 21, 4, plat_color, tiles);
            // Tiny single-tile mid-gap
            spawn_platform(commands, 5,  6,  4, plat_color, tiles);
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
            // Arrow traps on walls (floor 2+)
            if floor >= 2 {
                spawn_arrow_trap(commands, 1, 4, 1.0);  // left wall, shoots right
                spawn_arrow_trap(commands, 22, 3, -1.0); // right wall, shoots left
            }
        }

        // ── 6: Low walkways with swampy water gap ─────────────────────────────
        6 => {
            // Long walkways left and right, gap in middle
            spawn_platform_worn(commands, 2,  9,  3, plat_color, tiles);
            spawn_platform_worn(commands, 13, 21, 3, plat_color, tiles);
            // Small mid platform to bridge the gap
            spawn_platform(commands, 10, 12, 4, plat_color, tiles);
            // Optional high ledge (was row 6)
            spawn_platform(commands, 7, 10, 5, plat_color, tiles);
            // Swampy water in the central gap (floor level)
            spawn_water_strip(commands, 10, 12, 1);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 2.5, TILE_SIZE * 4.0, 1.8);
        }

        // ── 7: Tunnel / open corridor with overhangs ────────────────────────
        7 => {
            // Partial overhangs instead of sealed ceiling — gaps allow flight
            spawn_platform(commands, 4,  9,  8, plat_color, tiles);  // left overhang
            spawn_platform(commands, 14, 19, 8, plat_color, tiles);  // right overhang
            // Ground bumps inside the corridor
            spawn_platform(commands, 5,  7,  2, plat_color, tiles);
            spawn_platform(commands, 11, 13, 3, plat_color, tiles);
            spawn_platform(commands, 16, 18, 2, plat_color, tiles);
            // Mid-height stepping stone in the gap for player to reach flyers
            spawn_platform(commands, 10, 13, 6, plat_color, tiles);
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
            spawn_platform(commands, 2,  4,  2, plat_color, tiles);
            spawn_platform(commands, 7,  9,  3, plat_color, tiles);
            spawn_platform(commands, 11, 14, 2, plat_color, tiles);
            spawn_platform(commands, 17, 20, 3, plat_color, tiles);
            // High escape route
            spawn_platform(commands, 5,  6,  5, plat_color, tiles);
            spawn_platform(commands, 15, 16, 5, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            // Spike floors on island edges (floor 2+)
            if floor >= 2 {
                spawn_spike_floor(commands, 5, 2);
                spawn_spike_floor(commands, 10, 2);
                spawn_spike_floor(commands, 16, 3);
            }
        }

        // ── 9: Swamp marsh – water-filled lower area ────────────────────────
        9 => {
            // Water across the floor
            spawn_water_strip(commands, 3, 20, 1);
            // Raised dry platforms
            spawn_platform_worn(commands, 2,  5,  3, plat_color, tiles);
            spawn_platform_worn(commands, 8,  11, 2, plat_color, tiles);
            spawn_platform_worn(commands, 14, 17, 3, plat_color, tiles);
            spawn_platform_worn(commands, 19, 21, 2, plat_color, tiles);
            // Upper catwalk
            spawn_platform(commands, 6, 8, 5, plat_color, tiles);
            spawn_platform(commands, 12, 14, 5, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_mushroom(commands, TILE_SIZE * 3.5, TILE_SIZE * 3.0 + TILE_SIZE);
            spawn_mushroom(commands, TILE_SIZE * 15.5, TILE_SIZE * 3.0 + TILE_SIZE);
            // Poison clouds in the swamp (floor 2+)
            if floor >= 2 {
                spawn_poison_cloud(commands, TILE_SIZE * 7.0, TILE_SIZE * 2.0);
                spawn_poison_cloud(commands, TILE_SIZE * 16.0, TILE_SIZE * 2.0);
            }
        }

        // ── 10: Vertical elevator shaft – multiple moving platforms ─────────
        10 => {
            // Ground-level platforms on sides
            spawn_platform(commands, 2,  5,  2, plat_color, tiles);
            spawn_platform(commands, 18, 21, 2, plat_color, tiles);
            // High ledges on sides
            spawn_platform(commands, 2,  4,  5, plat_color, tiles);
            spawn_platform(commands, 19, 21, 5, plat_color, tiles);
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
            spawn_platform(commands, 2, 6, 2, plat_color, tiles);
            spawn_water_strip(commands, 7, 16, 1);
            spawn_platform(commands, 17, 21, 2, plat_color, tiles);
            // High road (upper platforms)
            spawn_platform(commands, 3,  5,  4, plat_color, tiles);
            spawn_platform(commands, 7,  10, 5, plat_color, tiles);
            spawn_platform(commands, 12, 15, 5, plat_color, tiles);
            spawn_platform(commands, 17, 20, 4, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 11.0, TILE_SIZE * 6.0, 0.7);
            spawn_mushroom(commands, TILE_SIZE * 4.0, TILE_SIZE * 2.0 + TILE_SIZE);
        }

        // ── 12: Pillared hall – dense columns creating corridors ─────────────
        12 => {
            // Ground platforms between pillars
            spawn_platform(commands, 2, 21, 2, plat_color, tiles);
            // Pillars creating corridors
            spawn_pillar(commands, 5,  3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 10, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 15, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            spawn_pillar(commands, 20, 3, 7, Color::srgb(0.25, 0.18, 0.10));
            // Mid-height walkways between pillars
            spawn_platform(commands, 6,  9,  5, plat_color, tiles);
            spawn_platform(commands, 11, 14, 5, plat_color, tiles);
            spawn_platform(commands, 16, 19, 5, plat_color, tiles);

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
            spawn_platform_worn(commands, 2,  5,  3, plat_color, tiles);
            spawn_platform_worn(commands, 7,  9,  4, plat_color, tiles);
            spawn_platform_worn(commands, 11, 13, 3, plat_color, tiles);
            spawn_platform_worn(commands, 15, 18, 5, plat_color, tiles);
            spawn_platform_worn(commands, 19, 21, 3, plat_color, tiles);
            // Collapsed pillar (decorative, acts as platform)
            spawn_platform(commands, 10, 10, 2, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 4.0);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);
            spawn_crystal(commands, TILE_SIZE * 6.0, TILE_SIZE * 4.0, 3.1);
        }

        // ── 14: The pit – deep central chasm with bridges ───────────────────
        14 => {
            // Solid ground on sides
            spawn_platform(commands, 2,  7,  2, plat_color, tiles);
            spawn_platform(commands, 16, 21, 2, plat_color, tiles);
            // Lava in the pit
            spawn_lava_strip(commands, 8, 15, 1);
            // Narrow bridge across
            spawn_platform(commands, 9,  10, 3, plat_color, tiles);
            spawn_platform(commands, 13, 14, 3, plat_color, tiles);
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
            spawn_platform(commands, 5,  7,  5, plat_color, tiles);
            spawn_platform(commands, 16, 18, 5, plat_color, tiles);

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
            spawn_platform(commands, 2,  4,  3, plat_color, tiles);
            spawn_platform(commands, 6,  8,  2, plat_color, tiles);
            spawn_platform(commands, 10, 12, 4, plat_color, tiles);
            spawn_platform(commands, 14, 16, 2, plat_color, tiles);
            spawn_platform(commands, 18, 21, 3, plat_color, tiles);
            // High bridge
            spawn_platform(commands, 8, 14, 5, plat_color, tiles);

            spawn_wall_torch(commands, LEFT_WALL_TORCH_X,   TILE_SIZE * 3.5);
            spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 3.5);
            spawn_mushroom(commands, TILE_SIZE * 7.0, TILE_SIZE * 2.0 + TILE_SIZE);
            spawn_crystal(commands, TILE_SIZE * 18.0, TILE_SIZE * 3.0 + TILE_SIZE, 2.0);
        }
    }

    if floor >= 3 {
        spawn_crystal(commands, TILE_SIZE * 2.0, TILE_SIZE, 3.0);
    }

    // Secret destructible wall (30% chance on floor 2+)
    if floor >= 2 {
        let secret_hash = seed.wrapping_mul(7919).wrapping_add(42);
        if secret_hash % 100 < 30 {
            spawn_secret_wall(commands, floor, seed);
        }
    }
}

// ─── Treasure Room ────────────────────────────────────────────────────────────

fn spawn_altar_room(commands: &mut Commands, seed: u64, tiles: &TilesetAssets) {
    let plat_color = Color::srgb(0.28, 0.22, 0.32);

    // Central raised platform with the altar
    spawn_platform(commands, 8, 15, 2, plat_color, tiles);
    // Steps up from left
    spawn_platform(commands, 5, 7, 2, plat_color, tiles);
    // Steps up from right
    spawn_platform(commands, 16, 18, 2, plat_color, tiles);

    // Spawn the altar in the center of the room
    let altar_x = ROOM_W / 2.0;
    let altar_y = TILE_SIZE * 3.0;
    spawn_altar(commands, altar_x, altar_y);

    // Atmospheric decorations
    // Purple crystals flanking the altar
    spawn_crystal(commands, altar_x - 80.0, TILE_SIZE * 2.5, 0.0);
    spawn_crystal(commands, altar_x + 80.0, TILE_SIZE * 2.5, 1.5);
    // Torches on walls
    spawn_wall_torch(commands, LEFT_WALL_TORCH_X, TILE_SIZE * 4.0);
    spawn_wall_torch(commands, right_wall_torch_x(), TILE_SIZE * 4.0);

    // Runic floor decorations
    let rune_color = Color::srgba(0.6, 0.4, 0.9, 0.15);
    for i in 0..4 {
        let rx = altar_x + (i as f32 - 1.5) * 30.0;
        commands.spawn((
            Sprite {
                color: rune_color,
                custom_size: Some(Vec2::new(8.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(rx, TILE_SIZE + 2.0, Z_TILES + 0.3),
            RoomEntity,
            PlayingEntity,
        ));
    }

    // Mystical candles near the altar
    for dx in [-50.0_f32, 50.0] {
        let cx = altar_x + dx;
        // Candle body
        commands.spawn((
            Sprite {
                color: Color::srgb(0.80, 0.78, 0.70),
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(cx, TILE_SIZE * 2.0 + 12.0, Z_TILES + 0.5),
            RoomEntity,
            PlayingEntity,
        ));
        // Candle flame
        commands.spawn((
            Sprite {
                color: Color::srgba(0.8, 0.5, 1.0, 0.85),
                custom_size: Some(Vec2::new(4.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(cx, TILE_SIZE * 2.0 + 22.0, Z_TILES + 0.6),
            TorchFlicker { timer: seed as f32 * 0.1 + dx, base_alpha: 0.85 },
            RoomEntity,
            PlayingEntity,
        ));
    }
}

/// Shop room: warm safe haven with carpet, crates, shelves, and glowing lanterns.
fn spawn_shop_room(commands: &mut Commands, _seed: u64, tiles: &TilesetAssets) {
    let plat_color = Color::srgb(0.32, 0.22, 0.14);

    // Raised platform in center for merchant to stand on
    spawn_platform(commands, 9, 14, 2, plat_color, tiles);
    // Side ledges
    spawn_platform(commands, 3, 6, 3, plat_color, tiles);
    spawn_platform(commands, 17, 20, 3, plat_color, tiles);

    // ── Podest dark borders + small carpet before each podest ──
    let border_color = Color::srgb(0.18, 0.12, 0.06);
    let carpet_rich = Color::srgba(0.60, 0.12, 0.08, 0.45);
    let carpet_fringe = Color::srgba(0.75, 0.55, 0.15, 0.35);
    // Center podest border (bottom edge)
    let center_y = 2.0 * TILE_SIZE;
    commands.spawn((
        Sprite { color: border_color, custom_size: Some(Vec2::new(TILE_SIZE * 6.2, 4.0)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0, center_y + 1.0, Z_TILES + 0.09),
        RoomEntity, PlayingEntity,
    ));
    // Carpet in front of center podest
    commands.spawn((
        Sprite { color: carpet_rich, custom_size: Some(Vec2::new(TILE_SIZE * 5.0, TILE_SIZE * 0.3)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0, TILE_SIZE + TILE_SIZE * 0.3, Z_TILES + 0.06),
        RoomEntity, PlayingEntity,
    ));
    commands.spawn((
        Sprite { color: carpet_fringe, custom_size: Some(Vec2::new(TILE_SIZE * 5.4, TILE_SIZE * 0.1)), ..default() },
        Transform::from_xyz(ROOM_W / 2.0, TILE_SIZE + TILE_SIZE * 0.15, Z_TILES + 0.07),
        RoomEntity, PlayingEntity,
    ));
    // Side podest borders + carpets
    for &(col_s, col_e) in &[(3, 6), (17, 20)] {
        let px = ((col_s + col_e) as f32 / 2.0) * TILE_SIZE + TILE_SIZE / 2.0;
        let py = 3.0 * TILE_SIZE;
        let pw = (col_e - col_s + 1) as f32 * TILE_SIZE + 4.0;
        commands.spawn((
            Sprite { color: border_color, custom_size: Some(Vec2::new(pw, 4.0)), ..default() },
            Transform::from_xyz(px, py + 1.0, Z_TILES + 0.09),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: carpet_rich, custom_size: Some(Vec2::new(pw - 20.0, TILE_SIZE * 0.25)), ..default() },
            Transform::from_xyz(px, TILE_SIZE + TILE_SIZE * 0.3, Z_TILES + 0.06),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Elevated center glow (highlights merchant area) ──
    commands.spawn((
        Sprite {
            color: Color::srgba(0.95, 0.70, 0.25, 0.05),
            custom_size: Some(Vec2::new(TILE_SIZE * 8.0, TILE_SIZE * 5.0)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0, TILE_SIZE * 3.5, Z_TILES + 0.02),
        RoomEntity, PlayingEntity,
    ));

    // ── Wall torches with flickering glow ──
    let torch_wall_y = TILE_SIZE * 6.5;
    for &tx in &[TILE_SIZE * 3.5, TILE_SIZE * 8.5, TILE_SIZE * 15.0, TILE_SIZE * 20.0] {
        spawn_wall_torch(commands, tx, torch_wall_y);
    }

    // ── SHOP banner above merchant position ──
    let banner_x = ROOM_W / 2.0;
    let banner_y = TILE_SIZE * 7.5;
    // Banner cloth
    commands.spawn((
        Sprite { color: Color::srgb(0.55, 0.15, 0.10), custom_size: Some(Vec2::new(56.0, 18.0)), ..default() },
        Transform::from_xyz(banner_x, banner_y, Z_TILES + 0.4),
        RoomEntity, PlayingEntity,
    ));
    // Banner border (gold trim)
    commands.spawn((
        Sprite { color: Color::srgb(0.80, 0.60, 0.15), custom_size: Some(Vec2::new(58.0, 2.0)), ..default() },
        Transform::from_xyz(banner_x, banner_y + 9.0, Z_TILES + 0.42),
        RoomEntity, PlayingEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.80, 0.60, 0.15), custom_size: Some(Vec2::new(58.0, 2.0)), ..default() },
        Transform::from_xyz(banner_x, banner_y - 9.0, Z_TILES + 0.42),
        RoomEntity, PlayingEntity,
    ));
    // Hanging ropes from banner to ceiling
    for &rx in &[banner_x - 26.0, banner_x + 26.0] {
        commands.spawn((
            Sprite { color: Color::srgb(0.45, 0.35, 0.20), custom_size: Some(Vec2::new(2.0, ROOM_H - TILE_SIZE - banner_y - 9.0)), ..default() },
            Transform::from_xyz(rx, (banner_y + 9.0 + ROOM_H - TILE_SIZE) / 2.0, Z_TILES + 0.35),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Hanging curtains/drapes from ceiling ──
    let curtain_color = Color::srgba(0.50, 0.14, 0.10, 0.55);
    let curtain_dark = Color::srgba(0.35, 0.08, 0.06, 0.50);
    for &cx in &[TILE_SIZE * 5.5, TILE_SIZE * 18.0] {
        let cy_top = ROOM_H - TILE_SIZE - 2.0;
        let drape_h = TILE_SIZE * 3.5;
        // Main drape
        commands.spawn((
            Sprite { color: curtain_color, custom_size: Some(Vec2::new(18.0, drape_h)), ..default() },
            Transform::from_xyz(cx, cy_top - drape_h / 2.0, Z_TILES + 0.25),
            RoomEntity, PlayingEntity,
        ));
        // Dark fold line
        commands.spawn((
            Sprite { color: curtain_dark, custom_size: Some(Vec2::new(3.0, drape_h - 4.0)), ..default() },
            Transform::from_xyz(cx + 4.0, cy_top - drape_h / 2.0, Z_TILES + 0.26),
            RoomEntity, PlayingEntity,
        ));
        // Gold tie
        commands.spawn((
            Sprite { color: Color::srgb(0.80, 0.60, 0.15), custom_size: Some(Vec2::new(12.0, 3.0)), ..default() },
            Transform::from_xyz(cx, cy_top - drape_h + 6.0, Z_TILES + 0.27),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Ceiling chains (shop-specific, fewer than combat rooms) ──
    let chain_color = Color::srgba(0.50, 0.45, 0.40, 0.70);
    for &chain_x in &[TILE_SIZE * 10.0, TILE_SIZE * 13.5] {
        for link in 0..3_u32 {
            let (w, h) = if link % 2 == 0 { (3.0, 8.0) } else { (6.0, 3.0) };
            let y = ROOM_H - TILE_SIZE - 6.0 - link as f32 * 9.0;
            commands.spawn((
                Sprite { color: chain_color, custom_size: Some(Vec2::new(w, h)), ..default() },
                Transform::from_xyz(chain_x, y, Z_TILES + 0.3),
                RoomEntity, PlayingEntity,
            ));
        }
    }

    // ── Crates / shelves flanking the merchant ──
    let crate_color = Color::srgb(0.38, 0.26, 0.14);
    let crate_dark = Color::srgb(0.28, 0.18, 0.08);
    let crate_positions: [(f32, f32); 4] = [
        (TILE_SIZE * 4.0, 3.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 5.5, 3.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 18.5, 3.0 * TILE_SIZE + TILE_SIZE),
        (TILE_SIZE * 19.5, 3.0 * TILE_SIZE + TILE_SIZE),
    ];
    for (cx, cy) in crate_positions {
        commands.spawn((
            Sprite { color: crate_color, custom_size: Some(Vec2::new(16.0, 14.0)), ..default() },
            Transform::from_xyz(cx, cy + 7.0, Z_PICKUPS - 0.2),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: crate_dark, custom_size: Some(Vec2::new(14.0, 2.0)), ..default() },
            Transform::from_xyz(cx, cy + 7.0, Z_PICKUPS - 0.15),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: crate_dark, custom_size: Some(Vec2::new(2.0, 12.0)), ..default() },
            Transform::from_xyz(cx, cy + 7.0, Z_PICKUPS - 0.15),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Shelves behind merchant (on walls) ──
    let shelf_color = Color::srgb(0.35, 0.24, 0.12);
    for sx in [TILE_SIZE * 7.0, TILE_SIZE * 16.5] {
        commands.spawn((
            Sprite { color: shelf_color, custom_size: Some(Vec2::new(TILE_SIZE * 1.5, 4.0)), ..default() },
            Transform::from_xyz(sx, TILE_SIZE * 5.5, Z_TILES + 0.1),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgb(0.3, 0.7, 0.4), custom_size: Some(Vec2::new(5.0, 8.0)), ..default() },
            Transform::from_xyz(sx - 6.0, TILE_SIZE * 5.5 + 6.0, Z_TILES + 0.12),
            RoomEntity, PlayingEntity,
        ));
        commands.spawn((
            Sprite { color: Color::srgb(0.85, 0.75, 0.5), custom_size: Some(Vec2::new(8.0, 5.0)), ..default() },
            Transform::from_xyz(sx + 8.0, TILE_SIZE * 5.5 + 5.0, Z_TILES + 0.12),
            RoomEntity, PlayingEntity,
        ));
    }

    // ── Warm ambient tint overlay (subtle) ──
    commands.spawn((
        Sprite {
            color: Color::srgba(0.6, 0.3, 0.1, 0.05),
            custom_size: Some(Vec2::new(ROOM_W, ROOM_H)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0, ROOM_H / 2.0, Z_TILES + 0.01),
        RoomEntity, PlayingEntity,
    ));
}

fn spawn_treasure_room(commands: &mut Commands, seed: u64, tiles: &TilesetAssets) {
    let plat_color = Color::srgb(0.34, 0.30, 0.22);
    let gold_accent = Color::srgb(0.85, 0.70, 0.20);
    let gold_dark = Color::srgb(0.65, 0.50, 0.12);

    // Raised central altar – lower than before (row 2 instead of 3)
    spawn_platform(commands, 7, 16, 2, plat_color, tiles);
    // Side wings at row 4
    spawn_platform(commands, 3, 6,  4, plat_color, tiles);
    spawn_platform(commands, 17, 20, 4, plat_color, tiles);

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

fn spawn_boss_room(commands: &mut Commands, floor: i32, tiles: &TilesetAssets) {
    let plat_color = Color::srgb(0.32, 0.16, 0.08);
    let pillar_color = Color::srgb(0.28, 0.14, 0.07);
    let arena_top = 4.0 * TILE_SIZE + TILE_SIZE;
    let center_x = ROOM_W / 2.0;

    // Wide ground arena
    spawn_platform(commands, 3, 20, 4, plat_color, tiles);
    // Two raised side platforms
    spawn_platform_worn(commands, 4,  7,  6, plat_color, tiles);
    spawn_platform_worn(commands, 16, 19, 6, plat_color, tiles);
    // Small stepping stones to side platforms
    spawn_platform(commands, 3, 4, 5, plat_color, tiles);
    spawn_platform(commands, 19, 20, 5, plat_color, tiles);

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
    // Inner shorter pillars (decorative, at the edges of the arena)
    spawn_pillar(commands, 6,  5, 7, pillar_color);
    spawn_pillar(commands, 17, 5, 7, pillar_color);

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

// ─── Safe Spawn Points ───────────────────────────────────────────────────────

/// Returns a safe (x, y) spawn point for the player based on room type.
/// Boss rooms spawn on top of the arena platform (row 4 → y = 5*TILE_SIZE).
/// Other rooms use the default left-side spawn above the floor.
fn safe_spawn_point(room_type: RoomType) -> (f32, f32) {
    match room_type {
        // Boss arena platform is at row 4 → top is y = 5*TILE_SIZE
        // Spawn at col 5 (well away from lava at cols 2-3)
        RoomType::Boss => (TILE_SIZE * 5.0, TILE_SIZE * 5.0 + 16.0),
        // Default: left side, above floor
        _ => (TILE_SIZE * 2.5, TILE_SIZE * 2.0),
    }
}

// ─── Animation Systems ────────────────────────────────────────────────────────

fn check_altar_proximity(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    player_q: Query<&Transform, With<Player>>,
    mut altar_q: Query<(&Transform, &mut AltarEntity), Without<Player>>,
    mut state: ResMut<AltarState>,
    room_state: Res<RoomState>,
) {
    if state.active { return; }
    let Ok(p_tf) = player_q.get_single() else { return; };
    check_altar_interaction(&keys, &gamepads, p_tf, &mut altar_q, &mut state, &room_state);
}

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
    mut relic_state: ResMut<RelicChoiceState>,
    relic_inventory: Res<RelicInventory>,
    tiles: Res<TilesetAssets>,
    well_assets: Res<WellAssets>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
) {
    // Block room transitions while overlays are active
    if floor_complete.active { return; }
    if relic_state.active { return; }

    let Ok((mut player, mut player_tf)) = player_q.get_single_mut() else { return };

    for (door_tf, door) in &door_q {
        if door.locked { continue; }

        let dist = (player_tf.translation.xy() - door_tf.translation.xy()).abs();
        if dist.x < 30.0 && dist.y < 60.0 {
            // Check if this is the last room of the floor (boss cleared)
            let is_floor_end = room_state.room_index + 1 >= room_state.floor_layout.len();

            if is_floor_end {
                // Show relic choice first, then floor complete
                if !relic_state.active && relic_state.choices[0].is_none() {
                    let seed = room_state.seed.wrapping_add(room_state.floor as u64 * 31);
                    start_relic_choice(&mut relic_state, &relic_inventory, seed);
                    break;
                }
                // After relic is chosen (state not active and choices were set), show floor complete
                if !relic_state.active {
                    floor_complete.active = true;
                    floor_complete.floor_completed = room_state.floor;
                    floor_complete.ui_spawned = false;
                    ev_sfx.send(PlaySfxEvent(SfxType::LevelComplete));
                    // Reset relic choice state for next floor
                    relic_state.choices = [None; 3];
                }
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
            room_state.room_cleared = !matches!(room_state.current_type, RoomType::Combat | RoomType::Boss);

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

            spawn_room(&mut commands, &room_state, room_state.room_index, &tiles, &well_assets);

            // Teleport player to safe spawn based on room type
            let (spawn_x, spawn_y) = safe_spawn_point(room_state.current_type);
            player_tf.translation.x = spawn_x;
            player_tf.translation.y = spawn_y;
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
    tiles: Res<TilesetAssets>,
    well_assets: Res<WellAssets>,
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

        spawn_room(&mut commands, &room_state, 0, &tiles, &well_assets);

        // Teleport player to safe spawn
        if let Ok((mut player, mut tf)) = player_q.get_single_mut() {
            let (sx, sy) = safe_spawn_point(room_state.current_type);
            tf.translation.x = sx;
            tf.translation.y = sy;
            player.vx = 0.0;
            player.vy = 0.0;
        }

        break;
    }
}
