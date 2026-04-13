use bevy::prelude::*;
use crate::{
    GameState, GameFont, RunData, PlayingEntity,
    player::Player,
    spell::{SpellSlots, SpellId},
    room::{RoomState, RoomType},
    equipment::{Equipment, ItemId},
};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // Load all equipment icon handles at startup.
        let asset_server = app.world().resource::<AssetServer>();
        let icon_base = "Icon/Free - Raven Fantasy Icons/Separated Files/16x16/";
        macro_rules! icon {
            ($name:expr) => { asset_server.load(format!("{}{}", icon_base, $name)) }
        }
        let icons = EquipmentIconAssets {
            rusty_sword:      icon!("fa577.png"),
            steel_blade:      icon!("fa578.png"),
            flame_edge:       icon!("fa579.png"),
            frost_fang:       icon!("fa581.png"),
            thunder_cleaver:  icon!("fa582.png"),
            leather_tunic:    icon!("fa583.png"),
            chain_mail:       icon!("fa584.png"),
            ember_plate:      icon!("fa585.png"),
            frost_guard:      icon!("fa588.png"),
            storm_armor:      icon!("fa586.png"),
            life_ring:        icon!("fa200.png"),
            mana_stone:       icon!("fa60.png"),
            gold_magnet:      icon!("fa590.png"),
            speed_boots:      icon!("fa75.png"),
            crit_charm:       icon!("fa100.png"),
            fire_amulet:      icon!("fa95.png"),
            ice_amulet:       icon!("fa90.png"),
            storm_amulet:     icon!("fa70.png"),
            vampire_fang:     icon!("fa45.png"),
            iron_will:        icon!("fa30.png"),
        };
        let spell_icons = SpellIconAssets {
            fireball:  asset_server.load(format!("{icon_base}fa3.png")),
            ice:       asset_server.load(format!("{icon_base}fa20.png")),
            lightning: asset_server.load(format!("{icon_base}fa70.png")),
            shield:    asset_server.load(format!("{icon_base}fa9.png")),
            locked:    asset_server.load(format!("{icon_base}fa50.png")),
        };
        let _ = asset_server;
        app.insert_resource(icons);
        app.insert_resource(spell_icons);

        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                (update_hud, update_minimap, health_bar_update, update_equipment_hud, update_spell_bar)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ─── Equipment icon assets ────────────────────────────────────────────────────

#[derive(Resource)]
pub struct EquipmentIconAssets {
    pub rusty_sword:     Handle<Image>,
    pub steel_blade:     Handle<Image>,
    pub flame_edge:      Handle<Image>,
    pub frost_fang:      Handle<Image>,
    pub thunder_cleaver: Handle<Image>,
    pub leather_tunic:   Handle<Image>,
    pub chain_mail:      Handle<Image>,
    pub ember_plate:     Handle<Image>,
    pub frost_guard:     Handle<Image>,
    pub storm_armor:     Handle<Image>,
    pub life_ring:       Handle<Image>,
    pub mana_stone:      Handle<Image>,
    pub gold_magnet:     Handle<Image>,
    pub speed_boots:     Handle<Image>,
    pub crit_charm:      Handle<Image>,
    pub fire_amulet:     Handle<Image>,
    pub ice_amulet:      Handle<Image>,
    pub storm_amulet:    Handle<Image>,
    pub vampire_fang:    Handle<Image>,
    pub iron_will:       Handle<Image>,
}

impl EquipmentIconAssets {
    pub fn handle_for(&self, id: ItemId) -> &Handle<Image> {
        match id {
            ItemId::RustySword      => &self.rusty_sword,
            ItemId::SteelBlade      => &self.steel_blade,
            ItemId::FlameEdge       => &self.flame_edge,
            ItemId::FrostFang       => &self.frost_fang,
            ItemId::ThunderCleaver  => &self.thunder_cleaver,
            ItemId::LeatherTunic    => &self.leather_tunic,
            ItemId::ChainMail       => &self.chain_mail,
            ItemId::EmberPlate      => &self.ember_plate,
            ItemId::FrostGuard      => &self.frost_guard,
            ItemId::StormArmor      => &self.storm_armor,
            ItemId::LifeRing        => &self.life_ring,
            ItemId::ManaStone       => &self.mana_stone,
            ItemId::GoldMagnet      => &self.gold_magnet,
            ItemId::SpeedBoots      => &self.speed_boots,
            ItemId::CritCharm       => &self.crit_charm,
            ItemId::FireAmulet      => &self.fire_amulet,
            ItemId::IceAmulet       => &self.ice_amulet,
            ItemId::StormAmulet     => &self.storm_amulet,
            ItemId::VampireFang     => &self.vampire_fang,
            ItemId::IronWill        => &self.iron_will,
        }
    }
}

// ─── Spell icon assets ────────────────────────────────────────────────────────

#[derive(Resource)]
pub struct SpellIconAssets {
    pub fireball:  Handle<Image>,
    pub ice:       Handle<Image>,
    pub lightning: Handle<Image>,
    pub shield:    Handle<Image>,
    /// Shown for slots that have not been purchased yet.
    pub locked:    Handle<Image>,
}

// ─── Spell bar components ─────────────────────────────────────────────────────

/// Marks one of the four spell slot boxes (index 0-3).
#[derive(Component)]
struct SpellSlotUi {
    index: usize,
}

/// The icon image node inside a spell slot.
#[derive(Component)]
struct SpellSlotIcon {
    index: usize,
}

/// Dark cooldown overlay that grows from the bottom up while a spell is on cooldown.
#[derive(Component)]
struct SpellCooldownOverlay {
    index: usize,
}

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Component)]
struct HudRoot;

/// Marker for the entire equipment slot row container.
#[derive(Component)]
struct EquipmentSlotsRoot;

/// Identifies which slot index (0=Weapon, 1=Armor, 2=Relic, 3=Charm) this node represents.
#[derive(Component)]
struct EquipmentSlotIcon {
    slot_index: usize,
}

#[derive(Component)]
struct ManaText;

#[derive(Component)]
struct GoldText;

#[derive(Component)]
struct FloorText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct EnemyText;

#[derive(Component)]
struct MinimapRoot;

#[derive(Component)]
struct MinimapCell {
    room_index: usize,
}

// ── Module 6: Health bar components ──────────────────────────────

/// Red foreground bar (jumps instantly to current HP).
#[derive(Component)]
struct HealthBarFill;

/// Yellow/white trailing bar (catches up slowly for delayed damage effect).
#[derive(Component)]
struct HealthBarDelayed {
    displayed_ratio: f32,
}

/// HP text label on top of the bar.
#[derive(Component)]
struct HealthBarLabel;

const HEALTH_BAR_W: f32 = 120.0;
const HEALTH_BAR_H: f32 = 14.0;

const SLOT_SIZE: f32 = 40.0;
const ICON_SIZE: f32 = 28.0;
const SLOT_GAP:  f32 = 6.0;

fn icon_handle_for_spell(spell: SpellId, icons: &SpellIconAssets) -> Handle<Image> {
    match spell {
        SpellId::Fireball  => icons.fireball.clone(),
        SpellId::IceShards => icons.ice.clone(),
        SpellId::Lightning => icons.lightning.clone(),
        SpellId::Shield    => icons.shield.clone(),
    }
}

fn setup_hud(
    mut commands: Commands,
    font: Res<GameFont>,
    icons: Res<SpellIconAssets>,
    existing: Query<&HudRoot>,
) {
    if existing.iter().next().is_some() { return; }
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            HudRoot,
            PlayingEntity,
        ))
        .with_children(|parent| {
            // Top row
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    // Left: Health bar + Mana + Enemy count
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|col| {
                        let f = font.0.clone();

                        // ── Health bar container (border + bars + label) ──
                        col.spawn(Node {
                            width: Val::Px(HEALTH_BAR_W + 4.0),
                            height: Val::Px(HEALTH_BAR_H + 4.0),
                            ..default()
                        })
                        .with_children(|bar_container| {
                            // Dark border/background
                            bar_container.spawn((
                                Node {
                                    width: Val::Px(HEALTH_BAR_W + 4.0),
                                    height: Val::Px(HEALTH_BAR_H + 4.0),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(0.0),
                                    top: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                            ));
                            // Delayed damage bar (yellow, behind red)
                            bar_container.spawn((
                                Node {
                                    width: Val::Px(HEALTH_BAR_W),
                                    height: Val::Px(HEALTH_BAR_H),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(2.0),
                                    top: Val::Px(2.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.9, 0.75, 0.1)),
                                HealthBarDelayed { displayed_ratio: 1.0 },
                            ));
                            // Current HP bar (red, in front)
                            bar_container.spawn((
                                Node {
                                    width: Val::Px(HEALTH_BAR_W),
                                    height: Val::Px(HEALTH_BAR_H),
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(2.0),
                                    top: Val::Px(2.0),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.85, 0.2, 0.15)),
                                HealthBarFill,
                            )).with_children(|fill| {
                                // HP text on top of bar
                                fill.spawn((
                                    Text::new("10/10"),
                                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                                    TextColor(Color::WHITE),
                                    HealthBarLabel,
                                ));
                            });
                        });

                        col.spawn((
                            Text::new("MANA: 100/100"),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(Color::srgb(0.7, 0.45, 0.9)),
                            ManaText,
                        ));
                        col.spawn((
                            Text::new(""),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(Color::srgb(0.9, 0.5, 0.2)),
                            EnemyText,
                        ));
                    });

                    // Right: Score + Gold + Floor
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::End,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|col| {
                        let f = font.0.clone();
                        col.spawn((
                            Text::new("000000"),
                            TextFont { font: f.clone(), font_size: 10.0, ..default() },
                            TextColor(Color::WHITE),
                            ScoreText,
                        ));
                        col.spawn((
                            Text::new("Gold: 0"),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(Color::srgb(0.9, 0.8, 0.3)),
                            GoldText,
                        ));
                        col.spawn((
                            Text::new("Floor 1 - Room 1"),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(Color::srgb(0.65, 0.55, 0.4)),
                            FloorText,
                        ));
                    });
                });

            // ── Bottom-left: Spell icon bar ───────────────────────
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    left: Val::Px(12.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(SLOT_GAP),
                    ..default()
                },
            )).with_children(|bar| {
                let key_labels = ["1", "2", "3", "4"];
                for i in 0..4usize {
                    bar.spawn((
                        Node {
                            width: Val::Px(SLOT_SIZE),
                            height: Val::Px(SLOT_SIZE),
                            border: UiRect::all(Val::Px(1.0)),
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.08, 0.06, 0.8)),
                        BorderColor(Color::srgba(0.3, 0.25, 0.2, 0.8)),
                        SpellSlotUi { index: i },
                    )).with_children(|slot| {
                        // (a) Key number label — top-left corner
                        slot.spawn((
                            Text::new(key_labels[i]),
                            TextFont { font: font.0.clone(), font_size: 6.0, ..default() },
                            TextColor(Color::srgba(0.9, 0.85, 0.7, 0.9)),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(2.0),
                                left: Val::Px(3.0),
                                ..default()
                            },
                        ));

                        // (b) Centered spell icon
                        slot.spawn((
                            ImageNode::new(icons.locked.clone()),
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(ICON_SIZE),
                                height: Val::Px(ICON_SIZE),
                                left: Val::Px((SLOT_SIZE - ICON_SIZE) / 2.0),
                                top: Val::Px((SLOT_SIZE - ICON_SIZE) / 2.0),
                                ..default()
                            },
                            SpellSlotIcon { index: i },
                        ));

                        // (c) Cooldown overlay — dark rect that shrinks from top as cooldown drains
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(0.0),
                                left: Val::Px(0.0),
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(0.0), // driven by update_spell_bar
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
                            SpellCooldownOverlay { index: i },
                        ));
                    });
                }
            });

            // Equipment slots row (bottom-right, above minimap)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(36.0),
                    right: Val::Px(10.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(2.0),
                    align_items: AlignItems::Center,
                    ..default()
                },
                EquipmentSlotsRoot,
            )).with_children(|row| {
                for slot_index in 0..4 {
                    // Outer box (dark background border)
                    row.spawn((
                        Node {
                            width: Val::Px(24.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.8)),
                        BorderColor(Color::srgba(0.3, 0.3, 0.35, 0.9)),
                    )).with_children(|cell| {
                        // Inner icon image node
                        cell.spawn((
                            ImageNode::default(),
                            Node {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                ..default()
                            },
                            EquipmentSlotIcon { slot_index },
                        ));
                    });
                }
            });

            // Inventory hint label (above equipment slots)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(62.0),
                    right: Val::Px(10.0),
                    ..default()
                },
                Text::new("[I] Inventory"),
                TextFont { font: font.0.clone(), font_size: 5.5, ..default() },
                TextColor(Color::srgba(0.6, 0.55, 0.4, 0.7)),
            ));

            // Minimap container (bottom-right)
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    right: Val::Px(10.0),
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(3.0),
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                MinimapRoot,
            ));
        });
}

fn update_hud(
    run: Res<RunData>,
    room_state: Res<RoomState>,
    player_q: Query<&Player>,
    mut score_q:  Query<&mut Text, (With<ScoreText>,  Without<ManaText>, Without<GoldText>, Without<FloorText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut mana_q:   Query<&mut Text, (With<ManaText>,   Without<ScoreText>, Without<GoldText>, Without<FloorText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut gold_q:   Query<&mut Text, (With<GoldText>,   Without<ScoreText>, Without<ManaText>, Without<FloorText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut floor_q:  Query<&mut Text, (With<FloorText>,  Without<ScoreText>, Without<ManaText>, Without<GoldText>,  Without<EnemyText>, Without<HealthBarLabel>)>,
    mut enemy_q:  Query<&mut Text, (With<EnemyText>,  Without<ScoreText>, Without<ManaText>, Without<GoldText>,  Without<FloorText>, Without<HealthBarLabel>)>,
) {
    let player = player_q.get_single().ok();

    if let Ok(mut text) = score_q.get_single_mut() {
        **text = format!("{:06}", run.score);
    }

    if let Ok(mut text) = mana_q.get_single_mut() {
        if let Some(p) = player {
            **text = format!("MANA: {}/{}", p.mana as i32, p.max_mana as i32);
        }
    }

    if let Ok(mut text) = gold_q.get_single_mut() {
        **text = format!("Gold: {}", run.gold);
    }

    if let Ok(mut text) = floor_q.get_single_mut() {
        **text = format!("Floor {} - Room {} ({:?})", room_state.floor, room_state.room_index + 1, room_state.current_type);
    }

    if let Ok(mut text) = enemy_q.get_single_mut() {
        if run.enemies_alive > 0 {
            **text = format!("Enemies: {}", run.enemies_alive);
        } else {
            **text = String::new();
        }
    }
}

// ── Spell bar update system ───────────────────────────────────────

fn update_spell_bar(
    slots_q: Query<&SpellSlots>,
    icons: Res<SpellIconAssets>,
    mut slot_q: Query<(&SpellSlotUi, &mut BorderColor)>,
    mut icon_q: Query<(&SpellSlotIcon, &mut ImageNode)>,
    mut overlay_q: Query<(&SpellCooldownOverlay, &mut Node)>,
) {
    let Ok(slots) = slots_q.get_single() else { return };

    // Border color: gold when ready, dim orange on cooldown, dark grey when locked
    for (slot_ui, mut border) in &mut slot_q {
        let i = slot_ui.index;
        *border = match slots.slots[i] {
            Some(_) if slots.cooldowns[i] <= 0.0 => BorderColor(Color::srgb(0.85, 0.7, 0.2)),
            Some(_)                               => BorderColor(Color::srgba(0.55, 0.35, 0.1, 0.9)),
            None                                  => BorderColor(Color::srgba(0.25, 0.2, 0.18, 0.7)),
        };
    }

    // Icon image: spell icon if unlocked, locked placeholder otherwise
    for (slot_icon, mut img) in &mut icon_q {
        let i = slot_icon.index;
        let new_handle = match slots.slots[i] {
            Some(spell) => icon_handle_for_spell(spell, &icons),
            None        => icons.locked.clone(),
        };
        if img.image != new_handle {
            img.image = new_handle;
        }
    }

    // Cooldown overlay: height = (remaining / max) * SLOT_SIZE, anchored at bottom
    for (overlay, mut node) in &mut overlay_q {
        let i = overlay.index;
        let cd = slots.cooldowns[i];
        let height_px = if cd <= 0.0 {
            0.0
        } else {
            let max_cd = match slots.slots[i] {
                Some(spell) => spell.cooldown(),
                None        => 1.0,
            };
            (cd / max_cd).clamp(0.0, 1.0) * SLOT_SIZE
        };
        node.height = Val::Px(height_px);
    }
}

// ── Module 6: Health bar update system ───────────────────────────

fn health_bar_update(
    player_q: Query<&Player>,
    mut fill_q: Query<(&mut Node, &mut BackgroundColor), (With<HealthBarFill>, Without<HealthBarDelayed>)>,
    mut delayed_q: Query<(&mut Node, &mut HealthBarDelayed), Without<HealthBarFill>>,
    mut label_q: Query<&mut Text, With<HealthBarLabel>>,
    time: Res<Time>,
) {
    let Ok(player) = player_q.get_single() else { return };
    let ratio = (player.health as f32 / player.max_health as f32).clamp(0.0, 1.0);

    // Update red fill bar (instant)
    if let Ok((mut node, mut bg)) = fill_q.get_single_mut() {
        node.width = Val::Px(HEALTH_BAR_W * ratio);
        // Color shifts: green > yellow > red
        let color = if ratio > 0.6 {
            Color::srgb(0.2, 0.75, 0.25)
        } else if ratio > 0.3 {
            Color::srgb(0.9, 0.75, 0.1)
        } else {
            Color::srgb(0.85, 0.2, 0.15)
        };
        *bg = BackgroundColor(color);
    }

    // Update delayed bar (catches up slowly)
    let dt = time.delta_secs();
    if let Ok((mut node, mut delayed)) = delayed_q.get_single_mut() {
        if delayed.displayed_ratio > ratio {
            // Slowly shrink toward actual ratio
            delayed.displayed_ratio -= dt * 0.8;
            delayed.displayed_ratio = delayed.displayed_ratio.max(ratio);
        } else {
            // Snap up if healed
            delayed.displayed_ratio = ratio;
        }
        node.width = Val::Px(HEALTH_BAR_W * delayed.displayed_ratio);
    }

    // Update text label
    if let Ok(mut text) = label_q.get_single_mut() {
        **text = format!("{}/{}", player.health, player.max_health);
    }
}

// ── Minimap ──────────────────────────────────────────────────────

// ── Equipment HUD ─────────────────────────────────────────────────

fn update_equipment_hud(
    player_q: Query<(Entity, &Player)>,
    equip_q: Query<&Equipment>,
    mut slot_q: Query<(&EquipmentSlotIcon, &mut ImageNode)>,
    icon_assets: Res<EquipmentIconAssets>,
) {
    let Ok((player_entity, _)) = player_q.get_single() else { return };
    let Ok(equip) = equip_q.get(player_entity) else { return };

    // Collect the 4 slot values in order: Weapon, Armor, Relic, Charm.
    let slots: [Option<ItemId>; 4] = [
        equip.weapon,
        equip.armor,
        equip.relic,
        equip.charm,
    ];

    for (slot_icon, mut image_node) in &mut slot_q {
        let item = slots[slot_icon.slot_index];
        match item {
            Some(id) => {
                image_node.image = icon_assets.handle_for(id).clone();
                image_node.color = Color::WHITE;
            }
            None => {
                // Empty slot — show no image (default empty handle is fine).
                image_node.image = Handle::default();
                image_node.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
            }
        }
    }
}

fn minimap_room_color(room_type: RoomType) -> Color {
    match room_type {
        RoomType::Start    => Color::srgb(0.4, 0.6, 0.4),
        RoomType::Combat   => Color::srgb(0.7, 0.3, 0.25),
        RoomType::Treasure => Color::srgb(0.9, 0.75, 0.2),
        RoomType::Shop     => Color::srgb(0.3, 0.5, 0.8),
        RoomType::Boss     => Color::srgb(0.8, 0.15, 0.15),
        RoomType::Altar    => Color::srgb(0.6, 0.3, 0.7),
    }
}

fn update_minimap(
    mut commands: Commands,
    room_state: Res<RoomState>,
    minimap_root_q: Query<Entity, With<MinimapRoot>>,
    mut cell_q: Query<(&MinimapCell, &mut BackgroundColor, &mut Node), Without<MinimapRoot>>,
    mut last_floor: Local<i32>,
    mut last_room_count: Local<usize>,
) {
    let Ok(root_entity) = minimap_root_q.get_single() else { return };
    let layout = &room_state.floor_layout;

    if *last_floor != room_state.floor || *last_room_count != layout.len() {
        *last_floor = room_state.floor;
        *last_room_count = layout.len();

        commands.entity(root_entity).despawn_descendants();

        commands.entity(root_entity).with_children(|parent| {
            for (i, &room_type) in layout.iter().enumerate() {
                let color = minimap_room_color(room_type);
                let is_current = i == room_state.room_index;
                let size = if is_current { 12.0 } else { 8.0 };

                parent.spawn((
                    Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        border: if is_current {
                            UiRect::all(Val::Px(1.0))
                        } else {
                            UiRect::ZERO
                        },
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderColor(if is_current { Color::WHITE } else { Color::NONE }),
                    MinimapCell { room_index: i },
                ));
            }
        });
        return;
    }

    for (cell, mut bg, mut node) in &mut cell_q {
        let is_current = cell.room_index == room_state.room_index;
        let room_type = layout.get(cell.room_index).copied().unwrap_or(RoomType::Combat);

        let base_color = minimap_room_color(room_type);
        if is_current {
            *bg = BackgroundColor(Color::WHITE);
            node.width = Val::Px(12.0);
            node.height = Val::Px(12.0);
            node.border = UiRect::all(Val::Px(1.0));
        } else if cell.room_index < room_state.room_index {
            let Color::Srgba(c) = base_color else { *bg = BackgroundColor(base_color); continue; };
            *bg = BackgroundColor(Color::srgba(c.red * 0.5, c.green * 0.5, c.blue * 0.5, 0.6));
            node.width = Val::Px(8.0);
            node.height = Val::Px(8.0);
            node.border = UiRect::ZERO;
        } else {
            *bg = BackgroundColor(base_color);
            node.width = Val::Px(8.0);
            node.height = Val::Px(8.0);
            node.border = UiRect::ZERO;
        }
    }
}
