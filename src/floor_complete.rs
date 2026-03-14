use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, RunData, PlayingEntity, ActiveSaveSlot, SaveSlotData,
    room::{AdvanceFloor, RoomState},
};

pub struct FloorCompletePlugin;

impl Plugin for FloorCompletePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FloorCompleteState::default())
            .add_systems(
                Update,
                (spawn_floor_complete_ui, floor_complete_input)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct FloorCompleteState {
    pub active: bool,
    pub floor_completed: i32,
    pub ui_spawned: bool,
}

#[derive(Component)]
struct FloorCompleteUI;

fn spawn_floor_complete_ui(
    mut commands: Commands,
    mut state: ResMut<FloorCompleteState>,
    run: Res<RunData>,
) {
    if !state.active || state.ui_spawned {
        return;
    }
    state.ui_spawned = true;

    let minutes = (run.time / 60.0) as i32;
    let seconds = (run.time % 60.0) as i32;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.04, 0.02, 0.92)),
            FloorCompleteUI,
            PlayingEntity,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(format!("FLOOR {} CONQUERED!", state.floor_completed)),
                TextFont { font_size: 38.0, ..default() },
                TextColor(Color::srgb(0.95, 0.7, 0.2)),
            ));

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            // Stats
            parent.spawn((
                Text::new(format!("Enemies Defeated: {}", run.enemies_killed)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.75, 0.6, 0.4)),
            ));

            parent.spawn((
                Text::new(format!("Gold: {}", run.gold)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            parent.spawn((
                Text::new(format!("Score: {:06}", run.score)),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Time: {}:{:02}", minutes, seconds)),
                TextFont { font_size: 18.0, ..default() },
                TextColor(Color::srgb(0.75, 0.6, 0.4)),
            ));

            parent.spawn(Node { height: Val::Px(28.0), ..default() });

            // Options
            parent.spawn((
                Text::new("[SPACE]  Descend Deeper"),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.9, 0.65, 0.2)),
            ));

            parent.spawn((
                Text::new("[ESC]  Save & Return to Surface"),
                TextFont { font_size: 17.0, ..default() },
                TextColor(Color::srgb(0.6, 0.5, 0.35)),
            ));
        });

    // Spawn celebration particles (gold confetti)
    for i in 0..40_u32 {
        let x_offset = (i as f32 / 40.0 - 0.5) * ROOM_W * 0.85;
        let y_jitter = (i as f32 * 31.7 % 80.0) - 40.0;
        let color = match i % 5 {
            0 => Color::srgb(1.0, 0.85, 0.15),
            1 => Color::srgb(0.85, 0.45, 0.1),
            2 => Color::srgb(0.95, 0.95, 0.9),
            3 => Color::srgb(0.9, 0.55, 0.15),
            _ => Color::srgb(1.0, 0.55, 0.1),
        };
        let (w, h) = if i % 2 == 0 { (7.0, 3.0) } else { (3.0, 8.0) };
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(
                ROOM_W / 2.0 + x_offset,
                ROOM_H * 0.8 + y_jitter,
                Z_EFFECTS + 2.0,
            ),
            VictoryConfetti {
                vx: (i as f32 * 17.3 % 100.0) - 50.0,
                vy: -(20.0 + (i as f32 * 13.7 % 50.0)),
                lifetime: 2.0 + (i as f32 * 0.04),
            },
            PlayingEntity,
        ));
    }
}

#[derive(Component)]
struct VictoryConfetti {
    vx: f32,
    vy: f32,
    lifetime: f32,
}

fn floor_complete_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut state: ResMut<FloorCompleteState>,
    mut commands: Commands,
    ui_q: Query<Entity, With<FloorCompleteUI>>,
    mut ev_advance: EventWriter<AdvanceFloor>,
    mut next_state: ResMut<NextState<GameState>>,
    run: Res<RunData>,
    room_state: Res<RoomState>,
    slot: Res<ActiveSaveSlot>,
    player_q: Query<&crate::player::Player>,
    slots_q: Query<&crate::spell::SpellSlots>,
    // Also update confetti
    mut confetti_q: Query<(Entity, &mut Transform, &mut Sprite, &mut VictoryConfetti)>,
    time: Res<Time>,
) {
    // Animate confetti regardless of input
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut c) in &mut confetti_q {
        c.lifetime -= dt;
        if c.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        c.vy -= 50.0 * dt;
        c.vx += ((tf.translation.y * 0.04).sin() * 12.0) * dt;
        c.vx *= 0.98_f32.powf(dt * 60.0);
        tf.translation.x += c.vx * dt;
        tf.translation.y += c.vy * dt;
        let alpha = if c.lifetime < 0.5 { c.lifetime * 2.0 } else { 1.0 };
        let col = sprite.color.to_srgba();
        sprite.color = Color::srgba(col.red, col.green, col.blue, alpha);
    }

    if !state.active {
        return;
    }

    let gp = gamepads.iter().next();
    let gp_descend = gp.map_or(false, |g| g.just_pressed(GamepadButton::South) || g.just_pressed(GamepadButton::Start));
    if keys.just_pressed(KeyCode::Space) || gp_descend {
        // Descend deeper
        state.active = false;
        state.ui_spawned = false;

        for e in &ui_q {
            commands.entity(e).despawn_recursive();
        }

        ev_advance.send(AdvanceFloor);
    }

    let gp_quit = gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if keys.just_pressed(KeyCode::Escape) || gp_quit {
        // Save and quit
        state.active = false;
        state.ui_spawned = false;

        // Build save data
        let player = player_q.get_single().ok();
        let spells = slots_q.get_single().ok();
        let save = SaveSlotData {
            floor: room_state.floor + 1, // Save the NEXT floor (we completed current)
            gold: run.gold,
            score: run.score,
            time_played: run.time,
            max_health: player.map(|p| p.max_health).unwrap_or(5),
            max_mana: player.map(|p| p.max_mana).unwrap_or(100.0),
            spells: if let Some(s) = spells {
                [
                    s.slots[0].is_some(),
                    s.slots[1].is_some(),
                    s.slots[2].is_some(),
                    s.slots[3].is_some(),
                ]
            } else {
                [false; 4]
            },
            enemies_killed: run.enemies_killed,
        };
        crate::save_slot(slot.0, &save);

        next_state.set(GameState::Title);
    }
}
