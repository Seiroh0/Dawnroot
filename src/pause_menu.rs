use bevy::prelude::*;
use crate::{
    GameState, GameFont, RunData, ResumingFromPause,
    player::Player, spell::SpellSlots,
    ActiveSaveSlot, SaveSlotData, MetaProgression,
    audio::AudioSettings,
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
            (pause_menu_input, settings_input)
                .run_if(in_state(GameState::Paused)),
        );
    }
}

// ── Pause menu ────────────────────────────────────────────────────

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

// ── Settings panel ────────────────────────────────────────────────

/// Root entity of the settings overlay.
#[derive(Component)]
pub struct SettingsPanel;

/// Which top-level tab is visible.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SettingsTab {
    #[default]
    Audio,
    Graphics,
    Controls,
}

impl SettingsTab {
    fn label(self) -> &'static str {
        match self {
            Self::Audio    => "Audio",
            Self::Graphics => "Graphics",
            Self::Controls => "Controls",
        }
    }
    fn all() -> [Self; 3] {
        [Self::Audio, Self::Graphics, Self::Controls]
    }
}

/// Tracks navigation state inside the settings panel.
#[derive(Resource, Default)]
pub struct SettingsState {
    pub current_tab: SettingsTab,
    /// Index of the currently highlighted row inside the active tab.
    pub selected_item: usize,
    pub input_cooldown: f32,
}

impl SettingsState {
    fn item_count(&self) -> usize {
        match self.current_tab {
            SettingsTab::Audio    => 3,   // Master, SFX, Music
            SettingsTab::Graphics => 1,   // Fullscreen toggle
            SettingsTab::Controls => 0,   // Read-only; nothing to select
        }
    }
}

// Marker components so we can find and update individual elements.
// All pub(crate) so that `settings_input` (also pub(crate)) can reference
// them in its Query parameters without triggering private_interfaces errors.

#[derive(Component)]
pub(crate) struct TabButton(SettingsTab);

#[derive(Component)]
pub(crate) struct SliderFill(usize); // 0=Master, 1=SFX, 2=Music

#[derive(Component)]
pub(crate) struct SliderLabel(usize);

#[derive(Component)]
pub(crate) struct FullscreenLabel;

#[derive(Component)]
pub(crate) struct SettingsRowHighlight(usize);

// ── toggle_pause ──────────────────────────────────────────────────

fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut next_state: ResMut<NextState<GameState>>,
    shop_state: Option<Res<crate::shop::ShopUiState>>,
    altar_state: Option<Res<crate::altar::AltarState>>,
    floor_state: Option<Res<crate::floor_complete::FloorCompleteState>>,
    relic_state: Option<Res<crate::relic::RelicChoiceState>>,
) {
    if shop_state.map_or(false, |s| s.active)   { return; }
    if altar_state.map_or(false, |s| s.active)  { return; }
    if floor_state.map_or(false, |s| s.active)  { return; }
    if relic_state.map_or(false, |s| s.active)  { return; }

    let gp  = gamepads.iter().next();
    let esc = keys.just_pressed(KeyCode::Escape)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::Start));
    if esc {
        next_state.set(GameState::Paused);
    }
}

// ── spawn_pause_menu ──────────────────────────────────────────────

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
                        TextFont { font: font.0.clone(), font_size: 16.0, ..default() },
                        TextColor(Color::srgb(0.9, 0.8, 0.5)),
                        Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
                    ));

                    // Menu items
                    for (i, label) in MENU_ITEMS.iter().enumerate() {
                        let prefix = if i == 0 { "> " } else { "  " };
                        let color  = if i == 0 {
                            Color::srgb(1.0, 0.9, 0.4)
                        } else {
                            Color::srgb(0.6, 0.55, 0.45)
                        };
                        panel.spawn((
                            Text::new(format!("{}{}", prefix, label)),
                            TextFont { font: font.0.clone(), font_size: 10.0, ..default() },
                            TextColor(color),
                            Node { margin: UiRect::vertical(Val::Px(4.0)), ..default() },
                            PauseMenuItem(i),
                        ));
                    }

                    // Controls hint
                    panel.spawn((
                        Text::new("[Up/Down] Select  [Enter/E] Confirm  [Esc] Resume"),
                        TextFont { font: font.0.clone(), font_size: 6.0, ..default() },
                        TextColor(Color::srgb(0.4, 0.35, 0.28)),
                        Node { margin: UiRect::top(Val::Px(12.0)), ..default() },
                    ));
                });
        });
}

// ── despawn_pause_menu ────────────────────────────────────────────

fn despawn_pause_menu(
    mut commands: Commands,
    q: Query<Entity, With<PauseMenuRoot>>,
    settings_q: Query<Entity, With<SettingsPanel>>,
    audio: Option<Res<AudioSettings>>,
) {
    // Persist audio on any exit from the paused state.
    if let Some(a) = audio {
        crate::audio::save_audio_settings(&a);
    }
    for e in &q       { commands.entity(e).try_despawn_recursive(); }
    for e in &settings_q { commands.entity(e).try_despawn_recursive(); }
    commands.remove_resource::<PauseMenuState>();
    commands.remove_resource::<SettingsState>();
}

// ── pause_menu_input ──────────────────────────────────────────────

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
    audio: Res<AudioSettings>,
    windows: Query<&Window>,
) {
    // Don't handle pause-menu input while settings overlay is open.
    if settings_q.iter().next().is_some() { return; }

    let dt = time.delta_secs();
    state.input_cooldown = (state.input_cooldown - dt).max(0.0);
    if state.input_cooldown > 0.0 { return; }

    let gp = gamepads.iter().next();

    let up   = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadUp));
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadDown));

    if up   && state.selected > 0                  { state.selected -= 1; state.input_cooldown = 0.15; }
    if down && state.selected < MENU_ITEMS.len()-1 { state.selected += 1; state.input_cooldown = 0.15; }

    // Update cursor visuals.
    for (item, mut text, mut color) in &mut item_q {
        let prefix = if item.0 == state.selected { "> " } else { "  " };
        **text  = format!("{}{}", prefix, MENU_ITEMS[item.0]);
        *color  = if item.0 == state.selected {
            TextColor(Color::srgb(1.0, 0.9, 0.4))
        } else {
            TextColor(Color::srgb(0.6, 0.55, 0.45))
        };
    }

    // ESC resumes without opening settings.
    let esc = keys.just_pressed(KeyCode::Escape)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::Start));
    if esc {
        crate::audio::save_audio_settings(&audio);
        commands.insert_resource(ResumingFromPause);
        next_state.set(GameState::Playing);
        return;
    }

    // Confirm.
    let confirm = keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::KeyE)
        || keys.just_pressed(KeyCode::Space)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));
    if !confirm { return; }

    match state.selected {
        0 => {
            crate::audio::save_audio_settings(&audio);
            commands.insert_resource(ResumingFromPause);
            next_state.set(GameState::Playing);
        }
        1 => {
            // Open settings overlay.
            let is_fullscreen = windows.iter().next().map_or(false, |w| {
                matches!(w.mode, bevy::window::WindowMode::BorderlessFullscreen(_))
            });
            commands.insert_resource(SettingsState::default());
            spawn_settings_panel(&mut commands, &font.0, &audio, is_fullscreen);
        }
        2 => {
            // Save & Quit.
            if let Ok(player) = player_q.get_single() {
                let spells = if let Ok(s) = slots_q.get_single() {
                    [s.slots[0].is_some(), s.slots[1].is_some(),
                     s.slots[2].is_some(), s.slots[3].is_some()]
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
            crate::audio::save_audio_settings(&audio);
            next_state.set(GameState::Title);
        }
        _ => {}
    }
}

// ── Settings panel spawn ──────────────────────────────────────────

/// Colours used throughout the settings panel.
mod col {
    use bevy::prelude::Color;
    pub const BG:          Color = Color::srgba(0.09, 0.07, 0.05, 0.97);
    pub const TAB_NORMAL:  Color = Color::srgb(0.35, 0.30, 0.22);
    pub const TAB_ACTIVE:  Color = Color::srgb(0.75, 0.65, 0.30);
    pub const ROW_NORMAL:  Color = Color::srgb(0.55, 0.50, 0.38);
    pub const ROW_SELECT:  Color = Color::srgb(1.00, 0.90, 0.40);
    pub const SLIDER_BG:   Color = Color::srgb(0.20, 0.17, 0.12);
    pub const SLIDER_FILL: Color = Color::srgb(0.70, 0.55, 0.20);
    pub const HINT:        Color = Color::srgb(0.35, 0.30, 0.22);
    pub const HEADING:     Color = Color::srgb(0.90, 0.80, 0.50);
}

const SLIDER_W: f32 = 200.0;
const SLIDER_H: f32 = 12.0;

/// Public so title.rs can call it directly.
pub(crate) fn spawn_settings_panel(
    commands: &mut Commands,
    font: &Handle<Font>,
    audio: &AudioSettings,
    is_fullscreen: bool,
) {
    let tab = SettingsTab::Audio;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left:   Val::Percent(12.0),
                right:  Val::Percent(12.0),
                top:    Val::Percent(10.0),
                bottom: Val::Percent(10.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(col::BG),
            GlobalZIndex(210),
            SettingsPanel,
        ))
        .with_children(|root| {
            // ── Header ──────────────────────────────────────
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            )).with_children(|h| {
                h.spawn((
                    Text::new("SETTINGS"),
                    TextFont { font: font.clone(), font_size: 13.0, ..default() },
                    TextColor(col::HEADING),
                ));
            });

            // ── Body: tabs column + content area ─────────────
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                width: Val::Percent(100.0),
                ..default()
            }).with_children(|body| {

                // -- Tab column --
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        width: Val::Px(90.0),
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(4.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.25)),
                )).with_children(|tabs| {
                    for t in SettingsTab::all() {
                        let active = t == tab;
                        let color  = if active { col::TAB_ACTIVE } else { col::TAB_NORMAL };
                        let prefix = if active { "> " } else { "  " };
                        tabs.spawn((
                            Text::new(format!("{}{}", prefix, t.label())),
                            TextFont { font: font.clone(), font_size: 8.5, ..default() },
                            TextColor(color),
                            Node { margin: UiRect::vertical(Val::Px(2.0)), ..default() },
                            TabButton(t),
                        ));
                    }
                });

                // -- Content area --
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        padding: UiRect::all(Val::Px(14.0)),
                        row_gap: Val::Px(10.0),
                        ..default()
                    },
                )).with_children(|content| {
                    spawn_audio_tab(content, font, audio);
                    spawn_graphics_tab_hidden(content, font, is_fullscreen);
                    spawn_controls_tab_hidden(content, font);
                });
            });

            // ── Footer hint ──────────────────────────────────
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.3)),
            )).with_children(|f| {
                f.spawn((
                    Text::new("[Q/E] Tab  [Up/Down] Select  [Left/Right] Adjust  [Esc] Back"),
                    TextFont { font: font.clone(), font_size: 7.0, ..default() },
                    TextColor(Color::srgb(0.55, 0.45, 0.30)),
                ));
            });
        });
}

// ── Tab content builders ──────────────────────────────────────────

#[derive(Component)]
pub(crate) struct AudioTabContent;

#[derive(Component)]
pub(crate) struct GraphicsTabContent;

#[derive(Component)]
pub(crate) struct ControlsTabContent;

fn spawn_audio_tab(parent: &mut ChildBuilder, font: &Handle<Font>, audio: &AudioSettings) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            ..default()
        },
        AudioTabContent,
        // Visible by default (Audio is the first tab).
    )).with_children(|col| {
        let sliders = [
            ("Master Volume", audio.master_volume, 0usize),
            ("SFX Volume",    audio.sfx_volume,    1),
            ("Music Volume",  audio.music_volume,  2),
        ];
        for (label, value, idx) in sliders {
            spawn_slider_row(col, font, label, value, idx, idx == 0);
        }
    });
}

fn spawn_graphics_tab_hidden(parent: &mut ChildBuilder, font: &Handle<Font>, is_fullscreen: bool) {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            width: Val::Percent(100.0),
            display: Display::None, // hidden until tab is switched
            ..default()
        },
        GraphicsTabContent,
    )).with_children(|col| {
        // Row: Fullscreen toggle
        col.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                width: Val::Percent(100.0),
                padding: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            SettingsRowHighlight(0),
        )).with_children(|row| {
            row.spawn((
                Text::new("  Fullscreen"),
                TextFont { font: font.clone(), font_size: 9.0, ..default() },
                TextColor(col::ROW_NORMAL),
                Node { width: Val::Px(120.0), ..default() },
            ));
            let mode_str = if is_fullscreen { "[Fullscreen]" } else { "[Windowed]" };
            row.spawn((
                Text::new(mode_str),
                TextFont { font: font.clone(), font_size: 9.0, ..default() },
                TextColor(col::ROW_SELECT),
                FullscreenLabel,
            ));
        });

        // Hint below
        col.spawn((
            Text::new("Press F11 anytime to toggle fullscreen"),
            TextFont { font: font.clone(), font_size: 6.5, ..default() },
            TextColor(col::HINT),
        ));
    });
}

fn spawn_controls_tab_hidden(parent: &mut ChildBuilder, font: &Handle<Font>) {
    const BINDINGS: &[(&str, &str)] = &[
        ("Move",       "A / D, Arrow Keys"),
        ("Jump",       "Space, W, Up Arrow"),
        ("Attack",     "LMB, J"),
        ("Block",      "RMB, K"),
        ("Ranged",     "F"),
        ("Spell 1-4",  "1 / 2 / 3 / 4"),
        ("Interact",   "E, Enter"),
        ("Dash",       "Left Shift"),
        ("Pause",      "Escape"),
    ];

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            width: Val::Percent(100.0),
            display: Display::None,
            ..default()
        },
        ControlsTabContent,
    )).with_children(|col| {
        col.spawn((
            Text::new("Key Bindings  (read-only)"),
            TextFont { font: font.clone(), font_size: 9.0, ..default() },
            TextColor(col::HEADING),
            Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
        ));

        for &(action, key) in BINDINGS {
            col.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Text::new(format!("{:<12}", action)),
                    TextFont { font: font.clone(), font_size: 8.0, ..default() },
                    TextColor(col::ROW_NORMAL),
                    Node { width: Val::Px(110.0), ..default() },
                ));
                row.spawn((
                    Text::new(key),
                    TextFont { font: font.clone(), font_size: 8.0, ..default() },
                    TextColor(col::ROW_SELECT),
                ));
            });
        }
    });
}

fn spawn_slider_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    value: f32,
    idx: usize,
    selected: bool,
) {
    let text_color = if selected { col::ROW_SELECT } else { col::ROW_NORMAL };
    let prefix     = if selected { "> " } else { "  " };
    let fill_w     = (value * SLIDER_W).clamp(0.0, SLIDER_W);
    let pct        = (value * 100.0).round() as i32;

    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            width: Val::Percent(100.0),
            padding: UiRect::vertical(Val::Px(3.0)),
            ..default()
        },
        SettingsRowHighlight(idx),
    )).with_children(|row| {
        // Label with cursor prefix
        row.spawn((
            Text::new(format!("{}{}", prefix, label)),
            TextFont { font: font.clone(), font_size: 9.0, ..default() },
            TextColor(text_color),
            Node { width: Val::Px(120.0), ..default() },
            SliderLabel(idx),
        ));

        // Track + fill bar
        row.spawn((
            Node {
                width: Val::Px(SLIDER_W),
                height: Val::Px(SLIDER_H),
                ..default()
            },
            BackgroundColor(col::SLIDER_BG),
        )).with_children(|track| {
            track.spawn((
                Node {
                    width: Val::Px(fill_w),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(col::SLIDER_FILL),
                SliderFill(idx),
            ));
        });

        // Percentage text
        row.spawn((
            Text::new(format!("{}%", pct)),
            TextFont { font: font.clone(), font_size: 8.0, ..default() },
            TextColor(col::ROW_SELECT),
            Node { width: Val::Px(36.0), ..default() },
            SliderLabel(idx), // reuse; we'll update both on change
        ));
    });
}

// ── settings_input ────────────────────────────────────────────────

pub(crate) fn settings_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    settings_q: Query<Entity, With<SettingsPanel>>,
    // State
    settings_state: Option<ResMut<SettingsState>>,
    mut audio: ResMut<AudioSettings>,
    mut windows: Query<&mut Window>,
    // UI update queries
    mut tab_q: Query<(&TabButton, &mut Text, &mut TextColor)>,
    mut slider_fill_q: Query<(&SliderFill, &mut Node), Without<SliderLabel>>,
    mut slider_label_q: Query<(&SliderLabel, &mut Text, &mut TextColor), Without<TabButton>>,
    mut fs_label_q: Query<(&FullscreenLabel, &mut Text), (Without<TabButton>, Without<SliderLabel>)>,
    mut row_hl_q: Query<(&SettingsRowHighlight, &mut BackgroundColor)>,
    mut audio_tab_q: Query<(&AudioTabContent, &mut Node), Without<SliderFill>>,
    mut gfx_tab_q: Query<(&GraphicsTabContent, &mut Node), (Without<SliderFill>, Without<AudioTabContent>)>,
    mut ctrl_tab_q: Query<(&ControlsTabContent, &mut Node), (Without<SliderFill>, Without<AudioTabContent>, Without<GraphicsTabContent>)>,
) {
    // Only run when the settings panel exists.
    if settings_q.iter().next().is_none() { return; }
    let Some(mut ss) = settings_state else { return };

    let dt = time.delta_secs();
    ss.input_cooldown = (ss.input_cooldown - dt).max(0.0);

    let gp = gamepads.iter().next();

    // ── ESC / Backspace: close settings ──────────────────────────
    let back = keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Backspace)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if back {
        crate::audio::save_audio_settings(&audio);
        for e in &settings_q {
            commands.entity(e).despawn_recursive();
        }
        commands.remove_resource::<SettingsState>();
        return;
    }

    if ss.input_cooldown > 0.0 { return; }

    // ── Tab switching: Q (prev), E (next), or 1/2/3 ──────────────
    let prev_tab = keys.just_pressed(KeyCode::KeyQ)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::LeftTrigger));
    let next_tab = keys.just_pressed(KeyCode::KeyE)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::RightTrigger));

    let digit_tab: Option<SettingsTab> =
        if keys.just_pressed(KeyCode::Digit1) { Some(SettingsTab::Audio) }
        else if keys.just_pressed(KeyCode::Digit2) { Some(SettingsTab::Graphics) }
        else if keys.just_pressed(KeyCode::Digit3) { Some(SettingsTab::Controls) }
        else { None };

    let new_tab: Option<SettingsTab> = if let Some(t) = digit_tab {
        Some(t)
    } else if prev_tab {
        Some(match ss.current_tab {
            SettingsTab::Audio    => SettingsTab::Controls,
            SettingsTab::Graphics => SettingsTab::Audio,
            SettingsTab::Controls => SettingsTab::Graphics,
        })
    } else if next_tab {
        Some(match ss.current_tab {
            SettingsTab::Audio    => SettingsTab::Graphics,
            SettingsTab::Graphics => SettingsTab::Controls,
            SettingsTab::Controls => SettingsTab::Audio,
        })
    } else {
        None
    };

    if let Some(t) = new_tab {
        ss.current_tab    = t;
        ss.selected_item  = 0;
        ss.input_cooldown = 0.18;
        apply_tab_visibility(&ss, &mut audio_tab_q, &mut gfx_tab_q, &mut ctrl_tab_q);
        update_tab_buttons(&ss, &mut tab_q);
        update_all_sliders(&ss, &audio, &mut slider_fill_q, &mut slider_label_q, &mut row_hl_q);
        return;
    }

    // ── Up / Down within tab ─────────────────────────────────────
    let up   = keys.just_pressed(KeyCode::KeyW) || keys.just_pressed(KeyCode::ArrowUp)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadUp));
    let down = keys.just_pressed(KeyCode::KeyS) || keys.just_pressed(KeyCode::ArrowDown)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadDown));

    let count = ss.item_count();
    if count > 0 {
        if up   && ss.selected_item > 0         { ss.selected_item -= 1; ss.input_cooldown = 0.15; }
        if down && ss.selected_item < count - 1 { ss.selected_item += 1; ss.input_cooldown = 0.15; }
    }

    // ── Left / Right: change value ────────────────────────────────
    // Allow holding left/right for continuous slider adjustment
    let left  = keys.pressed(KeyCode::ArrowLeft)  || keys.pressed(KeyCode::KeyA)
        || gp.map_or(false, |g| g.pressed(GamepadButton::DPadLeft));
    let right = keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD)
        || gp.map_or(false, |g| g.pressed(GamepadButton::DPadRight));

    // Enter / confirm for toggles
    let confirm = keys.just_pressed(KeyCode::Enter)
        || keys.just_pressed(KeyCode::Space)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));

    match ss.current_tab {
        SettingsTab::Audio => {
            let delta: f32 = if right { 0.05 } else if left { -0.05 } else { 0.0 };
            if delta != 0.0 {
                ss.input_cooldown = 0.10;
                match ss.selected_item {
                    0 => audio.master_volume = (audio.master_volume + delta).clamp(0.0, 1.0),
                    1 => audio.sfx_volume    = (audio.sfx_volume    + delta).clamp(0.0, 1.0),
                    2 => audio.music_volume  = (audio.music_volume  + delta).clamp(0.0, 1.0),
                    _ => {}
                }
            }
        }
        SettingsTab::Graphics => {
            if left || right || confirm {
                ss.input_cooldown = 0.20;
                if let Ok(mut window) = windows.get_single_mut() {
                    window.mode = match window.mode {
                        bevy::window::WindowMode::BorderlessFullscreen(_) => {
                            bevy::window::WindowMode::Windowed
                        }
                        _ => bevy::window::WindowMode::BorderlessFullscreen(
                            bevy::window::MonitorSelection::Current,
                        ),
                    };
                    let is_fs = matches!(
                        window.mode,
                        bevy::window::WindowMode::BorderlessFullscreen(_)
                    );
                    // Update label
                    for (_marker, mut text) in &mut fs_label_q {
                        **text = if is_fs { "[Fullscreen]".into() } else { "[Windowed]".into() };
                    }
                }
            }
        }
        SettingsTab::Controls => {
            // Read-only — nothing to do.
        }
    }

    // Always refresh slider visuals so they reflect the latest values.
    update_all_sliders(&ss, &audio, &mut slider_fill_q, &mut slider_label_q, &mut row_hl_q);
}

// ── Visual-update helpers ─────────────────────────────────────────

fn apply_tab_visibility(
    ss: &SettingsState,
    audio_tab_q:  &mut Query<(&AudioTabContent,    &mut Node), Without<SliderFill>>,
    gfx_tab_q:   &mut Query<(&GraphicsTabContent,  &mut Node), (Without<SliderFill>, Without<AudioTabContent>)>,
    ctrl_tab_q:  &mut Query<(&ControlsTabContent,  &mut Node), (Without<SliderFill>, Without<AudioTabContent>, Without<GraphicsTabContent>)>,
) {
    for (_m, mut n) in audio_tab_q.iter_mut() {
        n.display = if ss.current_tab == SettingsTab::Audio    { Display::Flex } else { Display::None };
    }
    for (_m, mut n) in gfx_tab_q.iter_mut() {
        n.display = if ss.current_tab == SettingsTab::Graphics { Display::Flex } else { Display::None };
    }
    for (_m, mut n) in ctrl_tab_q.iter_mut() {
        n.display = if ss.current_tab == SettingsTab::Controls { Display::Flex } else { Display::None };
    }
}

fn update_tab_buttons(
    ss:     &SettingsState,
    tab_q:  &mut Query<(&TabButton, &mut Text, &mut TextColor)>,
) {
    for (tb, mut text, mut color) in tab_q.iter_mut() {
        let active = tb.0 == ss.current_tab;
        let prefix = if active { "> " } else { "  " };
        **text  = format!("{}{}", prefix, tb.0.label());
        *color  = if active { TextColor(col::TAB_ACTIVE) } else { TextColor(col::TAB_NORMAL) };
    }
}

fn update_all_sliders(
    ss:               &SettingsState,
    audio:            &AudioSettings,
    fill_q:           &mut Query<(&SliderFill,  &mut Node), Without<SliderLabel>>,
    label_q:          &mut Query<(&SliderLabel, &mut Text, &mut TextColor), Without<TabButton>>,
    row_hl_q:         &mut Query<(&SettingsRowHighlight, &mut BackgroundColor)>,
) {
    let values = [audio.master_volume, audio.sfx_volume, audio.music_volume];
    let labels = ["Master Volume", "SFX Volume", "Music Volume"];

    // Update fill widths.
    for (fill, mut node) in fill_q.iter_mut() {
        if fill.0 < values.len() {
            node.width = Val::Px((values[fill.0] * SLIDER_W).clamp(0.0, SLIDER_W));
        }
    }

    // Update label texts & colours.
    // SliderLabel(idx) is placed on both the name-text and the percent-text;
    // we distinguish them by checking whether the text starts with a digit.
    let selected = ss.selected_item;
    for (lbl, mut text, mut color) in label_q.iter_mut() {
        let idx = lbl.0;
        if idx >= values.len() { continue; }
        let is_selected = ss.current_tab == SettingsTab::Audio && idx == selected;
        *color = if is_selected { TextColor(col::ROW_SELECT) } else { TextColor(col::ROW_NORMAL) };

        // Identify which text node this is: the name label or the percentage.
        // The name label starts with a cursor prefix or a letter; percent starts with digit.
        let first_char = text.chars().next();
        if first_char.map_or(false, |c| c.is_ascii_digit()) {
            // Percentage text
            **text = format!("{}%", (values[idx] * 100.0).round() as i32);
        } else {
            // Name label
            let prefix = if is_selected { "> " } else { "  " };
            **text = format!("{}{}", prefix, labels[idx]);
        }
    }

    // Row highlight backgrounds.
    for (rh, mut bg) in row_hl_q.iter_mut() {
        let is_selected = ss.current_tab == SettingsTab::Audio && rh.0 == selected;
        bg.0 = if is_selected {
            Color::srgba(0.55, 0.45, 0.15, 0.18)
        } else {
            Color::NONE
        };
    }
}
