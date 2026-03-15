use bevy::prelude::*;
use crate::{
    GameState, GameFont, RunData,
    player::Player, spell::SpellSlots,
    ActiveSaveSlot, SaveSlotData, MetaProgression,
};

pub struct PauseMenuPlugin;

impl Plugin for PauseMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_pause.run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
        .add_systems(
            Update,
            pause_menu_input.run_if(in_state(GameState::Paused)),
        );
    }
}

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct PauseMenuItem(usize);

#[derive(Resource)]
struct PauseMenuState {
    selected: usize,
    input_cooldown: f32,
}

const MENU_ITEMS: [&str; 3] = ["Resume", "Settings", "Save & Quit"];

/// Sub-panel shown when "Settings" is selected.
#[derive(Component)]
struct SettingsPanel;

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut next_state: ResMut<NextState<GameState>>,
    shop_state: Option<Res<crate::shop::ShopUiState>>,
    altar_state: Option<Res<crate::altar::AltarState>>,
    floor_state: Option<Res<crate::floor_complete::FloorCompleteState>>,
    relic_state: Option<Res<crate::relic::RelicChoiceState>>,
) {
    // Don't pause if any overlay is active
    if shop_state.map_or(false, |s| s.active) { return; }
    if altar_state.map_or(false, |s| s.active) { return; }
    if floor_state.map_or(false, |s| s.active) { return; }
    if relic_state.map_or(false, |s| s.active) { return; }

    let gp = gamepads.iter().next();
    let esc = keys.just_pressed(KeyCode::Escape) || gp.map_or(false, |g| g.just_pressed(GamepadButton::Start));
    if esc {
        next_state.set(GameState::Paused);
    }
}

fn spawn_pause_menu(mut commands: Commands, font: Res<GameFont>) {
    commands.insert_resource(PauseMenuState { selected: 0, input_cooldown: 0.0 });

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(200),
            PauseMenuRoot,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(32.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.06, 0.95)),
                ))
                .with_children(|panel| {
                    // Title
                    panel.spawn((
                        Text::new("PAUSED"),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.8, 0.5)),
                        Node {
                            margin: UiRect::bottom(Val::Px(16.0)),
                            ..default()
                        },
                    ));

                    // Menu items
                    for (i, label) in MENU_ITEMS.iter().enumerate() {
                        let prefix = if i == 0 { "> " } else { "  " };
                        let color = if i == 0 {
                            Color::srgb(1.0, 0.9, 0.4)
                        } else {
                            Color::srgb(0.6, 0.55, 0.45)
                        };
                        panel.spawn((
                            Text::new(format!("{}{}", prefix, label)),
                            TextFont {
                                font: font.0.clone(),
                                font_size: 10.0,
                                ..default()
                            },
                            TextColor(color),
                            Node {
                                margin: UiRect::vertical(Val::Px(4.0)),
                                ..default()
                            },
                            PauseMenuItem(i),
                        ));
                    }

                    // Controls hint
                    panel.spawn((
                        Text::new("[Up/Down] Select  [Enter/E] Confirm  [Esc] Resume"),
                        TextFont {
                            font: font.0.clone(),
                            font_size: 6.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.35, 0.28)),
                        Node {
                            margin: UiRect::top(Val::Px(12.0)),
                            ..default()
                        },
                    ));
                });
        });
}

fn despawn_pause_menu(
    mut commands: Commands,
    q: Query<Entity, With<PauseMenuRoot>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<PauseMenuState>();
}

fn pause_menu_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut state: ResMut<PauseMenuState>,
    mut item_q: Query<(&PauseMenuItem, &mut Text, &mut TextColor)>,
    settings_q: Query<Entity, With<SettingsPanel>>,
    font: Res<GameFont>,
    // For saving
    player_q: Query<&Player>,
    slots_q: Query<&SpellSlots>,
    run: Res<RunData>,
    slot: Res<ActiveSaveSlot>,
    meta: Res<MetaProgression>,
) {
    let dt = time.delta_secs();
    state.input_cooldown = (state.input_cooldown - dt).max(0.0);
    if state.input_cooldown > 0.0 {
        return;
    }

    let gp = gamepads.iter().next();

    // Navigate
    let up = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadUp));
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadDown));

    if up && state.selected > 0 {
        state.selected -= 1;
        state.input_cooldown = 0.15;
    }
    if down && state.selected < MENU_ITEMS.len() - 1 {
        state.selected += 1;
        state.input_cooldown = 0.15;
    }

    // Update visuals with ">" cursor
    for (item, mut text, mut color) in &mut item_q {
        let prefix = if item.0 == state.selected { "> " } else { "  " };
        **text = format!("{}{}", prefix, MENU_ITEMS[item.0]);
        *color = if item.0 == state.selected {
            TextColor(Color::srgb(1.0, 0.9, 0.4))
        } else {
            TextColor(Color::srgb(0.6, 0.55, 0.45))
        };
    }

    // Resume on ESC
    let esc = keys.just_pressed(KeyCode::Escape) || gp.map_or(false, |g| g.just_pressed(GamepadButton::Start));
    if esc {
        // If settings sub-panel is open, close it and return to main pause menu
        let has_settings = settings_q.iter().next().is_some();
        if has_settings {
            for e in &settings_q {
                commands.entity(e).despawn_recursive();
            }
            return;
        }
        next_state.set(GameState::Playing);
        return;
    }

    // Confirm (Enter, E, Space, or gamepad South)
    let confirm = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyE)
        || keys.just_pressed(KeyCode::Space)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));
    if !confirm {
        return;
    }

    match state.selected {
        0 => {
            // Resume
            next_state.set(GameState::Playing);
        }
        1 => {
            // Settings (placeholder)
            let has_settings = settings_q.iter().next().is_some();
            if has_settings {
                // Toggle off
                for e in &settings_q {
                    commands.entity(e).despawn_recursive();
                }
            } else {
                // Spawn placeholder panel
                commands.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(30.0),
                        right: Val::Percent(30.0),
                        top: Val::Percent(35.0),
                        bottom: Val::Percent(35.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.12, 0.10, 0.08, 0.95)),
                    GlobalZIndex(210),
                    SettingsPanel,
                )).with_children(|panel| {
                    panel.spawn((
                        Text::new("Coming Soon"),
                        TextFont { font: font.0.clone(), font_size: 12.0, ..default() },
                        TextColor(Color::srgb(0.6, 0.55, 0.45)),
                    ));
                });
            }
        }
        2 => {
            // Save & Quit → save and return to title
            if let Ok(player) = player_q.get_single() {
                let spells = if let Ok(s) = slots_q.get_single() {
                    [s.slots[0].is_some(), s.slots[1].is_some(), s.slots[2].is_some(), s.slots[3].is_some()]
                } else {
                    [false; 4]
                };
                let save_data = SaveSlotData {
                    floor: run.current_floor,
                    gold: run.gold,
                    score: run.score,
                    time_played: run.time,
                    max_health: player.max_health,
                    max_mana: player.max_mana,
                    spells,
                    enemies_killed: run.enemies_killed,
                };
                crate::save_slot(slot.0, &save_data);
                crate::save_meta(&meta);
            }
            // OnEnter(Title) cleanup_gameplay handles entity despawn
            next_state.set(GameState::Title);
        }
        _ => {}
    }
}
