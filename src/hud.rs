use bevy::prelude::*;
use crate::{GameState, GameFont, RunData, PlayingEntity, player::Player, spell::SpellSlots, room::{RoomState, RoomType}};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                (update_hud, update_minimap, health_bar_update).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct ManaText;

#[derive(Component)]
struct GoldText;

#[derive(Component)]
struct FloorText;

#[derive(Component)]
struct SpellText;

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

fn setup_hud(mut commands: Commands, font: Res<GameFont>) {
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

            // Bottom: Spell slots
            parent.spawn((
                Text::new("[1] Fireball [2] Ice [3] Lightning [4] Shield"),
                TextFont { font: font.0.clone(), font_size: 7.0, ..default() },
                TextColor(Color::srgb(0.75, 0.6, 0.4)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                SpellText,
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
    slots_q: Query<&SpellSlots>,
    mut score_q:  Query<&mut Text, (With<ScoreText>,  Without<ManaText>, Without<GoldText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut mana_q:   Query<&mut Text, (With<ManaText>,   Without<ScoreText>, Without<GoldText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut gold_q:   Query<&mut Text, (With<GoldText>,   Without<ScoreText>, Without<ManaText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut floor_q:  Query<&mut Text, (With<FloorText>,  Without<ScoreText>, Without<ManaText>, Without<GoldText>,  Without<SpellText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut spell_q:  Query<&mut Text, (With<SpellText>,  Without<ScoreText>, Without<ManaText>, Without<GoldText>,  Without<FloorText>, Without<EnemyText>, Without<HealthBarLabel>)>,
    mut enemy_q:  Query<&mut Text, (With<EnemyText>,  Without<ScoreText>, Without<ManaText>, Without<GoldText>,  Without<FloorText>, Without<SpellText>, Without<HealthBarLabel>)>,
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

    if let Ok(mut text) = spell_q.get_single_mut() {
        if let Ok(slots) = slots_q.get_single() {
            let mut parts = Vec::new();
            for (i, slot) in slots.slots.iter().enumerate() {
                if let Some(spell) = slot {
                    let cd = slots.cooldowns[i];
                    if cd > 0.0 {
                        parts.push(format!("[{}] {} ({:.1}s)", i + 1, spell.name(), cd));
                    } else {
                        parts.push(format!("[{}] {}", i + 1, spell.name()));
                    }
                } else {
                    parts.push(format!("[{}] ---", i + 1));
                }
            }
            **text = parts.join("  ");
        }
    }

    if let Ok(mut text) = enemy_q.get_single_mut() {
        if run.enemies_alive > 0 {
            **text = format!("Enemies: {}", run.enemies_alive);
        } else {
            **text = String::new();
        }
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
