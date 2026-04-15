use bevy::prelude::*;
use crate::{
    GameState, GameFont, PlayingEntity,
    player::Player,
    enemy::EnemyDefeated,
    spell::SpellSlots,
    room::RoomTransition,
    audio::{PlaySfxEvent, SfxType},
};

// ---------------------------------------------------------------------------
// Relic icon assets
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct RelicIconAssets {
    pub berserkers_edge:  Handle<Image>,
    pub vampiric_fang:    Handle<Image>,
    pub golden_idol:      Handle<Image>,
    pub iron_heart:       Handle<Image>,
    pub swift_boots:      Handle<Image>,
    pub arcane_orb:       Handle<Image>,
    pub chrono_bracelet:  Handle<Image>,
    pub warriors_band:    Handle<Image>,
    pub stone_skin:       Handle<Image>,
    pub guardian_amulet:  Handle<Image>,
}

impl RelicIconAssets {
    pub fn handle_for(&self, relic: Relic) -> &Handle<Image> {
        match relic {
            Relic::BerserkersEdge  => &self.berserkers_edge,
            Relic::VampiricFang    => &self.vampiric_fang,
            Relic::GoldenIdol      => &self.golden_idol,
            Relic::IronHeart       => &self.iron_heart,
            Relic::SwiftBoots      => &self.swift_boots,
            Relic::ArcaneOrb       => &self.arcane_orb,
            Relic::ChronoBracelet  => &self.chrono_bracelet,
            Relic::WarriorsBand    => &self.warriors_band,
            Relic::StoneSkin       => &self.stone_skin,
            Relic::GuardianAmulet  => &self.guardian_amulet,
        }
    }
}

/// Fired when an elite enemy dies with a 20% chance — adds a random relic to inventory.
#[derive(Event)]
pub struct RelicDropEvent;

pub struct RelicPlugin;

impl Plugin for RelicPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let icons = RelicIconAssets {
            berserkers_edge: asset_server.load("berserkeredge.png"),
            vampiric_fang:   asset_server.load("vampirefang.png"),
            golden_idol:     asset_server.load("goldenidol.png"),
            iron_heart:      asset_server.load("ironheart.png"),
            swift_boots:     asset_server.load("swiftboots.png"),
            arcane_orb:      asset_server.load("arcaneorb.png"),
            chrono_bracelet: asset_server.load("chronobracelet.png"),
            warriors_band:   asset_server.load("warriorsband.png"),
            stone_skin:      asset_server.load("stoneskin.png"),
            guardian_amulet: asset_server.load("amulet.png"),
        };
        let _ = asset_server;
        app.insert_resource(icons)
            .insert_resource(RelicInventory::default())
            .insert_resource(RelicChoiceState::default())
            .add_event::<RelicDropEvent>()
            .add_systems(OnEnter(GameState::Playing), reset_relics)
            .add_systems(
                Update,
                (
                    spawn_relic_choice_ui,
                    relic_choice_input,
                    vampiric_fang_system,
                    chrono_bracelet_system,
                    guardian_amulet_system,
                    handle_relic_drop,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Relic definitions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Relic {
    /// +15% crit chance
    BerserkersEdge,
    /// Heal 1 HP per 5 kills
    VampiricFang,
    /// +25% gold drops
    GoldenIdol,
    /// +2 max HP
    IronHeart,
    /// +20% move speed
    SwiftBoots,
    /// Mana regen +50%
    ArcaneOrb,
    /// -25% spell cooldowns
    ChronoBracelet,
    /// +1 melee damage
    WarriorsBand,
    /// Take 20% less damage
    StoneSkin,
    /// Start each room with a shield
    GuardianAmulet,
}

impl Relic {
    pub fn name(&self) -> &'static str {
        match self {
            Relic::BerserkersEdge => "Berserker's Edge",
            Relic::VampiricFang => "Vampiric Fang",
            Relic::GoldenIdol => "Golden Idol",
            Relic::IronHeart => "Iron Heart",
            Relic::SwiftBoots => "Swift Boots",
            Relic::ArcaneOrb => "Arcane Orb",
            Relic::ChronoBracelet => "Chrono Bracelet",
            Relic::WarriorsBand => "Warrior's Band",
            Relic::StoneSkin => "Stone Skin",
            Relic::GuardianAmulet => "Guardian Amulet",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Relic::BerserkersEdge => "+15% Crit Chance",
            Relic::VampiricFang => "Heal 1 HP per 5 kills",
            Relic::GoldenIdol => "+25% Gold Drops",
            Relic::IronHeart => "+2 Max HP",
            Relic::SwiftBoots => "+20% Move Speed",
            Relic::ArcaneOrb => "+50% Mana Regen",
            Relic::ChronoBracelet => "-25% Spell Cooldowns",
            Relic::WarriorsBand => "+1 Melee Damage",
            Relic::StoneSkin => "-20% Damage Taken",
            Relic::GuardianAmulet => "Shield at room start",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Relic::BerserkersEdge => Color::srgb(0.9, 0.3, 0.2),
            Relic::VampiricFang => Color::srgb(0.7, 0.1, 0.3),
            Relic::GoldenIdol => Color::srgb(1.0, 0.85, 0.2),
            Relic::IronHeart => Color::srgb(0.6, 0.6, 0.65),
            Relic::SwiftBoots => Color::srgb(0.2, 0.8, 0.4),
            Relic::ArcaneOrb => Color::srgb(0.4, 0.3, 0.9),
            Relic::ChronoBracelet => Color::srgb(0.3, 0.7, 0.9),
            Relic::WarriorsBand => Color::srgb(0.8, 0.5, 0.2),
            Relic::StoneSkin => Color::srgb(0.5, 0.45, 0.4),
            Relic::GuardianAmulet => Color::srgb(0.3, 0.6, 1.0),
        }
    }

    const ALL: [Relic; 10] = [
        Relic::BerserkersEdge,
        Relic::VampiricFang,
        Relic::GoldenIdol,
        Relic::IronHeart,
        Relic::SwiftBoots,
        Relic::ArcaneOrb,
        Relic::ChronoBracelet,
        Relic::WarriorsBand,
        Relic::StoneSkin,
        Relic::GuardianAmulet,
    ];
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// The player's collected relics for this run.
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct RelicInventory {
    pub relics: Vec<Relic>,
    /// Kill counter for Vampiric Fang
    pub kill_counter: i32,
}

impl RelicInventory {
    pub fn has(&self, relic: Relic) -> bool {
        self.relics.contains(&relic)
    }
}

/// State for the relic choice overlay.
#[derive(Resource, Default)]
pub struct RelicChoiceState {
    pub active: bool,
    pub ui_spawned: bool,
    pub choices: [Option<Relic>; 3],
    pub selected: usize,
    pub input_cooldown: f32,
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

#[derive(Component)]
struct RelicChoiceUI;

#[derive(Component)]
struct RelicCard(usize);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn reset_relics(mut inventory: ResMut<RelicInventory>, resuming: Option<Res<crate::ResumingFromPause>>) {
    if resuming.is_some() { return; }
    *inventory = RelicInventory::default();
}

/// Generate 3 random relic choices (excluding already owned).
pub fn start_relic_choice(state: &mut RelicChoiceState, inventory: &RelicInventory, seed: u64) {
    let available: Vec<Relic> = Relic::ALL
        .iter()
        .copied()
        .filter(|r| !inventory.has(*r))
        .collect();

    if available.is_empty() {
        state.active = false;
        return;
    }

    let mut choices = [None; 3];
    let mut used = Vec::new();
    for i in 0..3 {
        if available.len() <= used.len() { break; }
        let mut idx = ((seed.wrapping_mul(2654435761).wrapping_add(i as u64 * 7919)) % available.len() as u64) as usize;
        // Find unused
        let mut tries = 0;
        while used.contains(&idx) && tries < available.len() {
            idx = (idx + 1) % available.len();
            tries += 1;
        }
        if !used.contains(&idx) {
            choices[i] = Some(available[idx]);
            used.push(idx);
        }
    }

    state.active = true;
    state.ui_spawned = false;
    state.choices = choices;
    state.selected = 0;
    state.input_cooldown = 0.3;
}

fn spawn_relic_choice_ui(
    mut commands: Commands,
    mut state: ResMut<RelicChoiceState>,
    font: Res<GameFont>,
    icon_assets: Res<RelicIconAssets>,
) {
    if !state.active || state.ui_spawned {
        return;
    }
    state.ui_spawned = true;

    commands
        .spawn((
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
            RelicChoiceUI,
            PlayingEntity,
        ))
        .with_children(|parent| {
            let f = font.0.clone();

            // Title
            parent.spawn((
                Text::new("Choose a Relic"),
                TextFont { font: f.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.95, 0.8, 0.3)),
                Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
            ));

            // Card container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|row| {
                    for (i, choice) in state.choices.iter().enumerate() {
                        let Some(relic) = choice else { continue };
                        let is_selected = i == 0;
                        let border_color = if is_selected {
                            Color::srgb(1.0, 0.9, 0.4)
                        } else {
                            Color::srgb(0.3, 0.25, 0.2)
                        };

                        row.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(12.0)),
                                border: UiRect::all(Val::Px(2.0)),
                                row_gap: Val::Px(6.0),
                                width: Val::Px(140.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.08, 0.06, 0.04, 0.95)),
                            BorderColor(border_color),
                            RelicCard(i),
                        ))
                        .with_children(|card| {
                            // Relic name
                            card.spawn((
                                Text::new(relic.name()),
                                TextFont { font: f.clone(), font_size: 8.0, ..default() },
                                TextColor(relic.color()),
                            ));
                            // Relic icon sprite
                            spawn_relic_icon(card, *relic, &icon_assets);
                            // Description
                            card.spawn((
                                Text::new(relic.description()),
                                TextFont { font: f.clone(), font_size: 7.0, ..default() },
                                TextColor(Color::srgb(0.7, 0.65, 0.55)),
                            ));
                        });
                    }
                });

            // Controls hint
            parent.spawn((
                Text::new("[A/D] Select  [SPACE/Enter] Confirm"),
                TextFont { font: f.clone(), font_size: 7.0, ..default() },
                TextColor(Color::srgb(0.5, 0.45, 0.35)),
                Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
            ));
        });
}

fn relic_choice_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
    mut state: ResMut<RelicChoiceState>,
    mut inventory: ResMut<RelicInventory>,
    mut commands: Commands,
    ui_q: Query<Entity, With<RelicChoiceUI>>,
    mut card_q: Query<(&RelicCard, &mut BorderColor)>,
    mut player_q: Query<&mut Player>,
    mut recalc_ev: EventWriter<crate::equipment::RecalcStats>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
) {
    if !state.active { return; }

    let dt = time.delta_secs();
    state.input_cooldown = (state.input_cooldown - dt).max(0.0);
    if state.input_cooldown > 0.0 { return; }

    let gp = gamepads.iter().next();

    // Count valid choices
    let valid_count = state.choices.iter().filter(|c| c.is_some()).count();
    if valid_count == 0 {
        state.active = false;
        return;
    }

    // Navigate
    let left = keys.just_pressed(KeyCode::KeyA) || keys.just_pressed(KeyCode::ArrowLeft)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadLeft));
    let right = keys.just_pressed(KeyCode::KeyD) || keys.just_pressed(KeyCode::ArrowRight)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadRight));

    if left && state.selected > 0 {
        state.selected -= 1;
        state.input_cooldown = 0.12;
    }
    if right && state.selected < valid_count - 1 {
        state.selected += 1;
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

    // Confirm
    let confirm = keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));
    if !confirm { return; }

    // Apply chosen relic
    if let Some(relic) = state.choices[state.selected] {
        inventory.relics.push(relic);
        ev_sfx.send(PlaySfxEvent(SfxType::RelicPickup));

        // Apply immediate effects
        if let Ok(mut player) = player_q.get_single_mut() {
            match relic {
                Relic::IronHeart => {
                    player.max_health += 2;
                    player.health += 2;
                }
                Relic::SwiftBoots => {
                    player.bonus_speed += 0.2;
                }
                Relic::WarriorsBand => {
                    player.bonus_attack += 1;
                }
                _ => {}
            }
        }
        // Trigger stat recalc so relic bonuses are applied immediately
        recalc_ev.send(crate::equipment::RecalcStats);
    }

    // Close UI
    state.active = false;
    state.ui_spawned = false;
    for e in &ui_q {
        commands.entity(e).despawn_recursive();
    }
}

/// VampiricFang: heal 1 HP every 5 kills.
fn vampiric_fang_system(
    mut inventory: ResMut<RelicInventory>,
    mut ev_defeated: EventReader<EnemyDefeated>,
    mut player_q: Query<&mut Player>,
) {
    if !inventory.has(Relic::VampiricFang) {
        for _ in ev_defeated.read() {} // drain
        return;
    }
    let kill_count = ev_defeated.read().count() as i32;
    if kill_count == 0 { return; }
    inventory.kill_counter += kill_count;
    while inventory.kill_counter >= 5 {
        inventory.kill_counter -= 5;
        if let Ok(mut player) = player_q.get_single_mut() {
            player.health = (player.health + 1).min(player.max_health);
        }
    }
}

/// ChronoBracelet: reduce spell cooldowns by 25%.
fn chrono_bracelet_system(
    inventory: Res<RelicInventory>,
    mut slots_q: Query<&mut SpellSlots>,
    time: Res<Time>,
) {
    if !inventory.has(Relic::ChronoBracelet) { return; }
    let Ok(mut slots) = slots_q.get_single_mut() else { return };
    // Apply extra cooldown reduction (25% faster = subtract extra 25% per frame)
    let bonus_tick = time.delta_secs() * 0.25;
    for cd in slots.cooldowns.iter_mut() {
        if *cd > 0.0 {
            *cd = (*cd - bonus_tick).max(0.0);
        }
    }
}

/// GuardianAmulet: activate block at the start of each room.
fn guardian_amulet_system(
    inventory: Res<RelicInventory>,
    mut ev_room: EventReader<RoomTransition>,
    mut player_q: Query<&mut Player>,
) {
    if !inventory.has(Relic::GuardianAmulet) {
        for _ in ev_room.read() {} // drain
        return;
    }
    for _ in ev_room.read() {
        if let Ok(mut player) = player_q.get_single_mut() {
            player.is_blocking = true;
            player.block_timer = crate::constants::BLOCK_DURATION;
            player.block_cooldown = crate::constants::BLOCK_COOLDOWN;
        }
    }
}

// ---------------------------------------------------------------------------
// Relic icon sprites
// ---------------------------------------------------------------------------

/// Grants a random relic from the drop event (20% chance drop from elite enemies).
fn handle_relic_drop(
    mut ev: EventReader<RelicDropEvent>,
    mut inventory: ResMut<RelicInventory>,
    mut player_q: Query<&mut Player>,
    mut recalc_ev: EventWriter<crate::equipment::RecalcStats>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
) {
    for _ in ev.read() {
        let available: Vec<Relic> = Relic::ALL
            .iter()
            .copied()
            .filter(|r| !inventory.has(*r))
            .collect();
        if available.is_empty() { continue; }

        // Pick a random relic using a simple random index
        let idx = (rand::random::<u32>() as usize) % available.len();
        let relic = available[idx];
        inventory.relics.push(relic);
        ev_sfx.send(PlaySfxEvent(SfxType::RelicPickup));

        // Apply immediate effects
        if let Ok(mut player) = player_q.get_single_mut() {
            match relic {
                Relic::IronHeart => {
                    player.max_health += 2;
                    player.health += 2;
                }
                Relic::SwiftBoots => {
                    player.bonus_speed += 0.2;
                }
                Relic::WarriorsBand => {
                    player.bonus_attack += 1;
                }
                _ => {}
            }
        }
        recalc_ev.send(crate::equipment::RecalcStats);
    }
}

fn spawn_relic_icon(parent: &mut ChildBuilder, relic: Relic, icons: &RelicIconAssets) {
    parent.spawn((
        ImageNode::new(icons.handle_for(relic).clone()),
        Node {
            width: Val::Px(56.0),
            height: Val::Px(56.0),
            margin: UiRect::vertical(Val::Px(4.0)),
            ..default()
        },
    ));
}
