use bevy::prelude::*;
use crate::{
    GameState, GameFont, RunData, MetaProgression, PlayingEntity,
    room::{RoomState, RoomType, RoomTransition, RoomEntity},
    equipment::{ItemId, Equipment, RecalcStats},
    spell::SpellId,
    audio::{PlaySfxEvent, SfxType},
};

/// Compute scaled cost for a repeatable stat upgrade.
pub fn stat_upgrade_cost(base: i32, level: i32) -> i32 {
    (base as f32 * (1.0 + level as f32 * 0.65)).round() as i32
}

fn is_stat_upgrade(effect: &ShopEffect) -> bool {
    matches!(effect, ShopEffect::AttackUp | ShopEffect::DefenseUp | ShopEffect::SpeedUp | ShopEffect::MaxHpUp | ShopEffect::ManaUp)
}

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), reset_shop_state)
            .add_systems(
                Update,
                (
                    reset_shop_on_transition,
                    spawn_merchant_npc,
                    merchant_interaction,
                    merchant_anim_system,
                    shop_ui_navigation,
                    shop_ui_purchase,
                    shop_ui_close,
                    shop_ui_update_visuals,
                    shop_card_anim_system,
                    purchase_feedback_decay,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn reset_shop_state(
    mut commands: Commands,
    resuming: Option<Res<crate::ResumingFromPause>>,
) {
    if resuming.is_some() { return; }
    commands.insert_resource(MerchantSpawned(false));
    commands.remove_resource::<ShopUiState>();
}

// ---------------------------------------------------------------------------
// Shop UI state (resource-flag overlay, not a new GameState)
// ---------------------------------------------------------------------------

/// When active, the shop overlay is shown and player movement is blocked.
#[derive(Resource)]
pub struct ShopUiState {
    pub active: bool,
    /// Indexes into `items` (0..4 dealt cards)
    pub selected: usize,
    /// The 4 dealt items for this visit
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

/// Merchant animation states: Idle(0), Talk(1), Sell(2), Reject(3)
#[derive(Component)]
pub struct MerchantAnim {
    pub state: MerchantAnimState,
    pub timer: f32,
    pub bob_t: f32,
    pub base_y: f32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum MerchantAnimState {
    Idle,
    Talk,
    Sell,
    Reject,
}

impl MerchantAnimState {
    fn frame(&self) -> usize {
        match self {
            MerchantAnimState::Idle   => 0,
            MerchantAnimState::Talk   => 1,
            MerchantAnimState::Sell   => 2,
            MerchantAnimState::Reject => 3,
        }
    }
    fn duration(&self) -> f32 {
        match self {
            MerchantAnimState::Idle   => f32::MAX,
            MerchantAnimState::Talk   => 1.5,
            MerchantAnimState::Sell   => 1.2,
            MerchantAnimState::Reject => 1.0,
        }
    }
}

/// Marker on the sprite child of MerchantNpc for animation updates.
#[derive(Component)]
struct MerchantSprite;

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
#[allow(dead_code)]
pub enum UnlockReq {
    None,
    MinFloor(i32),
    MinRuns(i32),
    MinFloorOrRuns(i32, i32),
}

#[derive(Clone)]
pub struct ShopEntry {
    pub name: &'static str,
    pub description: &'static str,
    pub icon_path: &'static str,
    pub cost: i32,
    pub effect: ShopEffect,
    pub tier: ShopTier,
    pub unlock: UnlockReq,
}

fn all_shop_entries() -> Vec<ShopEntry> {
    vec![
        // ── Consumables (always available) ──
        ShopEntry {
            name: "Heiltrank",
            description: "Stellt alle Lebenspunkte vollständig wieder her.",
            icon_path: "healfull.png",
            cost: 30, effect: ShopEffect::HealFull,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "+1 Max LP",
            description: "Erhöht die maximalen Lebenspunkte dauerhaft um 1.",
            icon_path: "+1maxhp.png",
            cost: 55, effect: ShopEffect::MaxHpUp,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "+Mana Vorrat",
            description: "Vergrößert den Manapool dauerhaft um 20.",
            icon_path: "+1manapool.png",
            cost: 40, effect: ShopEffect::ManaUp,
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        // ── Stat upgrades (Tier 2) ──
        ShopEntry {
            name: "Angriff +",
            description: "Steigert den Angriffswert. Kann mehrfach gekauft werden.",
            icon_path: "attackup.png",
            cost: 50, effect: ShopEffect::AttackUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Tempo +",
            description: "Erhöht die Bewegungsgeschwindigkeit dauerhaft.",
            icon_path: "speedup.png",
            cost: 45, effect: ShopEffect::SpeedUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Verteidigung +",
            description: "Erhöht die Verteidigung. Reduziert erlittenen Schaden.",
            icon_path: "defenseup.png",
            cost: 50, effect: ShopEffect::DefenseUp,
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        // ── Spells ──
        ShopEntry {
            name: "Feuerball",
            description: "Schleudere einen explodierenden Feuerball auf Feinde.",
            icon_path: "fireball.png",
            cost: 50, effect: ShopEffect::UnlockSpell(0, SpellId::Fireball),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Eissplitter",
            description: "Feuere mehrere Eissplitter aus, die Feinde verlangsamen.",
            icon_path: "iceshards.png",
            cost: 45, effect: ShopEffect::UnlockSpell(1, SpellId::IceShards),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Blitz",
            description: "Ein kraftvoller Blitzschlag trifft alle Feinde in der Nähe.",
            icon_path: "lightning.png",
            cost: 75, effect: ShopEffect::UnlockSpell(2, SpellId::Lightning),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloorOrRuns(2, 1),
        },
        ShopEntry {
            name: "Schild",
            description: "Erschaffe einen magischen Schutzschild für kurze Zeit.",
            icon_path: "shield-spell.png",
            cost: 65, effect: ShopEffect::UnlockSpell(3, SpellId::Shield),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloorOrRuns(2, 1),
        },
        // ── Equipment: Tier 1 ──
        ShopEntry {
            name: "Rostiges Schwert",
            description: "Ein altes Schwert. Erhöht den Angriff leicht.",
            icon_path: "rustysword.png",
            cost: 35, effect: ShopEffect::EquipItem(ItemId::RustySword),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Ledertunika",
            description: "Einfache Lederrüstung. Gewährt etwas Schutz.",
            icon_path: "leathertunic.png",
            cost: 40, effect: ShopEffect::EquipItem(ItemId::LeatherTunic),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Lebensring",
            description: "Ein Ring, der die maximalen LP dauerhaft erhöht.",
            icon_path: "lifering.png",
            cost: 45, effect: ShopEffect::EquipItem(ItemId::LifeRing),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        ShopEntry {
            name: "Manastein",
            description: "Ein Edelstein, der den Manapool vergrößert.",
            icon_path: "manastone.png",
            cost: 45, effect: ShopEffect::EquipItem(ItemId::ManaStone),
            tier: ShopTier::Tier1, unlock: UnlockReq::None,
        },
        // ── Equipment: Tier 2 ──
        ShopEntry {
            name: "Stahlklinge",
            description: "Eine starke Stahlklinge. Deutlich mehr Schaden.",
            icon_path: "steelblade.png",
            cost: 80, effect: ShopEffect::EquipItem(ItemId::SteelBlade),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Kettenhemd",
            description: "Solides Kettenhemd für besseren Schutz im Kampf.",
            icon_path: "chainmail.png",
            cost: 85, effect: ShopEffect::EquipItem(ItemId::ChainMail),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Goldmagnet",
            description: "Zieht Gold aus der Ferne automatisch an.",
            icon_path: "goldmagnet.png",
            cost: 65, effect: ShopEffect::EquipItem(ItemId::GoldMagnet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Schnellstiefel",
            description: "Leichte Stiefel, die die Bewegung spürbar beschleunigen.",
            icon_path: "speedboots.png",
            cost: 70, effect: ShopEffect::EquipItem(ItemId::SpeedBoots),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Krit-Amulett",
            description: "Erhöht die Chance auf kritische Treffer.",
            icon_path: "critcharm.png",
            cost: 75, effect: ShopEffect::EquipItem(ItemId::CritCharm),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Feuer-Amulett",
            description: "Verstärkt Feuer-Zauber und gibt Feuerresistenz.",
            icon_path: "",
            cost: 60, effect: ShopEffect::EquipItem(ItemId::FireAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Eis-Amulett",
            description: "Verstärkt Eis-Zauber und gibt Kälteresistenz.",
            icon_path: "",
            cost: 60, effect: ShopEffect::EquipItem(ItemId::IceAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        ShopEntry {
            name: "Sturm-Amulett",
            description: "Verstärkt Blitz-Zauber und gibt Blitzresistenz.",
            icon_path: "",
            cost: 60, effect: ShopEffect::EquipItem(ItemId::StormAmulet),
            tier: ShopTier::Tier2, unlock: UnlockReq::MinFloor(2),
        },
        // ── Equipment: Tier 3 ──
        ShopEntry {
            name: "Flammenschneide",
            description: "Eine legendäre Klinge, die in Flammen gehüllt ist.",
            icon_path: "",
            cost: 130, effect: ShopEffect::EquipItem(ItemId::FlameEdge),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Frostfang",
            description: "Ein eisiger Reißzahn, der Feinde einfriert.",
            icon_path: "",
            cost: 130, effect: ShopEffect::EquipItem(ItemId::FrostFang),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Donnerbeil",
            description: "Ein mächtiges Beil, das Blitze entfesselt.",
            icon_path: "",
            cost: 150, effect: ShopEffect::EquipItem(ItemId::ThunderCleaver),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Glutplatten",
            description: "Schwere Plattenrüstung mit Feuerresistenz.",
            icon_path: "",
            cost: 140, effect: ShopEffect::EquipItem(ItemId::EmberPlate),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Frostrüstung",
            description: "Rüstung aus gefrorenem Stahl mit Kälteresistenz.",
            icon_path: "",
            cost: 140, effect: ShopEffect::EquipItem(ItemId::FrostGuard),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Sturmrüstung",
            description: "Rüstung, die statische Energie speichert.",
            icon_path: "",
            cost: 145, effect: ShopEffect::EquipItem(ItemId::StormArmor),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Vampirzahn",
            description: "Stiehlt beim Angriff Lebenspunkte vom Feind.",
            icon_path: "vampirefang.png",
            cost: 100, effect: ShopEffect::EquipItem(ItemId::VampireFang),
            tier: ShopTier::Tier3, unlock: UnlockReq::MinFloorOrRuns(3, 2),
        },
        ShopEntry {
            name: "Eiserner Wille",
            description: "Erhöht Verteidigung und maximale LP erheblich.",
            icon_path: "ironheart.png",
            cost: 95, effect: ShopEffect::EquipItem(ItemId::IronWill),
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
    fn dot_color(&self) -> Color {
        match self {
            ShopTier::Tier1 => Color::srgb(0.65, 0.65, 0.65),
            ShopTier::Tier2 => Color::srgb(0.3, 0.55, 0.95),
            ShopTier::Tier3 => Color::srgb(0.75, 0.35, 0.9),
        }
    }
}

fn fallback_icon_color(effect: &ShopEffect) -> Color {
    match effect {
        ShopEffect::HealFull | ShopEffect::MaxHpUp => Color::srgb(0.8, 0.2, 0.2),
        ShopEffect::ManaUp => Color::srgb(0.2, 0.4, 0.9),
        ShopEffect::AttackUp | ShopEffect::DefenseUp | ShopEffect::SpeedUp => Color::srgb(0.8, 0.6, 0.1),
        ShopEffect::UnlockSpell(_, _) => Color::srgb(0.2, 0.4, 0.9),
        ShopEffect::EquipItem(_) => Color::srgb(0.5, 0.5, 0.5),
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

/// Marker for the gold display text in the shop overlay.
#[derive(Component)]
struct ShopGoldText;

/// Marker for the merchant dialogue text in the shop overlay.
#[derive(Component)]
struct ShopMerchantText;

/// Marker for an individual card in the shop overlay.
#[derive(Component)]
struct ShopCard(usize);

/// Marker for the "Gekauft" overlay on a card.
#[derive(Component)]
struct ShopCardPurchasedOverlay(usize);

/// Marker for the price text on a card.
#[derive(Component)]
struct ShopCardPrice(usize);

/// Marker for the card name text.
#[derive(Component)]
struct ShopCardName(usize);

/// Smooth scale animation for a shop card.
#[derive(Component)]
struct ShopCardAnim {
    card_index: usize,
    target_scale: f32,
    current_scale: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn reset_shop_on_transition(
    mut ev: EventReader<RoomTransition>,
    mut commands: Commands,
    overlay_q: Query<Entity, With<ShopOverlayUI>>,
) {
    for _ in ev.read() {
        commands.insert_resource(MerchantSpawned(false));
        commands.remove_resource::<ShopUiState>();
        for e in &overlay_q {
            commands.entity(e).try_despawn_recursive();
        }
    }
}

/// Spawn the sprite-based merchant NPC in shop rooms.
fn spawn_merchant_npc(
    mut commands: Commands,
    room_state: Res<RoomState>,
    spawned: Option<Res<MerchantSpawned>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    if room_state.current_type != RoomType::Shop { return; }
    if spawned.map_or(false, |s| s.0) { return; }
    commands.insert_resource(MerchantSpawned(true));

    // merchant.png: 192x48, 4 frames @ 48x48 (Idle|Talk|Sell|Reject)
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 48), 4, 1, None, None,
    ));
    let texture: Handle<Image> = asset_server.load("merchant.png");

    // Display at 2x scale → 96x96 in-game
    let display_size = 96.0_f32;
    let x = crate::constants::ROOM_W / 2.0;
    // Stand on top of the center podest (row 2, height 1 tile): top surface = 3 * TILE_SIZE
    let podest_top = 3.0 * crate::constants::TILE_SIZE;
    let y = podest_top + display_size / 2.0;

    commands.spawn((
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(display_size, display_size)),
            ..default()
        },
        Transform::from_xyz(x, y, crate::constants::Z_PLAYER - 0.5),
        MerchantNpc { interacted: false },
        MerchantAnim { state: MerchantAnimState::Idle, timer: 0.0, bob_t: 0.0, base_y: y },
        RoomEntity,
        PlayingEntity,
    )).with_children(|p| {
        // Main sprite
        p.spawn((
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas { layout, index: 0 }),
                custom_size: Some(Vec2::new(display_size, display_size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            MerchantSprite,
        ));

        // Interaction prompt
        p.spawn((
            Text2d::new("[E] Shop"),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.9, 0.75, 0.4, 0.0)),
            Transform::from_xyz(0.0, display_size / 2.0 + 8.0, 1.0),
            MerchantPrompt,
        ));
    });
}

/// Animate the merchant sprite based on current MerchantAnimState.
fn merchant_anim_system(
    time: Res<Time>,
    mut merchant_q: Query<(&mut MerchantAnim, &mut Transform, &Children), With<MerchantNpc>>,
    mut sprite_q: Query<&mut Sprite, With<MerchantSprite>>,
) {
    let dt = time.delta_secs();
    let bob_period = std::f32::consts::TAU / 1.8;
    for (mut anim, mut tf, children) in &mut merchant_q {
        anim.timer += dt;
        anim.bob_t = (anim.bob_t + dt) % bob_period;

        // Idle bob: gentle up-down oscillation around fixed base_y (no drift)
        if anim.state == MerchantAnimState::Idle {
            let bob = (anim.bob_t * 1.8).sin() * 1.5;
            tf.translation.y = anim.base_y + bob;
        }

        // Timed states revert to Idle after duration
        if anim.state != MerchantAnimState::Idle && anim.timer >= anim.state.duration() {
            anim.state = MerchantAnimState::Idle;
            anim.timer = 0.0;
        }

        // Update sprite atlas frame on this merchant's own MerchantSprite child only
        let target_frame = anim.state.frame();
        for &child in children.iter() {
            if let Ok(mut sprite) = sprite_q.get_mut(child) {
                if let Some(ref mut atlas) = sprite.texture_atlas {
                    atlas.index = target_frame;
                }
            }
        }
    }
}

/// Detect player proximity to merchant and open shop on interaction.
fn merchant_interaction(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    player_q: Query<&Transform, (With<crate::player::Player>, Without<MerchantNpc>)>,
    mut merchant_q: Query<(&Transform, &mut MerchantNpc, &mut MerchantAnim), Without<crate::player::Player>>,
    mut prompt_q: Query<(&Parent, &mut TextColor), With<MerchantPrompt>>,
    mut commands: Commands,
    run: Res<RunData>,
    meta: Res<MetaProgression>,
    shop_state: Option<Res<ShopUiState>>,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    overlay_q: Query<Entity, With<ShopOverlayUI>>,
) {
    let Ok(p_tf) = player_q.get_single() else { return };
    if shop_state.as_ref().map_or(false, |s| s.active) { return; }

    let interact = keys.just_pressed(KeyCode::KeyE)
        || gamepads.iter().next().map_or(false, |g| g.just_pressed(GamepadButton::West));

    for (m_tf, mut merchant, mut anim) in &mut merchant_q {
        let dist = (p_tf.translation.xy() - m_tf.translation.xy()).length();
        let near = dist < 70.0;

        if near && interact {
            merchant.interacted = true;
            anim.state = MerchantAnimState::Talk;
            anim.timer = 0.0;

            let available: Vec<ShopEntry> = all_shop_entries()
                .into_iter()
                .filter(|e| e.unlock.is_met(run.current_floor, &meta))
                .collect();
            let selected = pick_shop_items(&available, run.current_floor);
            let count = selected.len();

            for e in &overlay_q {
                commands.entity(e).try_despawn_recursive();
            }

            spawn_shop_overlay(&mut commands, &font, &asset_server, &selected, run.gold);

            commands.insert_resource(ShopUiState {
                active: true,
                selected: 0,
                purchased: vec![false; count],
                items: selected,
                input_cooldown: 0.2,
            });
        }
    }

    for (parent, mut color) in &mut prompt_q {
        if let Ok((m_tf, _, _)) = merchant_q.get(parent.get()) {
            let dist = (p_tf.translation.xy() - m_tf.translation.xy()).length();
            color.0 = if dist < 70.0 && !shop_state.as_ref().map_or(false, |s| s.active) {
                Color::srgba(0.9, 0.75, 0.4, 1.0)
            } else {
                Color::srgba(0.9, 0.75, 0.4, 0.0)
            };
        }
    }
}

/// Spawn the card-based shop overlay UI.
fn spawn_shop_overlay(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    items: &[ShopEntry],
    gold: i32,
) {
    let f = font.0.clone();

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ShopOverlayUI,
        PlayingEntity,
    )).with_children(|root| {
        // ── Title ──
        root.spawn((
            Text::new("Händler"),
            TextFont { font: f.clone(), font_size: 20.0, ..default() },
            TextColor(Color::srgb(1.0, 0.82, 0.2)),
            Node {
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
        ));

        // ── Subtitle ──
        root.spawn((
            Text::new("Wähle ein Angebot"),
            TextFont { font: f.clone(), font_size: 11.0, ..default() },
            TextColor(Color::srgb(0.55, 0.52, 0.48)),
            Node {
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
        ));

        // ── Gold display ──
        root.spawn((
            Text::new(format!("Gold: {}", gold)),
            TextFont { font: f.clone(), font_size: 11.0, ..default() },
            TextColor(Color::srgb(0.95, 0.85, 0.4)),
            ShopGoldText,
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ));

        // ── Cards row ──
        root.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(16.0),
            align_items: AlignItems::Stretch,
            ..default()
        }).with_children(|row| {
            for (i, entry) in items.iter().enumerate() {
                let is_selected = i == 0;
                let card_bg = if is_selected {
                    Color::srgb(0.22, 0.15, 0.09)
                } else {
                    Color::srgb(0.15, 0.10, 0.07)
                };
                let border_color = if is_selected {
                    Color::srgb(1.0, 0.82, 0.2)
                } else {
                    Color::srgb(0.35, 0.25, 0.15)
                };

                row.spawn((
                    Node {
                        width: Val::Px(160.0),
                        min_height: Val::Px(210.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(if is_selected { 3.0 } else { 2.0 })),
                        row_gap: Val::Px(6.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    BackgroundColor(card_bg),
                    BorderColor(border_color),
                    BorderRadius::all(Val::Px(4.0)),
                    ShopCard(i),
                    ShopCardAnim {
                        card_index: i,
                        target_scale: if is_selected { 1.06 } else { 1.0 },
                        current_scale: if is_selected { 1.06 } else { 1.0 },
                    },
                )).with_children(|card| {
                    // ── Tier badge row (top-right aligned) ──
                    card.spawn(Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::FlexEnd,
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    }).with_children(|badge_row| {
                        badge_row.spawn((
                            Node {
                                width: Val::Px(10.0),
                                height: Val::Px(10.0),
                                border: UiRect::all(Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(entry.tier.dot_color()),
                            BorderRadius::all(Val::Px(5.0)),
                        ));
                    });

                    // ── Icon area (60x60) ──
                    if !entry.icon_path.is_empty() {
                        let img = asset_server.load(entry.icon_path);
                        card.spawn((
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                ..default()
                            },
                            ImageNode::new(img),
                        ));
                    } else {
                        card.spawn((
                            Node {
                                width: Val::Px(60.0),
                                height: Val::Px(60.0),
                                border: UiRect::all(Val::Px(0.0)),
                                ..default()
                            },
                            BackgroundColor(fallback_icon_color(&entry.effect)),
                            BorderRadius::all(Val::Px(4.0)),
                        ));
                    }

                    // ── Item name ──
                    card.spawn((
                        Text::new(entry.name),
                        TextFont { font: f.clone(), font_size: 13.0, ..default() },
                        TextColor(Color::WHITE),
                        ShopCardName(i),
                        Node {
                            ..default()
                        },
                    ));

                    // ── Description ──
                    card.spawn((
                        Text::new(entry.description),
                        TextFont { font: f.clone(), font_size: 9.5, ..default() },
                        TextColor(Color::srgb(0.7, 0.7, 0.65)),
                        Node {
                            flex_grow: 1.0,
                            ..default()
                        },
                    ));

                    // ── Divider ──
                    card.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(1.0),
                            margin: UiRect::vertical(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.28, 0.20, 0.12)),
                    ));

                    // ── Price row ──
                    let effective_cost = entry.cost; // initial display; updated each frame
                    card.spawn((
                        Text::new(format!("{} Gold", effective_cost)),
                        TextFont { font: f.clone(), font_size: 11.0, ..default() },
                        TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ShopCardPrice(i),
                    ));

                    // ── Purchased overlay (hidden initially — spawned as transparent) ──
                    card.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                        ShopCardPurchasedOverlay(i),
                    )).with_children(|overlay| {
                        overlay.spawn((
                            Text::new("✓ Gekauft"),
                            TextFont { font: f.clone(), font_size: 14.0, ..default() },
                            TextColor(Color::srgba(0.3, 0.9, 0.3, 0.0)),
                        ));
                    });
                });
            }
        });

        // ── Controls hint ──
        root.spawn((
            Text::new("A/D Navigieren  ENTER Kaufen  ESC Schließen"),
            TextFont { font: f.clone(), font_size: 8.0, ..default() },
            TextColor(Color::srgb(0.42, 0.38, 0.30)),
            Node {
                margin: UiRect::top(Val::Px(18.0)),
                ..default()
            },
        ));

        // ── Merchant dialogue text ──
        root.spawn((
            Text::new("Hmm... nimm, was du brauchst, Wanderer."),
            TextFont { font: f.clone(), font_size: 9.0, ..default() },
            TextColor(Color::srgb(0.55, 0.48, 0.35)),
            ShopMerchantText,
            Node {
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
        ));
    });
}

/// Navigate the shop with A/D (left/right) keys or DPad.
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
    let left = keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadLeft));
    let right = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::DPadRight));

    let count = state.items.len().min(4);
    if count == 0 { return; }

    if left {
        state.selected = if state.selected == 0 { count - 1 } else { state.selected - 1 };
        state.input_cooldown = 0.1;
    }
    if right {
        state.selected = (state.selected + 1) % count;
        state.input_cooldown = 0.1;
    }
}

/// Purchase the currently selected item.
fn shop_ui_purchase(
    mut shop_state: Option<ResMut<ShopUiState>>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut run: ResMut<RunData>,
    mut player_mut: Query<&mut crate::player::Player>,
    mut spell_slots_q: Query<&mut crate::spell::SpellSlots>,
    mut equipment_q: Query<&mut Equipment, With<crate::player::Player>>,
    mut recalc_ev: EventWriter<RecalcStats>,
    mut commands: Commands,
    font: Res<GameFont>,
    merchant_q: Query<&Transform, With<MerchantNpc>>,
    mut merchant_text_q: Query<&mut Text, With<ShopMerchantText>>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
    mut merchant_anim_q: Query<&mut MerchantAnim, With<MerchantNpc>>,
) {
    let Some(ref mut state) = shop_state else { return };
    if !state.active { return; }

    let gp = gamepads.iter().next();
    let buy = keys.just_pressed(KeyCode::KeyE) || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::West));

    if !buy { return; }

    let idx = state.selected;
    if idx >= state.items.len() { return; }

    if state.purchased[idx] {
        if let Ok(mut text) = merchant_text_q.get_single_mut() {
            **text = "Das hast du schon gekauft.".to_string();
        }
        return;
    }

    let item_name = state.items[idx].name.to_string();
    let item_effect = state.items[idx].effect.clone();

    let item_cost = if is_stat_upgrade(&item_effect) {
        let level = match &item_effect {
            ShopEffect::AttackUp  => run.stat_attack,
            ShopEffect::DefenseUp => run.stat_defense,
            ShopEffect::SpeedUp   => run.stat_speed,
            ShopEffect::MaxHpUp   => run.stat_hp,
            ShopEffect::ManaUp    => run.stat_mana,
            _ => 0,
        };
        stat_upgrade_cost(state.items[idx].cost, level)
    } else {
        state.items[idx].cost
    };

    if run.gold < item_cost {
        if let Ok(mut text) = merchant_text_q.get_single_mut() {
            **text = "Nicht genug Gold dafür...".to_string();
        }
        if let Ok(m_tf) = merchant_q.get_single() {
            spawn_feedback(&mut commands, m_tf.translation, "Nicht genug Gold!", Color::srgb(0.9, 0.3, 0.2), &font);
        }
        if let Ok(mut anim) = merchant_anim_q.get_single_mut() {
            anim.state = MerchantAnimState::Reject;
            anim.timer = 0.0;
        }
        return;
    }

    run.gold -= item_cost;
    apply_shop_effect(&item_effect, &mut player_mut, &mut spell_slots_q, &mut equipment_q, &mut recalc_ev);

    let is_stat = is_stat_upgrade(&item_effect);
    if !is_stat {
        state.purchased[idx] = true;
    }

    match &item_effect {
        ShopEffect::AttackUp  => run.stat_attack  += 1,
        ShopEffect::DefenseUp => run.stat_defense += 1,
        ShopEffect::SpeedUp   => run.stat_speed   += 1,
        ShopEffect::MaxHpUp   => run.stat_hp      += 1,
        ShopEffect::ManaUp    => run.stat_mana    += 1,
        _ => {}
    }
    ev_sfx.send(PlaySfxEvent(SfxType::ShopBuy));

    if let Ok(mut anim) = merchant_anim_q.get_single_mut() {
        anim.state = MerchantAnimState::Sell;
        anim.timer = 0.0;
    }

    if let Ok(m_tf) = merchant_q.get_single() {
        let msg = format!("{} gekauft!", item_name);
        spawn_feedback(&mut commands, m_tf.translation, &msg, Color::srgb(0.3, 0.9, 0.3), &font);
    }

    let responses = [
        "Eine weise Wahl.",
        "Das wird dir gut dienen.",
        "Kluge Entscheidung.",
        "Die Wurzeln sind einverstanden.",
        "Benutze es wohl dort unten.",
    ];
    let resp = responses[idx % responses.len()];
    if let Ok(mut text) = merchant_text_q.get_single_mut() {
        **text = resp.to_string();
    }
}

/// Apply a shop effect to the player.
fn apply_shop_effect(
    effect: &ShopEffect,
    player_mut: &mut Query<&mut crate::player::Player>,
    spell_slots_q: &mut Query<&mut crate::spell::SpellSlots>,
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

/// Update the shop overlay visuals each frame.
fn shop_ui_update_visuals(
    shop_state: Option<Res<ShopUiState>>,
    run: Res<RunData>,
    mut gold_text_q: Query<&mut Text, (With<ShopGoldText>, Without<ShopMerchantText>, Without<ShopCardName>, Without<ShopCardPrice>)>,
    mut card_q: Query<(&ShopCard, &mut BackgroundColor, &mut BorderColor), Without<ShopCardPurchasedOverlay>>,
    mut price_q: Query<(&ShopCardPrice, &mut Text, &mut TextColor), (Without<ShopGoldText>, Without<ShopMerchantText>, Without<ShopCardName>)>,
    mut name_q: Query<(&ShopCardName, &mut Text, &mut TextColor), (Without<ShopGoldText>, Without<ShopMerchantText>, Without<ShopCardPrice>)>,
    mut overlay_q: Query<(&ShopCardPurchasedOverlay, &mut BackgroundColor, &Children)>,
    mut text_color_q: Query<&mut TextColor, (Without<ShopCardName>, Without<ShopCardPrice>, Without<ShopGoldText>, Without<ShopMerchantText>)>,
) {
    let Some(ref state) = shop_state else { return };
    if !state.active { return; }

    // Update gold text
    if let Ok(mut text) = gold_text_q.get_single_mut() {
        **text = format!("Gold: {}", run.gold);
    }

    // Update card backgrounds and borders based on selection
    for (card, mut bg, mut border) in &mut card_q {
        let i = card.0;
        if i >= state.items.len() { continue; }
        let is_selected = i == state.selected;
        let is_purchased = state.purchased.get(i).copied().unwrap_or(false);

        if is_purchased {
            bg.0 = Color::srgba(0.08, 0.06, 0.04, 0.4);
            border.0 = if is_selected {
                Color::srgba(1.0, 0.82, 0.2, 0.4)
            } else {
                Color::srgba(0.35, 0.25, 0.15, 0.4)
            };
        } else if is_selected {
            bg.0 = Color::srgb(0.22, 0.15, 0.09);
            border.0 = Color::srgb(1.0, 0.82, 0.2);
        } else {
            bg.0 = Color::srgb(0.15, 0.10, 0.07);
            border.0 = Color::srgb(0.35, 0.25, 0.15);
        }
    }

    // Update card name texts
    for (name_comp, mut text, mut color) in &mut name_q {
        let i = name_comp.0;
        if i >= state.items.len() { continue; }
        let entry = &state.items[i];
        let is_purchased = state.purchased.get(i).copied().unwrap_or(false);

        if is_stat_upgrade(&entry.effect) {
            let level = match &entry.effect {
                ShopEffect::AttackUp  => run.stat_attack,
                ShopEffect::DefenseUp => run.stat_defense,
                ShopEffect::SpeedUp   => run.stat_speed,
                ShopEffect::MaxHpUp   => run.stat_hp,
                ShopEffect::ManaUp    => run.stat_mana,
                _ => 0,
            };
            **text = format!("{} Lv.{}", entry.name, level + 1);
        } else {
            **text = entry.name.to_string();
        }

        color.0 = if is_purchased {
            Color::srgba(1.0, 1.0, 1.0, 0.35)
        } else {
            Color::WHITE
        };
    }

    // Update card price texts
    for (price_comp, mut text, mut color) in &mut price_q {
        let i = price_comp.0;
        if i >= state.items.len() { continue; }
        let entry = &state.items[i];
        let is_purchased = state.purchased.get(i).copied().unwrap_or(false);

        let effective_cost = if is_stat_upgrade(&entry.effect) {
            let level = match &entry.effect {
                ShopEffect::AttackUp  => run.stat_attack,
                ShopEffect::DefenseUp => run.stat_defense,
                ShopEffect::SpeedUp   => run.stat_speed,
                ShopEffect::MaxHpUp   => run.stat_hp,
                ShopEffect::ManaUp    => run.stat_mana,
                _ => 0,
            };
            stat_upgrade_cost(entry.cost, level)
        } else {
            entry.cost
        };

        **text = format!("{} Gold", effective_cost);

        color.0 = if is_purchased {
            Color::srgba(1.0, 0.85, 0.3, 0.35)
        } else if run.gold >= effective_cost {
            Color::srgb(1.0, 0.85, 0.3)
        } else {
            Color::srgb(0.75, 0.3, 0.2)
        };
    }

    // Update purchased overlays
    for (overlay_comp, mut bg, children) in &mut overlay_q {
        let i = overlay_comp.0;
        let is_purchased = state.purchased.get(i).copied().unwrap_or(false);

        if is_purchased {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.55);
            // Make the "✓ Gekauft" child text visible
            for &child in children.iter() {
                if let Ok(mut tc) = text_color_q.get_mut(child) {
                    tc.0 = Color::srgba(0.3, 0.9, 0.3, 1.0);
                }
            }
        } else {
            bg.0 = Color::srgba(0.0, 0.0, 0.0, 0.0);
            for &child in children.iter() {
                if let Ok(mut tc) = text_color_q.get_mut(child) {
                    tc.0 = Color::srgba(0.3, 0.9, 0.3, 0.0);
                }
            }
        }
    }
}

/// Smoothly animate card scale based on selection state.
fn shop_card_anim_system(
    shop_state: Option<Res<ShopUiState>>,
    mut card_q: Query<(&mut ShopCardAnim, &mut Transform)>,
    time: Res<Time>,
) {
    let Some(ref state) = shop_state else { return };
    if !state.active { return; }

    let dt = time.delta_secs();
    for (mut anim, mut tf) in &mut card_q {
        anim.target_scale = if anim.card_index == state.selected { 1.06 } else { 1.0 };
        anim.current_scale += (anim.target_scale - anim.current_scale) * (8.0 * dt).min(1.0);
        let s = anim.current_scale;
        tf.scale = Vec3::new(s, s, 1.0);
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
        tf.translation.y += 30.0 * dt;
        if fb.timer <= 0.0 {
            commands.entity(entity).try_despawn_recursive();
        }
    }
}

/// Maximum items shown in a single shop visit (now 4 for card layout).
const MAX_SHOP_SLOTS: usize = 4;

/// Pick up to MAX_SHOP_SLOTS items from the available pool.
fn pick_shop_items(available: &[ShopEntry], _floor: i32) -> Vec<ShopEntry> {
    use std::collections::HashSet;

    if available.len() <= MAX_SHOP_SLOTS {
        return available.iter().map(|e| ShopEntry {
            name: e.name,
            description: e.description,
            icon_path: e.icon_path,
            cost: e.cost,
            effect: e.effect.clone(),
            tier: e.tier,
            unlock: e.unlock.clone(),
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
            name: e.name,
            description: e.description,
            icon_path: e.icon_path,
            cost: e.cost,
            effect: e.effect.clone(),
            tier: e.tier,
            unlock: e.unlock.clone(),
        }
    }).collect()
}
