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
                (check_room_exit, check_room_cleared)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Event)]
pub struct RoomCleared;

#[derive(Event)]
pub struct RoomTransition;

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct ExitDoor {
    pub locked: bool,
}

#[derive(Component)]
pub struct RoomEntity;

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

fn spawn_room(commands: &mut Commands, state: &RoomState, room_idx: usize) {
    let room_type = state.floor_layout.get(room_idx).copied().unwrap_or(RoomType::Combat);
    let seed = state.seed.wrapping_add(room_idx as u64).wrapping_mul(state.floor as u64 + 1);

    // Background
    commands.spawn((
        Sprite {
            color: Color::srgb(0.08, 0.06, 0.12),
            custom_size: Some(Vec2::new(ROOM_W, ROOM_H)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0, ROOM_H / 2.0, Z_BACKGROUND),
        RoomEntity,
        PlayingEntity,
    ));

    // Floor
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, TILE_SIZE / 2.0, Color::srgb(0.22, 0.18, 0.28));
    }

    // Ceiling
    for col in 0..ROOM_COLUMNS {
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, ROOM_H - TILE_SIZE / 2.0, Color::srgb(0.18, 0.14, 0.22));
    }

    // Left wall
    for row in 0..ROOM_ROWS {
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, TILE_SIZE / 2.0, y, Color::srgb(0.2, 0.16, 0.25));
    }

    // Right wall (with gap for exit door)
    for row in 0..ROOM_ROWS {
        let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        // Leave door gap at rows 1-3 (above floor)
        if row >= 1 && row <= 3 {
            continue;
        }
        spawn_tile(commands, ROOM_W - TILE_SIZE / 2.0, y, Color::srgb(0.2, 0.16, 0.25));
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

    // Room-specific content
    match room_type {
        RoomType::Start => spawn_start_platforms(commands),
        RoomType::Combat => spawn_combat_room(commands, seed, state.floor),
        RoomType::Treasure => spawn_treasure_room(commands, seed),
        RoomType::Boss => spawn_boss_room(commands, state.floor),
        RoomType::Shop => {} // Shop handled by shop.rs
    }
}

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

fn spawn_platform(commands: &mut Commands, col_start: i32, col_end: i32, row: i32, color: Color) {
    let y = row as f32 * TILE_SIZE + TILE_SIZE / 2.0;
    for col in col_start..=col_end {
        if col <= 0 || col >= ROOM_COLUMNS - 1 { continue; }
        let x = col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        spawn_tile(commands, x, y, color);
    }
}

fn spawn_start_platforms(commands: &mut Commands) {
    let plat_color = Color::srgb(0.25, 0.22, 0.3);
    // Simple welcoming layout
    spawn_platform(commands, 3, 8, 4, plat_color);
    spawn_platform(commands, 10, 15, 6, plat_color);
    spawn_platform(commands, 16, 21, 4, plat_color);
}

fn spawn_combat_room(commands: &mut Commands, seed: u64, floor: i32) {
    let plat_color = Color::srgb(0.25, 0.22, 0.3);
    let template = (seed % 5) as i32;

    match template {
        0 => {
            // Staircase up
            spawn_platform(commands, 3, 6, 3, plat_color);
            spawn_platform(commands, 8, 11, 5, plat_color);
            spawn_platform(commands, 13, 16, 7, plat_color);
            spawn_platform(commands, 18, 21, 5, plat_color);
        }
        1 => {
            // Multi-level arena
            spawn_platform(commands, 2, 7, 4, plat_color);
            spawn_platform(commands, 9, 14, 7, plat_color);
            spawn_platform(commands, 16, 21, 4, plat_color);
            spawn_platform(commands, 5, 10, 10, plat_color);
        }
        2 => {
            // Pits with bridges
            spawn_platform(commands, 4, 6, 4, plat_color);
            spawn_platform(commands, 10, 12, 4, plat_color);
            spawn_platform(commands, 16, 18, 4, plat_color);
            spawn_platform(commands, 7, 9, 7, plat_color);
            spawn_platform(commands, 13, 15, 7, plat_color);
        }
        3 => {
            // Tower platforms
            spawn_platform(commands, 5, 7, 3, plat_color);
            spawn_platform(commands, 5, 7, 6, plat_color);
            spawn_platform(commands, 5, 7, 9, plat_color);
            spawn_platform(commands, 11, 13, 4, plat_color);
            spawn_platform(commands, 11, 13, 7, plat_color);
            spawn_platform(commands, 17, 19, 3, plat_color);
            spawn_platform(commands, 17, 19, 6, plat_color);
        }
        _ => {
            // Open arena with raised center
            spawn_platform(commands, 3, 20, 4, plat_color);
            spawn_platform(commands, 8, 15, 7, plat_color);
            spawn_platform(commands, 10, 13, 10, plat_color);
        }
    }

    // Scale difficulty with floor
    let _difficulty = floor;
}

fn spawn_treasure_room(commands: &mut Commands, _seed: u64) {
    let plat_color = Color::srgb(0.3, 0.28, 0.2);
    spawn_platform(commands, 8, 15, 3, plat_color);
    // Treasure chest visual placeholder
    commands.spawn((
        Sprite {
            color: Color::srgb(0.8, 0.65, 0.2),
            custom_size: Some(Vec2::new(24.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(ROOM_W / 2.0, 3.0 * TILE_SIZE + TILE_SIZE / 2.0 + 30.0, Z_PICKUPS),
        RoomEntity,
        PlayingEntity,
    ));
}

fn spawn_boss_room(commands: &mut Commands, _floor: i32) {
    let plat_color = Color::srgb(0.3, 0.15, 0.2);
    // Wide arena
    spawn_platform(commands, 3, 20, 4, plat_color);
    spawn_platform(commands, 6, 9, 7, plat_color);
    spawn_platform(commands, 14, 17, 7, plat_color);
}

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
    _state: ResMut<RoomState>,
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
                room_state.seed = room_state.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
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
