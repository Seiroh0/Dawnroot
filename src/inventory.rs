use bevy::prelude::*;
use crate::{
    GameState, GameFont, PlayingEntity,
    player::Player,
    equipment::{Equipment, PlayerStats, ItemId, ItemSlot, ItemSet},
    relic::RelicInventory,
    hud::EquipmentIconAssets,
};

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                toggle_inventory,
                spawn_inventory_ui,
                close_inventory,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ---------------------------------------------------------------------------
// Resource / Components
// ---------------------------------------------------------------------------

/// When present, the inventory overlay is shown and player movement is blocked.
#[derive(Resource)]
pub struct InventoryOpen;

#[derive(Component)]
struct InventoryUI;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn toggle_inventory(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    inv: Option<Res<InventoryOpen>>,
    ui_q: Query<Entity, With<InventoryUI>>,
    // Don't open inventory while shop or relic choice is active
    shop: Option<Res<crate::shop::ShopUiState>>,
    relic: Res<crate::relic::RelicChoiceState>,
) {
    if !keys.just_pressed(KeyCode::Tab) && !keys.just_pressed(KeyCode::KeyI) {
        return;
    }
    // Block if shop overlay active
    if shop.map_or(false, |s| s.active) { return; }
    // Block if relic choice active
    if relic.active { return; }

    if inv.is_some() {
        // Close
        commands.remove_resource::<InventoryOpen>();
        for e in &ui_q {
            commands.entity(e).despawn_recursive();
        }
    } else {
        // Open
        commands.insert_resource(InventoryOpen);
    }
}

fn spawn_inventory_ui(
    mut commands: Commands,
    inv: Option<Res<InventoryOpen>>,
    existing: Query<&InventoryUI>,
    font: Res<GameFont>,
    player_q: Query<(Entity, &Player)>,
    equip_q: Query<&Equipment>,
    stats: Res<PlayerStats>,
    relic_inv: Res<RelicInventory>,
    icon_assets: Res<EquipmentIconAssets>,
) {
    if inv.is_none() { return; }
    if existing.iter().next().is_some() { return; }

    let Ok((player_entity, player)) = player_q.get_single() else { return };
    let equip = equip_q.get(player_entity).ok();

    let f = font.0.clone();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.01, 0.04, 0.92)),
            GlobalZIndex(140),
            InventoryUI,
            PlayingEntity,
        ))
        .with_children(|root| {
            // ── Title ──
            root.spawn((
                Text::new("INVENTORY"),
                TextFont { font: f.clone(), font_size: 14.0, ..default() },
                TextColor(Color::srgb(0.95, 0.85, 0.4)),
                Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
            ));

            // ── Main content: two columns ──
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(32.0),
                align_items: AlignItems::FlexStart,
                ..default()
            }).with_children(|cols| {
                // ═══ LEFT COLUMN: Equipment slots ═══
                cols.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    width: Val::Px(260.0),
                    ..default()
                }).with_children(|left| {
                    // Section header
                    left.spawn((
                        Text::new("Equipment"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.7, 0.5)),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));

                    let slot_order = [
                        (ItemSlot::Weapon, "Weapon"),
                        (ItemSlot::Armor, "Armor"),
                        (ItemSlot::Relic, "Relic"),
                        (ItemSlot::Charm, "Charm"),
                    ];

                    for (slot, slot_label) in &slot_order {
                        let item = equip.and_then(|eq| *eq.get_slot(*slot));
                        spawn_equipment_row(left, &f, slot_label, item, &icon_assets);
                    }

                    // ── Set Bonuses ──
                    if !stats.active_set_bonuses.is_empty() {
                        left.spawn((
                            Text::new("Set Bonuses"),
                            TextFont { font: f.clone(), font_size: 10.0, ..default() },
                            TextColor(Color::srgb(0.8, 0.7, 0.5)),
                            Node { margin: UiRect { top: Val::Px(12.0), bottom: Val::Px(4.0), ..default() }, ..default() },
                        ));

                        for (set, count) in &stats.active_set_bonuses {
                            let set_name = match set {
                                ItemSet::Fire => "Fire",
                                ItemSet::Ice => "Ice",
                                ItemSet::Storm => "Storm",
                                ItemSet::None => continue,
                            };
                            let bonus_text = if *count >= 3 {
                                format!("{}/3 {} Set (FULL)", count, set_name)
                            } else {
                                format!("{}/3 {} Set", count, set_name)
                            };
                            let bonus_desc = match (set, *count >= 3) {
                                (ItemSet::Fire, false) => "+15% ATK",
                                (ItemSet::Fire, true) => "+25% ATK, +2 ATK, 5% Lifesteal",
                                (ItemSet::Ice, false) => "+15% DEF, +2 HP",
                                (ItemSet::Ice, true) => "+25% DEF, +2 DEF, +5 HP",
                                (ItemSet::Storm, false) => "+15% Speed, +10% Crit",
                                (ItemSet::Storm, true) => "+25% Speed, +20% Crit, +10% ATK",
                                _ => "",
                            };

                            left.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(2.0),
                                ..default()
                            }).with_children(|bonus_row| {
                                let set_color = match set {
                                    ItemSet::Fire => Color::srgb(1.0, 0.5, 0.2),
                                    ItemSet::Ice => Color::srgb(0.4, 0.8, 1.0),
                                    ItemSet::Storm => Color::srgb(0.6, 0.4, 1.0),
                                    _ => Color::WHITE,
                                };
                                bonus_row.spawn((
                                    Text::new(bonus_text),
                                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                                    TextColor(Color::srgb(0.3, 0.9, 0.3)),
                                ));
                                bonus_row.spawn((
                                    Text::new(bonus_desc),
                                    TextFont { font: f.clone(), font_size: 6.0, ..default() },
                                    TextColor(set_color),
                                ));
                            });
                        }
                    }
                });

                // ═══ RIGHT COLUMN: Stats + Relics ═══
                cols.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    width: Val::Px(200.0),
                    ..default()
                }).with_children(|right| {
                    // ── Player Stats ──
                    right.spawn((
                        Text::new("Stats"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(Color::srgb(0.8, 0.7, 0.5)),
                        Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                    ));

                    let stat_lines = [
                        (format!("HP: {}/{}", player.health, player.max_health), Color::srgb(0.85, 0.3, 0.2)),
                        (format!("Mana: {}/{}", player.mana as i32, player.max_mana as i32), Color::srgb(0.5, 0.4, 0.9)),
                        (format!("ATK: {}", stats.attack), Color::srgb(0.9, 0.6, 0.2)),
                        (format!("DEF: {}", stats.defense), Color::srgb(0.4, 0.6, 0.9)),
                        (format!("Speed: {:.0}%", stats.speed_mult * 100.0), Color::srgb(0.3, 0.8, 0.4)),
                        (format!("Crit: {:.0}%", stats.crit_chance * 100.0), Color::srgb(0.9, 0.8, 0.2)),
                    ];

                    for (text, color) in &stat_lines {
                        right.spawn((
                            Text::new(text.clone()),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(*color),
                        ));
                    }

                    // ── Relics ──
                    if !relic_inv.relics.is_empty() {
                        right.spawn((
                            Text::new("Relics"),
                            TextFont { font: f.clone(), font_size: 10.0, ..default() },
                            TextColor(Color::srgb(0.8, 0.7, 0.5)),
                            Node { margin: UiRect { top: Val::Px(12.0), bottom: Val::Px(4.0), ..default() }, ..default() },
                        ));

                        for relic in &relic_inv.relics {
                            right.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(6.0),
                                align_items: AlignItems::Center,
                                ..default()
                            }).with_children(|row| {
                                // Colored dot
                                row.spawn((
                                    Node {
                                        width: Val::Px(6.0),
                                        height: Val::Px(6.0),
                                        ..default()
                                    },
                                    BackgroundColor(relic.color()),
                                ));
                                // Name + description
                                row.spawn((
                                    Text::new(format!("{} - {}", relic.name(), relic.description())),
                                    TextFont { font: f.clone(), font_size: 6.0, ..default() },
                                    TextColor(Color::srgb(0.7, 0.65, 0.55)),
                                ));
                            });
                        }
                    }
                });
            });

            // ── Controls hint ──
            root.spawn((
                Text::new("[TAB/I] Close"),
                TextFont { font: f.clone(), font_size: 7.0, ..default() },
                TextColor(Color::srgb(0.5, 0.45, 0.35)),
                Node { margin: UiRect::top(Val::Px(20.0)), ..default() },
            ));
        });
}

fn close_inventory(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    inv: Option<Res<InventoryOpen>>,
    ui_q: Query<Entity, With<InventoryUI>>,
) {
    if inv.is_none() { return; }
    if keys.just_pressed(KeyCode::Escape) {
        commands.remove_resource::<InventoryOpen>();
        for e in &ui_q {
            commands.entity(e).despawn_recursive();
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn spawn_equipment_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    slot_label: &str,
    item: Option<ItemId>,
    icon_assets: &EquipmentIconAssets,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(8.0),
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(4.0)),
        ..default()
    }).with_children(|row| {
        // Slot label
        row.spawn((
            Text::new(format!("{}:", slot_label)),
            TextFont { font: font.clone(), font_size: 7.0, ..default() },
            TextColor(Color::srgb(0.6, 0.55, 0.45)),
            Node { width: Val::Px(52.0), ..default() },
        ));

        match item {
            Some(id) => {
                let data = id.data();
                // Icon
                row.spawn((
                    ImageNode::new(icon_assets.handle_for(id).clone()),
                    Node {
                        width: Val::Px(20.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                ));
                // Name + description column
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(1.0),
                    ..default()
                }).with_children(|info| {
                    // Item name with set color
                    let name_color = match data.set {
                        crate::equipment::ItemSet::Fire => Color::srgb(1.0, 0.5, 0.2),
                        crate::equipment::ItemSet::Ice => Color::srgb(0.4, 0.8, 1.0),
                        crate::equipment::ItemSet::Storm => Color::srgb(0.6, 0.4, 1.0),
                        crate::equipment::ItemSet::None => Color::srgb(0.85, 0.8, 0.7),
                    };
                    info.spawn((
                        Text::new(data.name),
                        TextFont { font: font.clone(), font_size: 7.0, ..default() },
                        TextColor(name_color),
                    ));
                    // Description
                    info.spawn((
                        Text::new(data.description),
                        TextFont { font: font.clone(), font_size: 6.0, ..default() },
                        TextColor(Color::srgb(0.55, 0.5, 0.4)),
                    ));
                    // Set membership tag
                    if data.set != crate::equipment::ItemSet::None {
                        let set_name = match data.set {
                            crate::equipment::ItemSet::Fire => "Fire Set",
                            crate::equipment::ItemSet::Ice => "Ice Set",
                            crate::equipment::ItemSet::Storm => "Storm Set",
                            _ => "",
                        };
                        info.spawn((
                            Text::new(format!("[{}]", set_name)),
                            TextFont { font: font.clone(), font_size: 5.0, ..default() },
                            TextColor(Color::srgb(0.4, 0.7, 0.4)),
                        ));
                    }
                });
            }
            None => {
                row.spawn((
                    Text::new("- Empty -"),
                    TextFont { font: font.clone(), font_size: 7.0, ..default() },
                    TextColor(Color::srgb(0.4, 0.35, 0.3)),
                ));
            }
        }
    });
}
