use bevy::prelude::*;
use crate::{
    GameState, GameFont, RunData, MetaProgression, PlayingEntity,
    room::{RoomState, RoomType, RoomTransition},
    equipment::{ItemId, Equipment, RecalcStats},
    spell::SpellId,
};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                reset_shop_on_transition,
                spawn_merchant_npc,
                merchant_interaction,
                shop_ui_navigation,
                shop_ui_purchase,
                shop_ui_close,
                shop_ui_update_visuals,
                purchase_feedback_decay,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ---------------------------------------------------------------------------
// Shop UI state (resource-flag overlay, not a new GameState)
// ---------------------------------------------------------------------------

/// When active, the shop overlay is shown and player movement is blocked.
#[derive(Resource)]
pub struct ShopUiState {
    pub active: bool,
    pub selected: usize,
    pub items: Vec<ShopEntry>,
    pub purchased: Vec<bool>,
    pub input_cooldown: f32,
}

/// Marker for the merchant NPC entity (separate from dialogue NPCs).
#[derive(Component)]
pub struct MerchantNpc {
    pub interacted: bool,
}

#[derive(Component)]
struct MerchantPrompt;

// ---------------------------------------------------------------------------
// Shop item definitions
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShopTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Clone)]
pub enum ShopEffect {
    HealFull,
    MaxHpUp,
    ManaUp,
    AttackUp,
    SpeedUp,
    DefenseUp,
    UnlockSpell(usize, SpellId),
    EquipItem(ItemId),
}

/// Requirement to unlock a shop item.
#[derive(Clone)]
pub enum UnlockReq {
    None,
    MinFloor(i32),
    MinRuns(i32),
    MinFloorOrRuns(i32, i32),
}

#[derive(Clone)]
pub struct ShopEntry {
    pub name: &'static str,
    pub cost: i32,
    pub effect: ShopEffect,
    pub tier: ShopTier,
    pub unlock: UnlockReq,
}

fn all_shop_entries() -> Vec<ShopEntry> {
    vec![
        // ── Consumables (always available) ──
        ShopEntry {
            name: "Heal Full", cost: 30, effect: ShopEffect::HealFull,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "+1 Max HP", cost: 60, effect: ShopEffect::MaxHpUp,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "+Mana Pool", cost: 45, effect: ShopEffect::ManaUp,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        // ── Stat upgrades (Tier 2) ──
        ShopEntry {
            name: "Attack Up", cost: 55, effect: ShopEffect::AttackUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Speed Up", cost: 50, effect: ShopEffect::SpeedUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Defense Up", cost: 55, effect: ShopEffect::DefenseUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        // ── Spells ──
        ShopEntry {
            name: "Fireball", cost: 50, effect: ShopEffect::UnlockSpell(0, SpellId::Fireball),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Ice Shards", cost: 45, effect: ShopEffect::UnlockSpell(1, SpellId::IceShards),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Lightning", cost: 75, effect: ShopEffect::UnlockSpell(2, SpellId::Lightning),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloorOrRuns(2, 1),
        },
        ShopEntry {
            name: "Shield", cost: 65, effect: ShopEffect::UnlockSpell(3, SpellId::Shield),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloorOrRuns(2, 1),
        },
        // ── Equipment: Tier 1 ──
        ShopEntry {
            name: "Rusty Sword", cost: 35, effect: ShopEffect::EquipItem(ItemId::RustySword),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Leather Tunic", cost: 40, effect: ShopEffect::EquipItem(ItemId::LeatherTunic),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Life Ring", cost: 45, effect: ShopEffect::EquipItem(ItemId::LifeRing),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Mana Stone", cost: 45, effect: ShopEffect::EquipItem(ItemId::ManaStone),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        // ── Equipment: Tier 2 ──
        ShopEntry {
            name: "Steel Blade", cost: 80, effect: ShopEffect::EquipItem(ItemId::SteelBlade),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Chain Mail", cost: 85, effect: ShopEffect::EquipItem(ItemId::ChainMail),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Gold Magnet", cost: 65, effect: ShopEffect::EquipItem(ItemId::GoldMagnet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Speed Boots", cost: 70, effect: ShopEffect::EquipItem(ItemId::SpeedBoots),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Crit Charm", cost: 75, effect: ShopEffect::EquipItem(ItemId::CritCharm),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Fire Amulet", cost: 60, effect: ShopEffect::EquipItem(ItemId::FireAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Ice Amulet", cost: 60, effect: ShopEffect::EquipItem(ItemId::IceAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Storm Amulet", cost: 60, effect: ShopEffect::EquipItem(ItemId::StormAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        // ── Equipment: Tier 3 ──
        ShopEntry {
            name: "Flame Edge", cost: 130, effect: ShopEffect::EquipItem(ItemId::FlameEdge),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Frost Fang", cost: 130, effect: ShopEffect::EquipItem(ItemId::FrostFang),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Thunder Cleaver", cost: 150, effect: ShopEffect::EquipItem(ItemId::ThunderCleaver),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Ember Plate", cost: 140, effect: ShopEffect::EquipItem(ItemId::EmberPlate),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Frost Guard", cost: 140, effect: ShopEffect::EquipItem(ItemId::FrostGuard),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Storm Armor", cost: 145, effect: ShopEffect::EquipItem(ItemId::StormArmor),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Vampire Fang", cost: 100, effect: ShopEffect::EquipItem(ItemId::VampireFang),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Iron Will", cost: 95, effect: ShopEffect::EquipItem(ItemId::IronWill),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
    ]
}

impl UnlockReq {
    fn is_met(&self, current_floor: i32, meta: &MetaProgression) -> bool {
        match self {
            UnlockReq::None => true,
            UnlockReq::MinFloor(f) => current_floor >= *f || meta.best_floor >= *f,
            UnlockReq::MinRuns(r) => meta.runs_completed >= *r,
            UnlockReq::MinFloorOrRuns(f, r) => {
                current_floor >= *f || meta.best_floor >= *f || meta.runs_completed >= *r
            }
        }
    }
}

impl ShopTier {
    fn color(&self) -> Color {
        match self {
            ShopTier::Tier1 => Color::srgb(0.8, 0.7, 0.3),
            ShopTier::Tier2 => Color::srgb(0.4, 0.7, 0.9),
            ShopTier::Tier3 => Color::srgb(0.85, 0.5, 0.9),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            ShopTier::Tier1 => "",
            ShopTier::Tier2 => "[T2] ",
            ShopTier::Tier3 => "[T3] ",
        }
    }
}

// ---------------------------------------------------------------------------
// Resources & components
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct MerchantSpawned(bool);

/// Brief floating text when buying something.
#[derive(Component)]
struct PurchaseFeedback {
    timer: f32,
}

/// Marker for the shop overlay UI root.
#[derive(Component)]
struct ShopOverlayUI;

/// Marker for the item list container inside the shop overlay.
#[derive(Component)]
struct ShopItemList;

/// Marker for the gold display text in the shop overlay.
#[derive(Component)]
struct ShopGoldText;

/// Marker for the merchant dialogue text in the shop overlay.
#[derive(Component)]
struct ShopMerchantText;

/// Marker for individual item row in the shop overlay.
#[derive(Component)]
struct ShopItemRow(usize);

/// Marker for the item name text within a row.
#[derive(Component)]
struct ShopItemName(usize);

/// Marker for the item cost text within a row.
#[derive(Component)]
struct ShopItemCost(usize);

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn reset_shop_on_transition(
    mut ev: EventReader<RoomTransition>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        commands.insert_resource(MerchantSpawned(false));
        commands.remove_resource::<ShopUiState>();
    }
}

/// Spawn a multi-part stone merchant NPC in shop rooms.
fn spawn_merchant_npc(
    mut commands: Commands,
    room_state: Res<RoomState>,
    spawned: Option<Res<MerchantSpawned>>,
) {
    if room_state.current_type != RoomType::Shop { return; }
    if spawned.map_or(false, |s| s.0) { return; }
    commands.insert_resource(MerchantSpawned(true));

    let x = 480.0; // center of room
    let y = 100.0;

    commands.spawn((
        // Invisible collision root
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(40.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(x, y, crate::constants::Z_PLAYER - 0.5),
        MerchantNpc { interacted: false },
        PlayingEntity,
    )).with_children(|p| {
        // ── Stone body: large rounded boulder base ──
        p.spawn((
            Sprite {
                color: Color::srgb(0.38, 0.34, 0.28),
                custom_size: Some(Vec2::new(36.0, 30.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -2.0, 0.1),
        ));
        // Left shoulder bump (rounded stone protrusion)
        p.spawn((
            Sprite {
                color: Color::srgb(0.35, 0.31, 0.25),
                custom_size: Some(Vec2::new(14.0, 18.0)),
                ..default()
            },
            Transform::from_xyz(-16.0, 2.0, 0.08),
        ));
        // Right shoulder bump
        p.spawn((
            Sprite {
                color: Color::srgb(0.36, 0.32, 0.26),
                custom_size: Some(Vec2::new(14.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(16.0, 0.0, 0.08),
        ));
        // ── Head: smaller stone on top ──
        p.spawn((
            Sprite {
                color: Color::srgb(0.42, 0.38, 0.32),
                custom_size: Some(Vec2::new(22.0, 18.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 18.0, 0.2),
        ));
        // Brow ridge (darker overhanging stone)
        p.spawn((
            Sprite {
                color: Color::srgb(0.32, 0.28, 0.22),
                custom_size: Some(Vec2::new(24.0, 6.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 24.0, 0.25),
        ));
        // ── Eyes: glowing amber orbs set in stone ──
        // Left eye socket (dark)
        p.spawn((
            Sprite {
                color: Color::srgb(0.12, 0.10, 0.08),
                custom_size: Some(Vec2::new(6.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(-5.0, 19.0, 0.3),
        ));
        // Left eye glow
        p.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.65, 0.2),
                custom_size: Some(Vec2::new(3.5, 3.0)),
                ..default()
            },
            Transform::from_xyz(-5.0, 19.0, 0.35),
        ));
        // Right eye socket (dark)
        p.spawn((
            Sprite {
                color: Color::srgb(0.12, 0.10, 0.08),
                custom_size: Some(Vec2::new(6.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(5.0, 19.0, 0.3),
        ));
        // Right eye glow
        p.spawn((
            Sprite {
                color: Color::srgb(0.9, 0.65, 0.2),
                custom_size: Some(Vec2::new(3.5, 3.0)),
                ..default()
            },
            Transform::from_xyz(5.0, 19.0, 0.35),
        ));
        // ── Mouth: jagged crack ──
        p.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.12, 0.08),
                custom_size: Some(Vec2::new(10.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 12.0, 0.3),
        ));
        // ── Stone texture details ──
        // Crack line across body
        p.spawn((
            Sprite {
                color: Color::srgb(0.28, 0.24, 0.18),
                custom_size: Some(Vec2::new(20.0, 2.0)),
                ..default()
            },
            Transform::from_xyz(3.0, -6.0, 0.15),
        ));
        // Small moss patch (green tint)
        p.spawn((
            Sprite {
                color: Color::srgb(0.25, 0.40, 0.20),
                custom_size: Some(Vec2::new(8.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(-10.0, -10.0, 0.15),
        ));
        // Another moss patch
        p.spawn((
            Sprite {
                color: Color::srgb(0.22, 0.35, 0.18),
                custom_size: Some(Vec2::new(5.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(12.0, 8.0, 0.15),
        ));
        // ── Stone base: wide flat foundation ──
        p.spawn((
            Sprite {
                color: Color::srgb(0.30, 0.27, 0.22),
                custom_size: Some(Vec2::new(44.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -18.0, 0.05),
        ));
        // Small pebbles around base
        p.spawn((
            Sprite {
                color: Color::srgb(0.33, 0.29, 0.23),
                custom_size: Some(Vec2::new(6.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(-22.0, -20.0, 0.04),
        ));
        p.spawn((
            Sprite {
                color: Color::srgb(0.35, 0.30, 0.24),
                custom_size: Some(Vec2::new(5.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(24.0, -21.0, 0.04),
        ));
        // ── Crystal embedded in stone (valuable look) ──
        p.spawn((
            Sprite {
                color: Color::srgb(0.6, 0.45, 0.9),
                custom_size: Some(Vec2::new(5.0, 7.0)),
                ..default()
            },
            Transform::from_xyz(-8.0, 4.0, 0.2),
        ));
        p.spawn((
            Sprite {
                color: Color::srgb(0.7, 0.55, 0.95),
                custom_size: Some(Vec2::new(3.0, 5.0)),
                ..default()
            },
            Transform::from_xyz(-6.0, 6.0, 0.22),
        ));

        // Interaction prompt (hidden until player near)
        p.spawn((
            Text2d::new("[E] Shop"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.9, 0.75, 0.4, 0.0)),
            Transform::from_xyz(0.0, 38.0, 1.0),
            MerchantPrompt,
        ));
    });
}

/// Detect player proximity to merchant and open shop on interaction.
fn merchant_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    player_q: Query<&Transform, (With<crate::player::Player>, Without<MerchantNpc>)>,
    mut merchant_q: Query<(&Transform, &mut MerchantNpc), Without<crate::player::Player>>,
    mut prompt_q: Query<(&Parent, &mut TextColor), With<MerchantPrompt>>,
    mut commands: Commands,
    run: Res<RunData>,
    meta: Res<MetaProgression>,
    shop_state: Option<Res<ShopUiState>>,
    font: Res<GameFont>,
    overlay_q: Query<Entity, With<ShopOverlayUI>>,
) {
    let Ok(p_tf) = player_q.get_single() else { return };
    // Don't process if shop is already open
    if shop_state.as_ref().map_or(false, |s| s.active) { return; }

    let interact = keys.just_pressed(KeyCode::KeyE)
        || gamepads.iter().next().map_or(false, |g| g.just_pressed(GamepadButton::West));

    for (m_tf, mut merchant) in &mut merchant_q {
        let dist = (p_tf.translation.xy() - m_tf.translation.xy()).length();
        let near = dist < 70.0;

        if near && interact {
            merchant.interacted = true;

            // Build shop inventory
            let available: Vec<ShopEntry> = all_shop_entries()
                .into_iter()
                .filter(|e| e.unlock.is_met(run.current_floor, &meta))
                .collect();
            let selected = pick_shop_items(&available, run.current_floor);
            let count = selected.len();

            // Remove old overlay if any
            for e in &overlay_q {
                commands.entity(e).try_despawn_recursive();
            }

            commands.insert_resource(ShopUiState {
                active: true,
                selected: 0,
                purchased: vec![false; count],
                items: selected,
                input_cooldown: 0.2,
            });

            // Spawn the UI overlay
            spawn_shop_overlay(&mut commands, &font);
        }
    }

    // Update prompt visibility for all merchant prompts
    for (parent, mut color) in &mut prompt_q {
        if let Ok((m_tf, _)) = merchant_q.get(parent.get()) {
            let dist = (p_tf.translation.xy() - m_tf.translation.xy()).length();
            color.0 = if dist < 70.0 && !shop_state.as_ref().map_or(false, |s| s.active) {
                Color::srgba(0.9, 0.75, 0.4, 1.0)
            } else {
                Color::srgba(0.9, 0.75, 0.4, 0.0)
            };
        }
    }
}

/// Spawn the shop overlay UI (centered panel with item list).
fn spawn_shop_overlay(commands: &mut Commands, font: &GameFont) {
    let f = font.0.clone();

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(15.0),
            right: Val::Percent(15.0),
            top: Val::Percent(8.0),
            bottom: Val::Percent(8.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(8.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.06, 0.04, 0.02, 0.94)),
        BorderColor(Color::srgb(0.50, 0.35, 0.15)),
        BorderRadius::all(Val::Px(6.0)),
        ShopOverlayUI,
        PlayingEntity,
    )).with_children(|root| {
        // ── Header row: title + gold ──
        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            margin: UiRect::bottom(Val::Px(4.0)),
            ..default()
        }).with_children(|row| {
            row.spawn((
                Text::new("~ Stone Merchant ~"),
                TextFont { font: f.clone(), font_size: 12.0, ..default() },
                TextColor(Color::srgb(0.9, 0.65, 0.2)),
            ));
            row.spawn((
                Text::new("Gold: ---"),
                TextFont { font: f.clone(), font_size: 10.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.4)),
                ShopGoldText,
            ));
        });

        // ── Merchant dialogue line ──
        root.spawn((
            Text::new("Hmm... take what you need, traveler."),
            TextFont { font: f.clone(), font_size: 8.0, ..default() },
            TextColor(Color::srgb(0.65, 0.55, 0.40)),
            ShopMerchantText,
        ));

        // ── Divider ──
        root.spawn((
            Node {
                height: Val::Px(2.0),
                margin: UiRect::vertical(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.35, 0.25, 0.12)),
        ));

        // ── Item list container ──
        root.spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                overflow: Overflow::clip(),
                flex_grow: 1.0,
                ..default()
            },
            ShopItemList,
        ));

        // ── Controls hint ──
        root.spawn((
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
        )).with_children(|hint_row| {
            hint_row.spawn((
                Text::new("[Up/Down] Select  [E/Enter] Buy  [Esc/B] Close"),
                TextFont { font: f.clone(), font_size: 6.0, ..default() },
                TextColor(Color::srgb(0.45, 0.38, 0.28)),
            ));
        });
    });
}

/// Navigate the shop item list with Up/Down keys or DPad.
fn shop_ui_navigation(
    mut shop_state: Option<ResMut<ShopUiState>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    time: Res<Time>,
) {
    let Some(ref mut state) = shop_state else { return };
    if !state.active { return; }

    state.input_cooldown = (state.input_cooldown - time.delta_secs()).max(0.0);
    if state.input_cooldown > 0.0 { return; }

    let gp = gamepads.iter().next();
    let up = keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadUp));
    let down = keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadDown));

    let count = state.items.len();
    if count == 0 { return; }

    if up {
        state.selected = if state.selected == 0 { count - 1 } else { state.selected - 1 };
        state.input_cooldown = 0.08;
    }
    if down {
        state.selected = (state.selected + 1) % count;
        state.input_cooldown = 0.08;
    }
}

/// Purchase the currently selected item.
fn shop_ui_purchase(
    mut shop_state: Option<ResMut<ShopUiState>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut run: ResMut<RunData>,
    mut player_mut: Query<&mut crate::player::Player, Without<crate::spell::SpellSlots>>,
    mut spell_slots_q: Query<&mut crate::spell::SpellSlots, Without<crate::player::Player>>,
    mut equipment_q: Query<&mut Equipment, With<crate::player::Player>>,
    mut recalc_ev: EventWriter<RecalcStats>,
    mut commands: Commands,
    font: Res<GameFont>,
    merchant_q: Query<&Transform, With<MerchantNpc>>,
    mut merchant_text_q: Query<&mut Text, With<ShopMerchantText>>,
) {
    let Some(ref mut state) = shop_state else { return };
    if !state.active { return; }

    let gp = gamepads.iter().next();
    let buy = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::West));

    if !buy { return; }

    let idx = state.selected;
    if idx >= state.items.len() { return; }

    // Already purchased
    if state.purchased[idx] {
        if let Ok(mut text) = merchant_text_q.get_single_mut() {
            **text = "You already bought that one.".to_string();
        }
        return;
    }

    let item_cost = state.items[idx].cost;
    let item_name = state.items[idx].name.to_string();
    let item_effect = state.items[idx].effect.clone();

    if run.gold < item_cost {
        if let Ok(mut text) = merchant_text_q.get_single_mut() {
            **text = "You don't have enough gold for that...".to_string();
        }
        if let Ok(m_tf) = merchant_q.get_single() {
            spawn_feedback(&mut commands, m_tf.translation, "Not enough gold!", Color::srgb(0.9, 0.3, 0.2), &font);
        }
        return;
    }

    run.gold -= item_cost;
    apply_shop_effect(&item_effect, &mut player_mut, &mut spell_slots_q, &mut equipment_q, &mut recalc_ev);
    state.purchased[idx] = true;

    // Purchase feedback
    if let Ok(m_tf) = merchant_q.get_single() {
        let msg = format!("Bought {}!", item_name);
        spawn_feedback(&mut commands, m_tf.translation, &msg, Color::srgb(0.3, 0.9, 0.3), &font);
    }

    // Update merchant dialogue
    let responses = [
        "A fine choice.",
        "That'll serve you well.",
        "Wise purchase, traveler.",
        "The roots approve.",
        "Use it well down there.",
    ];
    let resp = responses[idx % responses.len()];
    if let Ok(mut text) = merchant_text_q.get_single_mut() {
        **text = resp.to_string();
    }
}

/// Apply a shop effect to the player.
fn apply_shop_effect(
    effect: &ShopEffect,
    player_mut: &mut Query<&mut crate::player::Player, Without<crate::spell::SpellSlots>>,
    spell_slots_q: &mut Query<&mut crate::spell::SpellSlots, Without<crate::player::Player>>,
    equipment_q: &mut Query<&mut Equipment, With<crate::player::Player>>,
    recalc_ev: &mut EventWriter<RecalcStats>,
) {
    match effect {
        ShopEffect::HealFull => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.health = player.max_health;
            }
        }
        ShopEffect::MaxHpUp => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.max_health += 1;
                player.health += 1;
            }
        }
        ShopEffect::ManaUp => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.max_mana += 20.0;
                player.mana += 20.0;
            }
        }
        ShopEffect::AttackUp => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.bonus_attack += 1;
            }
            recalc_ev.send(RecalcStats);
        }
        ShopEffect::DefenseUp => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.bonus_defense += 1;
            }
            recalc_ev.send(RecalcStats);
        }
        ShopEffect::SpeedUp => {
            if let Ok(mut player) = player_mut.get_single_mut() {
                player.bonus_speed += 0.1;
            }
            recalc_ev.send(RecalcStats);
        }
        ShopEffect::UnlockSpell(slot, spell) => {
            if let Ok(mut slots) = spell_slots_q.get_single_mut() {
                slots.slots[*slot] = Some(*spell);
            }
        }
        ShopEffect::EquipItem(item_id) => {
            if let Ok(mut equip) = equipment_q.get_single_mut() {
                equip.equip(*item_id);
                recalc_ev.send(RecalcStats);
            }
        }
    }
}

/// Close the shop overlay on Escape or gamepad East(B).
fn shop_ui_close(
    mut shop_state: Option<ResMut<ShopUiState>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    overlay_q: Query<Entity, With<ShopOverlayUI>>,
) {
    let Some(ref mut state) = shop_state else { return };
    if !state.active { return; }

    let gp = gamepads.iter().next();
    let close = keys.just_pressed(KeyCode::Escape)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));

    if close {
        state.active = false;
        for e in &overlay_q {
            commands.entity(e).try_despawn_recursive();
        }
    }
}

/// Update the shop overlay visuals each frame (gold, selection highlight, purchased items).
fn shop_ui_update_visuals(
    shop_state: Option<Res<ShopUiState>>,
    run: Res<RunData>,
    mut gold_text_q: Query<&mut Text, (With<ShopGoldText>, Without<ShopMerchantText>, Without<ShopItemName>, Without<ShopItemCost>)>,
    mut item_list_q: Query<(Entity, &Children), With<ShopItemList>>,
    item_row_q: Query<&ShopItemRow>,
    mut bg_q: Query<&mut BackgroundColor, With<ShopItemRow>>,
    mut name_q: Query<(&mut Text, &mut TextColor, &ShopItemName), (Without<ShopGoldText>, Without<ShopMerchantText>, Without<ShopItemCost>)>,
    mut cost_q: Query<(&mut Text, &mut TextColor, &ShopItemCost), (Without<ShopGoldText>, Without<ShopMerchantText>, Without<ShopItemName>)>,
    mut commands: Commands,
    font: Res<GameFont>,
) {
    let Some(ref state) = shop_state else { return };
    if !state.active { return; }

    // Update gold text
    if let Ok(mut text) = gold_text_q.get_single_mut() {
        **text = format!("Gold: {}", run.gold);
    }

    // Check if item rows exist; if not, spawn them
    let Ok((list_entity, children)) = item_list_q.get_single_mut() else { return };

    let has_rows = children.iter().any(|c| item_row_q.get(*c).is_ok());

    if !has_rows {
        // Spawn item rows
        let f = font.0.clone();
        commands.entity(list_entity).with_children(|list| {
            for (i, entry) in state.items.iter().enumerate() {
                let purchased = state.purchased[i];
                let selected = i == state.selected;

                let bg = if selected {
                    Color::srgba(0.3, 0.22, 0.10, 0.6)
                } else {
                    Color::srgba(0.0, 0.0, 0.0, 0.0)
                };

                list.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    ShopItemRow(i),
                )).with_children(|row| {
                    let tier_label = entry.tier.label();
                    let name_str = if purchased {
                        format!("{}{} [SOLD]", tier_label, entry.name)
                    } else {
                        format!("{}{}", tier_label, entry.name)
                    };

                    let name_color = if purchased {
                        Color::srgb(0.4, 0.35, 0.28)
                    } else {
                        entry.tier.color()
                    };

                    row.spawn((
                        Text::new(name_str),
                        TextFont { font: f.clone(), font_size: 9.0, ..default() },
                        TextColor(name_color),
                        ShopItemName(i),
                    ));

                    let cost_str = if purchased {
                        "---".to_string()
                    } else {
                        format!("{}g", entry.cost)
                    };
                    let cost_color = if purchased {
                        Color::srgb(0.4, 0.35, 0.28)
                    } else if run.gold >= entry.cost {
                        Color::srgb(0.9, 0.85, 0.4)
                    } else {
                        Color::srgb(0.7, 0.3, 0.2)
                    };

                    row.spawn((
                        Text::new(cost_str),
                        TextFont { font: f.clone(), font_size: 9.0, ..default() },
                        TextColor(cost_color),
                        ShopItemCost(i),
                    ));
                });
            }
        });
        return;
    }

    // Update row backgrounds based on selection
    for (row, mut bg) in std::iter::zip(item_row_q.iter(), bg_q.iter_mut()) {
        if row.0 == state.selected {
            bg.0 = Color::srgba(0.3, 0.22, 0.10, 0.6);
        } else {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
        }
    }

    // Update name texts
    for (mut text, mut color, name) in &mut name_q {
        let i = name.0;
        if i >= state.items.len() { continue; }
        let entry = &state.items[i];
        let purchased = state.purchased[i];
        let tier_label = entry.tier.label();

        **text = if purchased {
            format!("{}{} [SOLD]", tier_label, entry.name)
        } else {
            format!("{}{}", tier_label, entry.name)
        };

        color.0 = if purchased {
            Color::srgb(0.4, 0.35, 0.28)
        } else {
            entry.tier.color()
        };
    }

    // Update cost texts
    for (mut text, mut color, cost) in &mut cost_q {
        let i = cost.0;
        if i >= state.items.len() { continue; }
        let entry = &state.items[i];
        let purchased = state.purchased[i];

        **text = if purchased {
            "---".to_string()
        } else {
            format!("{}g", entry.cost)
        };

        color.0 = if purchased {
            Color::srgb(0.4, 0.35, 0.28)
        } else if run.gold >= entry.cost {
            Color::srgb(0.9, 0.85, 0.4)
        } else {
            Color::srgb(0.7, 0.3, 0.2)
        };
    }
}

fn spawn_feedback(commands: &mut Commands, pos: Vec3, text: &str, color: Color, font: &GameFont) {
    commands.spawn((
        Text2d::new(text.to_string()),
        TextFont { font: font.0.clone(), font_size: 8.0, ..default() },
        TextColor(color),
        Transform::from_xyz(pos.x, pos.y + 40.0, crate::constants::Z_HUD + 1.0),
        PurchaseFeedback { timer: 1.5 },
        PlayingEntity,
    ));
}

fn purchase_feedback_decay(
    mut commands: Commands,
    mut q: Query<(Entity, &mut PurchaseFeedback, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut fb, mut tf) in &mut q {
        fb.timer -= dt;
        tf.translation.y += 30.0 * dt; // float up
        if fb.timer <= 0.0 {
            commands.entity(entity).try_despawn_recursive();
        }
    }
}

/// Maximum items shown in a single shop visit.
const MAX_SHOP_SLOTS: usize = 5;

/// Pick up to MAX_SHOP_SLOTS items from the available pool.
fn pick_shop_items(available: &[ShopEntry], _floor: i32) -> Vec<ShopEntry> {
    use std::collections::HashSet;

    if available.len() <= MAX_SHOP_SLOTS {
        return available.iter().map(|e| ShopEntry {
            name: e.name, cost: e.cost, effect: e.effect.clone(),
            tier: e.tier, unlock: e.unlock.clone(),
        }).collect();
    }

    let mut selected = Vec::new();
    let mut used_indices = HashSet::new();

    // Always include one consumable
    let consumables: Vec<usize> = available.iter().enumerate()
        .filter(|(_, e)| matches!(e.effect, ShopEffect::HealFull | ShopEffect::MaxHpUp | ShopEffect::ManaUp))
        .map(|(i, _)| i)
        .collect();
    if !consumables.is_empty() {
        let idx = consumables[rand::random::<usize>() % consumables.len()];
        used_indices.insert(idx);
        selected.push(idx);
    }

    // Fill remaining slots randomly (Fisher-Yates)
    let mut remaining: Vec<usize> = (0..available.len())
        .filter(|i| !used_indices.contains(i))
        .collect();
    for i in (1..remaining.len()).rev() {
        let j = rand::random::<usize>() % (i + 1);
        remaining.swap(i, j);
    }
    for idx in remaining {
        if selected.len() >= MAX_SHOP_SLOTS { break; }
        selected.push(idx);
    }

    selected.iter().map(|&i| {
        let e = &available[i];
        ShopEntry {
            name: e.name, cost: e.cost, effect: e.effect.clone(),
            tier: e.tier, unlock: e.unlock.clone(),
        }
    }).collect()
}
