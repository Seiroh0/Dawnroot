use bevy::prelude::*;
use crate::{
    GameState, GameFont, PlayingEntity, RunData,
    player::Player,
    equipment::{Equipment, PlayerStats, ItemId, ItemSlot, ItemSet},
    relic::{RelicInventory, RelicIconAssets},
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
    relic_icons: Res<RelicIconAssets>,
    run: Res<RunData>,
) {
    if inv.is_none() { return; }
    if existing.iter().next().is_some() { return; }

    let Ok((player_entity, player)) = player_q.get_single() else { return };
    let equip = equip_q.get(player_entity).ok();

    let f = font.0.clone();

    // Colors
    let col_header  = Color::srgb(0.95, 0.82, 0.35);
    let col_dim     = Color::srgba(0.55, 0.5, 0.4, 0.85);
    let panel_bg    = Color::srgba(0.06, 0.04, 0.10, 0.8);
    let panel_border = Color::srgba(0.3, 0.2, 0.5, 0.4);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.02, 0.06, 0.93)),
            GlobalZIndex(140),
            InventoryUI,
            PlayingEntity,
        ))
        .with_children(|root| {
            // ── Title bar ──
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                width: Val::Percent(100.0),
                margin: UiRect::bottom(Val::Px(10.0)),
                ..default()
            }).with_children(|bar| {
                bar.spawn((
                    Text::new("INVENTORY"),
                    TextFont { font: f.clone(), font_size: 14.0, ..default() },
                    TextColor(col_header),
                ));
                bar.spawn((
                    Text::new("[TAB / I] Close"),
                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                    TextColor(col_dim),
                ));
            });

            // ── Three-column content row ──
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                align_items: AlignItems::FlexStart,
                width: Val::Percent(100.0),
                ..default()
            }).with_children(|cols| {

                // ═══════════════════════════════════════
                // LEFT COLUMN: Equipment + Set Bonuses
                // ═══════════════════════════════════════
                cols.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        width: Val::Px(250.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor(panel_border),
                )).with_children(|left| {
                    // Column header
                    left.spawn((
                        Text::new("EQUIPMENT"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(col_header),
                        Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
                    ));

                    let slot_order = [
                        (ItemSlot::Weapon, "Weapon"),
                        (ItemSlot::Armor,  "Armor"),
                        (ItemSlot::Relic,  "Relic"),
                        (ItemSlot::Charm,  "Charm"),
                    ];

                    for (slot, slot_label) in &slot_order {
                        let item = equip.and_then(|eq| *eq.get_slot(*slot));
                        spawn_equipment_row(left, &f, slot_label, item, &icon_assets);
                    }

                    // ── Set Bonuses ──
                    left.spawn((
                        Text::new("SET BONUSES"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(col_header),
                        Node { margin: UiRect { top: Val::Px(14.0), bottom: Val::Px(6.0), ..default() }, ..default() },
                    ));

                    if stats.active_set_bonuses.is_empty() {
                        left.spawn((
                            Text::new("No active set bonuses"),
                            TextFont { font: f.clone(), font_size: 6.5, ..default() },
                            TextColor(col_dim),
                        ));
                    } else {
                        for (set, count) in &stats.active_set_bonuses {
                            let set_name = match set {
                                ItemSet::Fire  => "Fire",
                                ItemSet::Ice   => "Ice",
                                ItemSet::Storm => "Storm",
                                ItemSet::None  => continue,
                            };
                            let set_color = match set {
                                ItemSet::Fire  => Color::srgb(1.0, 0.5, 0.2),
                                ItemSet::Ice   => Color::srgb(0.4, 0.8, 1.0),
                                ItemSet::Storm => Color::srgb(0.6, 0.4, 1.0),
                                _ => Color::WHITE,
                            };
                            let bonus_desc = match (set, *count >= 3) {
                                (ItemSet::Fire,  false) => "+15% ATK",
                                (ItemSet::Fire,  true)  => "+25% ATK, +2 ATK, 5% Lifesteal",
                                (ItemSet::Ice,   false) => "+15% DEF, +2 HP",
                                (ItemSet::Ice,   true)  => "+25% DEF, +2 DEF, +5 HP",
                                (ItemSet::Storm, false) => "+15% Speed, +10% Crit",
                                (ItemSet::Storm, true)  => "+25% Speed, +20% Crit, +10% ATK",
                                _ => "",
                            };
                            let bonus_label = if *count >= 3 {
                                format!("{}/3 {} Set (FULL)", count, set_name)
                            } else {
                                format!("{}/3 {} Set", count, set_name)
                            };

                            left.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(1.0),
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            }).with_children(|bonus| {
                                bonus.spawn((
                                    Text::new(bonus_label),
                                    TextFont { font: f.clone(), font_size: 7.0, ..default() },
                                    TextColor(Color::srgb(0.3, 0.9, 0.3)),
                                ));
                                bonus.spawn((
                                    Text::new(bonus_desc),
                                    TextFont { font: f.clone(), font_size: 6.0, ..default() },
                                    TextColor(set_color),
                                ));
                            });
                        }
                    }
                });

                // ═══════════════════════════════════════
                // CENTER COLUMN: Stats
                // ═══════════════════════════════════════
                cols.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        width: Val::Px(180.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor(panel_border),
                )).with_children(|center| {
                    // Column header
                    center.spawn((
                        Text::new("STATS"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(col_header),
                        Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
                    ));

                    // HP
                    spawn_stat_row(
                        center, &f,
                        &format!("HP: {}/{}", player.health, player.max_health),
                        Color::srgb(0.85, 0.3, 0.2),
                        None,
                    );
                    // Mana
                    spawn_stat_row(
                        center, &f,
                        &format!("Mana: {}/{}", player.mana as i32, player.max_mana as i32),
                        Color::srgb(0.5, 0.4, 0.9),
                        None,
                    );
                    // ATK
                    let atk_suffix = if run.stat_attack > 0 {
                        Some((format!(" (+{} Lv)", run.stat_attack), Color::srgb(0.3, 0.9, 0.3)))
                    } else { None };
                    spawn_stat_row(
                        center, &f,
                        &format!("ATK: {}", stats.attack),
                        Color::srgb(0.9, 0.6, 0.2),
                        atk_suffix.as_ref().map(|(s, c)| (s.as_str(), *c)),
                    );
                    // DEF
                    let def_suffix = if run.stat_defense > 0 {
                        Some((format!(" (+{} Lv)", run.stat_defense), Color::srgb(0.3, 0.9, 0.3)))
                    } else { None };
                    spawn_stat_row(
                        center, &f,
                        &format!("DEF: {}", stats.defense),
                        Color::srgb(0.4, 0.6, 0.9),
                        def_suffix.as_ref().map(|(s, c)| (s.as_str(), *c)),
                    );
                    // Speed
                    let spd_suffix = if run.stat_speed > 0 {
                        Some((format!(" (+{} Lv)", run.stat_speed), Color::srgb(0.3, 0.9, 0.3)))
                    } else { None };
                    spawn_stat_row(
                        center, &f,
                        &format!("Speed: {:.0}%", stats.speed_mult * 100.0),
                        Color::srgb(0.3, 0.8, 0.4),
                        spd_suffix.as_ref().map(|(s, c)| (s.as_str(), *c)),
                    );
                    // Crit
                    spawn_stat_row(
                        center, &f,
                        &format!("Crit: {:.0}%", stats.crit_chance * 100.0),
                        Color::srgb(0.9, 0.8, 0.2),
                        None,
                    );
                    // Gold Bonus (only if > 0)
                    if stats.gold_bonus > 0.0 {
                        spawn_stat_row(
                            center, &f,
                            &format!("Gold Bonus: +{:.0}%", stats.gold_bonus * 100.0),
                            Color::srgb(1.0, 0.85, 0.2),
                            None,
                        );
                    }
                    // Lifesteal (only if > 0)
                    if stats.lifesteal > 0.0 {
                        spawn_stat_row(
                            center, &f,
                            &format!("Lifesteal: {:.0}%", stats.lifesteal * 100.0),
                            Color::srgb(0.8, 0.2, 0.4),
                            None,
                        );
                    }
                });

                // ═══════════════════════════════════════
                // RIGHT COLUMN: Relics
                // ═══════════════════════════════════════
                cols.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        width: Val::Px(200.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(panel_bg),
                    BorderColor(panel_border),
                )).with_children(|right| {
                    // Column header
                    right.spawn((
                        Text::new("RELICS"),
                        TextFont { font: f.clone(), font_size: 10.0, ..default() },
                        TextColor(col_header),
                        Node { margin: UiRect::bottom(Val::Px(6.0)), ..default() },
                    ));

                    if relic_inv.relics.is_empty() {
                        right.spawn((
                            Text::new("No relics yet"),
                            TextFont { font: f.clone(), font_size: 7.0, ..default() },
                            TextColor(col_dim),
                        ));
                    } else {
                        for relic in &relic_inv.relics {
                            right.spawn(Node {
                                flex_direction: FlexDirection::Row,
                                column_gap: Val::Px(8.0),
                                align_items: AlignItems::FlexStart,
                                margin: UiRect::bottom(Val::Px(6.0)),
                                ..default()
                            }).with_children(|row| {
                                // 32x32 relic icon
                                row.spawn((
                                    ImageNode::new(relic_icons.handle_for(*relic).clone()),
                                    Node {
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                ));
                                // Name + description
                                row.spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                }).with_children(|info| {
                                    info.spawn((
                                        Text::new(relic.name()),
                                        TextFont { font: f.clone(), font_size: 7.5, ..default() },
                                        TextColor(relic.color()),
                                    ));
                                    info.spawn((
                                        Text::new(relic.description()),
                                        TextFont { font: f.clone(), font_size: 6.0, ..default() },
                                        TextColor(col_dim),
                                    ));
                                });
                            });
                        }
                    }
                });
            });

            // ── Bottom hint ──
            root.spawn((
                Text::new("[TAB / I] Toggle Inventory  |  [ESC] Close"),
                TextFont { font: f.clone(), font_size: 6.5, ..default() },
                TextColor(col_dim),
                Node { margin: UiRect::top(Val::Px(10.0)), ..default() },
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

fn spawn_stat_row(
    parent: &mut ChildBuilder,
    font: &Handle<Font>,
    label: &str,
    color: Color,
    suffix: Option<(&str, Color)>,
) {
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        ..default()
    }).with_children(|row| {
        row.spawn((
            Text::new(label),
            TextFont { font: font.clone(), font_size: 7.0, ..default() },
            TextColor(color),
        ));
        if let Some((suf_text, suf_color)) = suffix {
            row.spawn((
                Text::new(suf_text),
                TextFont { font: font.clone(), font_size: 6.5, ..default() },
                TextColor(suf_color),
            ));
        }
    });
}

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
        padding: UiRect::all(Val::Px(3.0)),
        margin: UiRect::bottom(Val::Px(2.0)),
        ..default()
    }).with_children(|row| {
        match item {
            Some(id) => {
                let data = id.data();
                // 24x24 icon
                row.spawn((
                    ImageNode::new(icon_assets.handle_for(id).clone()),
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                ));
                // Name + description + set tag
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(1.0),
                    ..default()
                }).with_children(|info| {
                    let name_color = match data.set {
                        ItemSet::Fire  => Color::srgb(1.0, 0.5, 0.2),
                        ItemSet::Ice   => Color::srgb(0.4, 0.8, 1.0),
                        ItemSet::Storm => Color::srgb(0.6, 0.4, 1.0),
                        ItemSet::None  => Color::srgb(0.85, 0.8, 0.7),
                    };
                    // Slot label + item name
                    info.spawn((
                        Text::new(format!("[{}] {}", slot_label, data.name)),
                        TextFont { font: font.clone(), font_size: 7.0, ..default() },
                        TextColor(name_color),
                    ));
                    // Description
                    info.spawn((
                        Text::new(data.description),
                        TextFont { font: font.clone(), font_size: 6.0, ..default() },
                        TextColor(Color::srgba(0.55, 0.5, 0.4, 0.9)),
                    ));
                    // Set tag
                    if data.set != ItemSet::None {
                        let set_name = match data.set {
                            ItemSet::Fire  => "[Fire Set]",
                            ItemSet::Ice   => "[Ice Set]",
                            ItemSet::Storm => "[Storm Set]",
                            _ => "",
                        };
                        info.spawn((
                            Text::new(set_name),
                            TextFont { font: font.clone(), font_size: 5.5, ..default() },
                            TextColor(Color::srgb(0.4, 0.7, 0.4)),
                        ));
                    }
                });
            }
            None => {
                // Empty slot placeholder
                row.spawn((
                    Node {
                        width: Val::Px(24.0),
                        height: Val::Px(24.0),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.12, 0.6)),
                    BorderColor(Color::srgba(0.3, 0.25, 0.35, 0.5)),
                ));
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(1.0),
                    ..default()
                }).with_children(|info| {
                    info.spawn((
                        Text::new(format!("[{}]", slot_label)),
                        TextFont { font: font.clone(), font_size: 6.5, ..default() },
                        TextColor(Color::srgba(0.5, 0.45, 0.38, 0.7)),
                    ));
                    info.spawn((
                        Text::new("— Empty —"),
                        TextFont { font: font.clone(), font_size: 6.5, ..default() },
                        TextColor(Color::srgba(0.4, 0.35, 0.3, 0.6)),
                    ));
                });
            }
        }
    });
}
