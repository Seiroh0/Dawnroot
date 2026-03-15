use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, GameFont, PlayingEntity,
    player::Player,
    room::{RoomEntity, RoomState},
};

pub struct AltarPlugin;

impl Plugin for AltarPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AltarState::default())
            .add_systems(
                Update,
                (spawn_altar_ui, altar_input, apply_altar_pulse)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Curse / Blessing definitions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum AltarChoice {
    Blessing(BlessingKind),
    Curse(CurseKind),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlessingKind {
    /// +1 max HP
    Vitality,
    /// +10 max mana
    Wisdom,
    /// Heal 3 HP
    Restoration,
    /// +5% crit chance
    Precision,
    /// +10% gold bonus
    Prosperity,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CurseKind {
    /// +3 attack but -2 max HP
    BloodPact,
    /// +40% speed but -30% max mana
    FrenziedStride,
    /// +25% crit chance but take +1 contact damage from all enemies
    GlassCannon,
    /// Double gold drops but enemies get +50% speed on this floor
    GreedyHeart,
    /// Heal to full but lose all mana
    DarkBargain,
}

impl BlessingKind {
    fn name(&self) -> &'static str {
        match self {
            BlessingKind::Vitality => "Vitality",
            BlessingKind::Wisdom => "Wisdom",
            BlessingKind::Restoration => "Restoration",
            BlessingKind::Precision => "Precision",
            BlessingKind::Prosperity => "Prosperity",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            BlessingKind::Vitality => "+1 Max HP",
            BlessingKind::Wisdom => "+10 Max Mana",
            BlessingKind::Restoration => "Heal 3 HP",
            BlessingKind::Precision => "+5% Crit Chance",
            BlessingKind::Prosperity => "+10% Gold Bonus",
        }
    }

    fn color(&self) -> Color {
        Color::srgb(0.3, 0.8, 1.0)
    }

    const ALL: [BlessingKind; 5] = [
        BlessingKind::Vitality,
        BlessingKind::Wisdom,
        BlessingKind::Restoration,
        BlessingKind::Precision,
        BlessingKind::Prosperity,
    ];
}

impl CurseKind {
    fn name(&self) -> &'static str {
        match self {
            CurseKind::BloodPact => "Blood Pact",
            CurseKind::FrenziedStride => "Frenzied Stride",
            CurseKind::GlassCannon => "Glass Cannon",
            CurseKind::GreedyHeart => "Greedy Heart",
            CurseKind::DarkBargain => "Dark Bargain",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            CurseKind::BloodPact => "+3 Attack, -2 Max HP",
            CurseKind::FrenziedStride => "+40% Speed, -30% Mana",
            CurseKind::GlassCannon => "+25% Crit, +1 Dmg Taken",
            CurseKind::GreedyHeart => "2x Gold, Enemies +50% Spd",
            CurseKind::DarkBargain => "Full Heal, Lose All Mana",
        }
    }

    fn color(&self) -> Color {
        Color::srgb(0.9, 0.2, 0.3)
    }

    const ALL: [CurseKind; 5] = [
        CurseKind::BloodPact,
        CurseKind::FrenziedStride,
        CurseKind::GlassCannon,
        CurseKind::GreedyHeart,
        CurseKind::DarkBargain,
    ];
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// State for the altar choice overlay.
#[derive(Resource, Default)]
pub struct AltarState {
    pub active: bool,
    pub ui_spawned: bool,
    /// Left = Blessing, Right = Curse
    pub blessing: Option<BlessingKind>,
    pub curse: Option<CurseKind>,
    /// 0 = blessing selected, 1 = curse selected
    pub selected: usize,
    pub input_cooldown: f32,
    /// Track active curses for gameplay effects
    pub active_curses: Vec<CurseKind>,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct AltarChoiceUI;

#[derive(Component)]
struct AltarCard(usize);

#[derive(Component)]
pub struct AltarEntity {
    pub interacted: bool,
}

#[derive(Component)]
pub struct AltarGlow {
    pub timer: f32,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Spawn an altar in the room at the given position.
pub fn spawn_altar(commands: &mut Commands, x: f32, y: f32) {
    // Altar base - stone pedestal
    let pedestal_color = Color::srgb(0.35, 0.30, 0.28);
    let pedestal_hi = Color::srgb(0.45, 0.40, 0.36);
    let pillar_color = Color::srgb(0.30, 0.26, 0.24);
    let bowl_color = Color::srgb(0.25, 0.22, 0.20);
    let blessing_glow = Color::srgba(0.3, 0.7, 1.0, 0.4);
    let curse_glow = Color::srgba(0.9, 0.15, 0.25, 0.4);
    let rune_color = Color::srgba(0.8, 0.6, 1.0, 0.6);

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(x, y, Z_TILES + 2.0),
        AltarEntity { interacted: false },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        // Wide stone base
        parent.spawn((
            Sprite { color: pedestal_color, custom_size: Some(Vec2::new(40.0, 10.0)), ..default() },
            Transform::from_xyz(0.0, -10.0, 0.1),
        ));
        // Base top highlight
        parent.spawn((
            Sprite { color: pedestal_hi, custom_size: Some(Vec2::new(38.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, -6.0, 0.15),
        ));
        // Left pillar
        parent.spawn((
            Sprite { color: pillar_color, custom_size: Some(Vec2::new(8.0, 28.0)), ..default() },
            Transform::from_xyz(-14.0, 4.0, 0.1),
        ));
        // Right pillar
        parent.spawn((
            Sprite { color: pillar_color, custom_size: Some(Vec2::new(8.0, 28.0)), ..default() },
            Transform::from_xyz(14.0, 4.0, 0.1),
        ));
        // Bowl / basin on top
        parent.spawn((
            Sprite { color: bowl_color, custom_size: Some(Vec2::new(20.0, 8.0)), ..default() },
            Transform::from_xyz(0.0, 14.0, 0.12),
        ));
        // Blessing glow (left side)
        parent.spawn((
            Sprite { color: blessing_glow, custom_size: Some(Vec2::new(14.0, 14.0)), ..default() },
            Transform::from_xyz(-14.0, 18.0, 0.08),
            AltarGlow { timer: 0.0 },
        ));
        // Curse glow (right side)
        parent.spawn((
            Sprite { color: curse_glow, custom_size: Some(Vec2::new(14.0, 14.0)), ..default() },
            Transform::from_xyz(14.0, 18.0, 0.08),
            AltarGlow { timer: 1.5 },
        ));
        // Rune markings on base
        parent.spawn((
            Sprite { color: rune_color, custom_size: Some(Vec2::new(6.0, 2.0)), ..default() },
            Transform::from_xyz(-6.0, -9.0, 0.16),
        ));
        parent.spawn((
            Sprite { color: rune_color, custom_size: Some(Vec2::new(6.0, 2.0)), ..default() },
            Transform::from_xyz(6.0, -9.0, 0.16),
        ));
        parent.spawn((
            Sprite { color: rune_color, custom_size: Some(Vec2::new(2.0, 4.0)), ..default() },
            Transform::from_xyz(0.0, -10.0, 0.16),
        ));
        // Interaction prompt
        parent.spawn((
            Text2d::new("[E] Pray"),
            TextFont {
                font_size: 7.0,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.7, 0.5, 0.8)),
            Transform::from_xyz(0.0, 26.0, 0.5),
        ));
    });
}

/// Generate random blessing + curse choices.
pub fn start_altar_choice(state: &mut AltarState, seed: u64) {
    let blessing_idx = (seed.wrapping_mul(2654435761) % BlessingKind::ALL.len() as u64) as usize;
    let curse_idx = (seed.wrapping_mul(7919).wrapping_add(13) % CurseKind::ALL.len() as u64) as usize;

    state.active = true;
    state.ui_spawned = false;
    state.blessing = Some(BlessingKind::ALL[blessing_idx]);
    state.curse = Some(CurseKind::ALL[curse_idx]);
    state.selected = 0;
    state.input_cooldown = 0.3;
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn spawn_altar_ui(
    mut commands: Commands,
    mut state: ResMut<AltarState>,
    font: Res<GameFont>,
) {
    if !state.active || state.ui_spawned { return; }
    state.ui_spawned = true;

    let blessing = state.blessing.unwrap();
    let curse = state.curse.unwrap();

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(12.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.01, 0.04, 0.88)),
        GlobalZIndex(150),
        AltarChoiceUI,
        PlayingEntity,
    )).with_children(|parent| {
        let f = font.0.clone();

        // Title
        parent.spawn((
            Text::new("Ancient Altar"),
            TextFont { font: f.clone(), font_size: 14.0, ..default() },
            TextColor(Color::srgb(0.8, 0.6, 1.0)),
            Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
        ));

        // Subtitle
        parent.spawn((
            Text::new("Choose your fate"),
            TextFont { font: f.clone(), font_size: 8.0, ..default() },
            TextColor(Color::srgb(0.6, 0.5, 0.7)),
            Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
        ));

        // Card container
        parent.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            ..default()
        }).with_children(|row| {
            // Blessing card (index 0)
            let b_border = Color::srgb(1.0, 0.9, 0.4);
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(8.0),
                    width: Val::Px(160.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.08, 0.15, 0.95)),
                BorderColor(b_border),
                AltarCard(0),
            )).with_children(|card| {
                // Label
                card.spawn((
                    Text::new("BLESSING"),
                    TextFont { font: f.clone(), font_size: 8.0, ..default() },
                    TextColor(Color::srgb(0.3, 0.8, 1.0)),
                ));
                // Icon (blue square)
                card.spawn((
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(blessing.color()),
                ));
                // Name
                card.spawn((
                    Text::new(blessing.name()),
                    TextFont { font: f.clone(), font_size: 9.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.9, 0.95)),
                ));
                // Description
                card.spawn((
                    Text::new(blessing.description()),
                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                    TextColor(Color::srgb(0.6, 0.8, 0.9)),
                ));
                // Safety label
                card.spawn((
                    Text::new("(Safe)"),
                    TextFont { font: f.clone(), font_size: 6.0, ..default() },
                    TextColor(Color::srgb(0.4, 0.7, 0.5)),
                ));
            });

            // Curse card (index 1)
            let c_border = Color::srgb(0.3, 0.25, 0.2);
            row.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    row_gap: Val::Px(8.0),
                    width: Val::Px(160.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.04, 0.04, 0.95)),
                BorderColor(c_border),
                AltarCard(1),
            )).with_children(|card| {
                // Label
                card.spawn((
                    Text::new("CURSE"),
                    TextFont { font: f.clone(), font_size: 8.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.2, 0.3)),
                ));
                // Icon (red square)
                card.spawn((
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(curse.color()),
                ));
                // Name
                card.spawn((
                    Text::new(curse.name()),
                    TextFont { font: f.clone(), font_size: 9.0, ..default() },
                    TextColor(Color::srgb(0.95, 0.85, 0.8)),
                ));
                // Description
                card.spawn((
                    Text::new(curse.description()),
                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                    TextColor(Color::srgb(0.9, 0.6, 0.5)),
                ));
                // Risk label
                card.spawn((
                    Text::new("(Risky)"),
                    TextFont { font: f.clone(), font_size: 6.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.3, 0.2)),
                ));
            });
        });

        // Controls hint
        parent.spawn((
            Text::new("[A/D] Select  [SPACE/Enter] Confirm  [ESC] Skip"),
            TextFont { font: f.clone(), font_size: 7.0, ..default() },
            TextColor(Color::srgb(0.5, 0.45, 0.35)),
            Node { margin: UiRect::top(Val::Px(12.0)), ..default() },
        ));
    });
}

fn altar_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut state: ResMut<AltarState>,
    mut commands: Commands,
    ui_q: Query<Entity, With<AltarChoiceUI>>,
    mut card_q: Query<(&AltarCard, &mut BorderColor)>,
    mut player_q: Query<&mut Player>,
    mut stats: ResMut<crate::equipment::PlayerStats>,
) {
    if !state.active { return; }

    let dt = time.delta_secs();
    state.input_cooldown = (state.input_cooldown - dt).max(0.0);
    if state.input_cooldown > 0.0 { return; }

    let gp = gamepads.iter().next();

    // Navigate
    let left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadLeft));
    let right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadRight));

    if left && state.selected > 0 {
        state.selected = 0;
        state.input_cooldown = 0.12;
    }
    if right && state.selected < 1 {
        state.selected = 1;
        state.input_cooldown = 0.12;
    }

    // Update card borders
    for (card, mut border) in &mut card_q {
        *border = if card.0 == state.selected {
            BorderColor(Color::srgb(1.0, 0.9, 0.4))
        } else {
            BorderColor(Color::srgb(0.3, 0.25, 0.2))
        };
    }

    // Skip (ESC)
    let skip = keys.just_pressed(KeyCode::Escape)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if skip {
        close_altar(&mut state, &mut commands, &ui_q);
        return;
    }

    // Confirm
    let confirm = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));
    if !confirm { return; }

    if let Ok(mut player) = player_q.get_single_mut() {
        if state.selected == 0 {
            // Apply blessing
            if let Some(blessing) = state.blessing {
                apply_blessing(blessing, &mut player, &mut stats);
            }
        } else {
            // Apply curse
            if let Some(curse) = state.curse {
                apply_curse(curse, &mut player, &mut stats);
                state.active_curses.push(curse);
            }
        }
    }

    close_altar(&mut state, &mut commands, &ui_q);
}

fn close_altar(
    state: &mut AltarState,
    commands: &mut Commands,
    ui_q: &Query<Entity, With<AltarChoiceUI>>,
) {
    state.active = false;
    state.ui_spawned = false;
    state.blessing = None;
    state.curse = None;
    for e in ui_q {
        commands.entity(e).despawn_recursive();
    }
}

fn apply_blessing(blessing: BlessingKind, player: &mut Player, stats: &mut crate::equipment::PlayerStats) {
    match blessing {
        BlessingKind::Vitality => {
            player.max_health += 1;
            player.health += 1;
        }
        BlessingKind::Wisdom => {
            player.max_mana += 10.0;
            player.mana = (player.mana + 10.0).min(player.max_mana);
        }
        BlessingKind::Restoration => {
            player.health = (player.health + 3).min(player.max_health);
        }
        BlessingKind::Precision => {
            stats.crit_chance += 0.05;
        }
        BlessingKind::Prosperity => {
            stats.gold_bonus += 0.10;
        }
    }
}

fn apply_curse(curse: CurseKind, player: &mut Player, stats: &mut crate::equipment::PlayerStats) {
    match curse {
        CurseKind::BloodPact => {
            player.bonus_attack += 3;
            stats.attack += 3;
            player.max_health = (player.max_health - 2).max(1);
            player.health = player.health.min(player.max_health);
        }
        CurseKind::FrenziedStride => {
            player.bonus_speed += 0.4;
            stats.speed_mult += 0.4;
            player.max_mana *= 0.7;
            player.mana = player.mana.min(player.max_mana);
        }
        CurseKind::GlassCannon => {
            stats.crit_chance += 0.25;
            // +1 damage taken is tracked via active_curses check in combat
        }
        CurseKind::GreedyHeart => {
            stats.gold_bonus += 1.0; // double gold
            // Enemy speed boost tracked via active_curses
        }
        CurseKind::DarkBargain => {
            player.health = player.max_health;
            player.mana = 0.0;
        }
    }
}

/// Pulsing glow animation for altar glow sprites.
fn apply_altar_pulse(
    time: Res<Time>,
    mut glow_q: Query<(&mut AltarGlow, &mut Sprite)>,
) {
    for (mut glow, mut sprite) in &mut glow_q {
        glow.timer += time.delta_secs() * 2.0;
        let alpha = 0.25 + 0.2 * glow.timer.sin();
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

/// Check if player is near an altar and presses E to interact.
pub fn check_altar_interaction(
    keys: &ButtonInput<KeyCode>,
    gamepads: &Query<&Gamepad>,
    player_tf: &Transform,
    altar_q: &mut Query<(&Transform, &mut AltarEntity), Without<Player>>,
    state: &mut AltarState,
    room_state: &RoomState,
) {
    if state.active { return; }

    let gp = gamepads.iter().next();
    let interact = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::West));
    if !interact { return; }

    for (altar_tf, mut altar) in altar_q {
        if altar.interacted { continue; }
        let dist = (player_tf.translation.xy() - altar_tf.translation.xy()).abs();
        if dist.x < 40.0 && dist.y < 50.0 {
            altar.interacted = true;
            let seed = room_state.seed.wrapping_add(room_state.room_index as u64 * 97);
            start_altar_choice(state, seed);
            break;
        }
    }
}
