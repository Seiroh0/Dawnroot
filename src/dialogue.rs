use bevy::prelude::*;
use crate::{constants::*, GameState, GameFont, PlayingEntity, room::{RoomState, RoomType, RoomTransition}};

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DialogueState::default())
            .add_event::<StartDialogue>()
            .add_systems(
                Update,
                (
                    spawn_npcs_in_rooms,
                    npc_interaction,
                    handle_start_dialogue,
                    update_dialogue_ui,
                    advance_dialogue,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Components & Resources
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct Npc {
    pub name: String,
    pub lines: Vec<String>,
    pub interacted: bool,
}

#[derive(Component)]
struct NpcVisual;

#[derive(Component)]
struct NpcPrompt;

#[derive(Component)]
struct DialogueUI;

#[derive(Event)]
struct StartDialogue {
    name: String,
    lines: Vec<String>,
}

#[derive(Resource, Default)]
struct DialogueState {
    active: bool,
    speaker: String,
    lines: Vec<String>,
    current_line: usize,
    char_index: usize,
    char_timer: f32,
    ui_spawned: bool,
    cooldown: f32,
}

// ---------------------------------------------------------------------------
// NPC definitions per room type
// ---------------------------------------------------------------------------

fn start_room_npc(floor: i32) -> (&'static str, Vec<String>) {
    match floor {
        1 => ("Old Keeper", vec![
            "So you've descended into the roots...".into(),
            "Many have tried. The well swallows them all.".into(),
            "Press onward. Defeat the guardians of each floor.".into(),
            "And whatever you do... don't fall into the lava.".into(),
        ]),
        2 => ("Wandering Spirit", vec![
            "You survived the first floor? Impressive.".into(),
            "The creatures grow fiercer the deeper you go.".into(),
            "I've been trapped here for... how long now?".into(),
            "Find the exit. Don't end up like me.".into(),
        ]),
        3 => ("Root Hermit", vec![
            "The roots grow thick here. They have a will of their own.".into(),
            "I've learned to listen to them. They whisper secrets.".into(),
            "The boss of this floor guards something ancient.".into(),
            "Be ready. It won't be like the others.".into(),
        ]),
        _ => ("Echo of the Deep", vec![
            "Few have ever reached this depth.".into(),
            "The Dawnroot itself stirs below.".into(),
            "You are close to the truth now.".into(),
            "Steel yourself for what lies ahead.".into(),
        ]),
    }
}

fn boss_taunt(floor: i32) -> (&'static str, Vec<String>) {
    match floor {
        1 => ("Guardian of the Shallows", vec![
            "You dare trespass in MY domain?".into(),
            "The roots will reclaim your bones!".into(),
        ]),
        2 => ("Warden of Depths", vec![
            "Another fool seeking the Dawnroot...".into(),
            "Your journey ends HERE.".into(),
        ]),
        3 => ("Ancient Sentinel", vec![
            "I have stood guard for a thousand years.".into(),
            "You will not pass.".into(),
        ]),
        _ => ("The Rootbound", vec![
            "So... you've come at last.".into(),
            "Let us see if you are worthy.".into(),
        ]),
    }
}

// ---------------------------------------------------------------------------
// Spawn NPCs when entering certain rooms
// ---------------------------------------------------------------------------

fn spawn_npcs_in_rooms(
    mut commands: Commands,
    mut ev_transition: EventReader<RoomTransition>,
    room_state: Res<RoomState>,
    mut dialogue_state: ResMut<DialogueState>,
) {
    for _ in ev_transition.read() {
        // Close any open dialogue on room transition
        dialogue_state.active = false;
        dialogue_state.ui_spawned = false;

        match room_state.current_type {
            RoomType::Start => {
                let (name, lines) = start_room_npc(room_state.floor);
                spawn_npc(&mut commands, name, lines, 200.0, 120.0);
            }
            RoomType::Shop => {
                // Merchant NPC is now spawned by shop.rs (stone merchant)
            }
            RoomType::Boss => {
                let (name, lines) = boss_taunt(room_state.floor);
                spawn_npc(&mut commands, name, lines, ROOM_W / 2.0, 160.0);
            }
            _ => {}
        }
    }
}

fn spawn_npc(commands: &mut Commands, name: &str, lines: Vec<String>, x: f32, y: f32) {
    commands.spawn((
        // NPC root (collision box)
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(24.0, 36.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_PLAYER - 0.5),
        Npc {
            name: name.to_string(),
            lines,
            interacted: false,
        },
        PlayingEntity,
    )).with_children(|p| {
        // Body (robed figure)
        p.spawn((
            Sprite {
                color: Color::srgb(0.35, 0.28, 0.18),
                custom_size: Some(Vec2::new(16.0, 20.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            NpcVisual,
        ));
        // Head
        p.spawn((
            Sprite {
                color: Color::srgb(0.72, 0.58, 0.44),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 14.0, 0.2),
            NpcVisual,
        ));
        // Hood/hat
        p.spawn((
            Sprite {
                color: Color::srgb(0.30, 0.22, 0.14),
                custom_size: Some(Vec2::new(16.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 19.0, 0.25),
            NpcVisual,
        ));
        // Eyes (two dots)
        p.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.10, 0.05),
                custom_size: Some(Vec2::new(2.0, 2.5)),
                ..default()
            },
            Transform::from_xyz(-2.5, 14.5, 0.3),
            NpcVisual,
        ));
        p.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.10, 0.05),
                custom_size: Some(Vec2::new(2.0, 2.5)),
                ..default()
            },
            Transform::from_xyz(2.5, 14.5, 0.3),
            NpcVisual,
        ));
        // Interaction prompt (hidden until player is near)
        p.spawn((
            Text2d::new("[E] Talk"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.9, 0.75, 0.4, 0.0)),
            Transform::from_xyz(0.0, 32.0, 1.0),
            NpcPrompt,
        ));
    });
}

// ---------------------------------------------------------------------------
// Proximity check + interaction trigger
// ---------------------------------------------------------------------------

fn npc_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    player_q: Query<&Transform, (With<crate::player::Player>, Without<Npc>)>,
    mut npc_q: Query<(&Transform, &mut Npc), Without<crate::player::Player>>,
    mut prompt_q: Query<(&Parent, &mut TextColor), With<NpcPrompt>>,
    mut ev_start: EventWriter<StartDialogue>,
    dialogue_state: Res<DialogueState>,
) {
    let Ok(p_tf) = player_q.get_single() else { return };
    if dialogue_state.active { return; }

    let interact = keys.just_pressed(KeyCode::KeyE)
        || gamepads.iter().next().map_or(false, |g| g.just_pressed(GamepadButton::West));

    // Show/hide prompt based on proximity; trigger dialogue on interact
    for (npc_tf, mut npc) in &mut npc_q {
        let dist = (p_tf.translation.xy() - npc_tf.translation.xy()).length();
        let near = dist < 60.0;

        // Show prompt when near (find the prompt child)
        if near && interact && !npc.interacted {
            npc.interacted = true;
            ev_start.send(StartDialogue {
                name: npc.name.clone(),
                lines: npc.lines.clone(),
            });
        }
    }

    // Update prompt visibility based on proximity
    for (parent, mut color) in &mut prompt_q {
        if let Ok((ntf, _)) = npc_q.get(parent.get()) {
            let dist = (p_tf.translation.xy() - ntf.translation.xy()).length();
            color.0 = if dist < 60.0 {
                Color::srgba(0.9, 0.75, 0.4, 1.0)
            } else {
                Color::srgba(0.9, 0.75, 0.4, 0.0)
            };
        }
    }
}

// ---------------------------------------------------------------------------
// Dialogue state management
// ---------------------------------------------------------------------------

fn handle_start_dialogue(
    mut ev: EventReader<StartDialogue>,
    mut state: ResMut<DialogueState>,
) {
    for event in ev.read() {
        state.active = true;
        state.speaker = event.name.clone();
        state.lines = event.lines.clone();
        state.current_line = 0;
        state.char_index = 0;
        state.char_timer = 0.0;
        state.ui_spawned = false;
        state.cooldown = 0.2; // prevent instant skip
    }
}

fn advance_dialogue(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut state: ResMut<DialogueState>,
    mut commands: Commands,
    ui_q: Query<Entity, With<DialogueUI>>,
    time: Res<Time>,
) {
    if !state.active { return; }
    state.cooldown = (state.cooldown - time.delta_secs()).max(0.0);
    if state.cooldown > 0.0 { return; }

    let gp_advance = gamepads.iter().next().map_or(false, |g| {
        g.just_pressed(GamepadButton::South) || g.just_pressed(GamepadButton::West)
    });
    let advance = keys.just_pressed(KeyCode::Space)
        || keys.just_pressed(KeyCode::KeyE)
        || keys.just_pressed(KeyCode::Enter)
        || gp_advance;

    if !advance { return; }

    let current_text = &state.lines[state.current_line];
    if state.char_index < current_text.len() {
        // Skip typewriter — show full line immediately
        state.char_index = current_text.len();
    } else {
        // Next line or close
        state.current_line += 1;
        if state.current_line >= state.lines.len() {
            // Close dialogue
            state.active = false;
            state.ui_spawned = false;
            for e in &ui_q {
                commands.entity(e).try_despawn_recursive();
            }
        } else {
            state.char_index = 0;
            state.char_timer = 0.0;
            state.cooldown = 0.1;
        }
    }
}

// ---------------------------------------------------------------------------
// Dialogue UI rendering
// ---------------------------------------------------------------------------

#[derive(Component)]
struct DialogueSpeakerText;

#[derive(Component)]
struct DialogueBodyText;

fn update_dialogue_ui(
    mut commands: Commands,
    mut state: ResMut<DialogueState>,
    mut speaker_q: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueBodyText>)>,
    mut body_q: Query<&mut Text, (With<DialogueBodyText>, Without<DialogueSpeakerText>)>,
    ui_q: Query<Entity, With<DialogueUI>>,
    time: Res<Time>,
    font: Res<GameFont>,
) {
    if !state.active {
        // Despawn UI if dialogue closed externally
        if state.ui_spawned {
            state.ui_spawned = false;
            for e in &ui_q {
                commands.entity(e).try_despawn_recursive();
            }
        }
        return;
    }

    // Spawn UI if needed
    if !state.ui_spawned {
        state.ui_spawned = true;
        spawn_dialogue_box(&mut commands, &state.speaker, &font.0);
    }

    // Typewriter effect
    let dt = time.delta_secs();
    state.char_timer += dt;
    let chars_per_sec = 35.0;
    let target_chars = (state.char_timer * chars_per_sec) as usize;
    let line_len = state.lines[state.current_line].len();
    state.char_index = state.char_index.max(target_chars).min(line_len);

    // Update displayed text
    let visible: String = state.lines[state.current_line].chars().take(state.char_index).collect();
    let speaker = state.speaker.clone();
    if let Ok(mut text) = body_q.get_single_mut() {
        **text = visible;
    }
    if let Ok(mut text) = speaker_q.get_single_mut() {
        **text = speaker;
    }
}

fn spawn_dialogue_box(commands: &mut Commands, speaker: &str, font: &Handle<Font>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Px(40.0),
                right: Val::Px(40.0),
                min_height: Val::Px(90.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.04, 0.02, 0.92)),
            BorderColor(Color::srgb(0.45, 0.30, 0.15)),
            BorderRadius::all(Val::Px(4.0)),
            DialogueUI,
            PlayingEntity,
        ))
        .with_children(|parent| {
            let f = font.clone();
            // Speaker name
            parent.spawn((
                Text::new(speaker.to_string()),
                TextFont { font: f.clone(), font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.95, 0.7, 0.25)),
                DialogueSpeakerText,
            ));

            // Body text (starts empty, filled by typewriter)
            parent.spawn((
                Text::new("".to_string()),
                TextFont { font: f.clone(), font_size: 8.0, ..default() },
                TextColor(Color::srgb(0.85, 0.78, 0.65)),
                DialogueBodyText,
            ));

            // Advance hint
            parent.spawn((
                Text::new("[E/Space/A] continue"),
                TextFont { font: f.clone(), font_size: 6.0, ..default() },
                TextColor(Color::srgb(0.5, 0.4, 0.3)),
            ));
        });
}
