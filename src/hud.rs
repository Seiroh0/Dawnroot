use bevy::prelude::*;
use crate::{GameState, GameFont, RunData, PlayingEntity, player::Player, spell::SpellSlots, room::RoomState};

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_hud)
            .add_systems(
                Update,
                update_hud.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HealthText;

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
                    // Left: Health + Mana
                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|col| {
                        let f = font.0.clone();
                        col.spawn((
                            Text::new("HP: 10/10"),
                            TextFont { font: f.clone(), font_size: 9.0, ..default() },
                            TextColor(Color::srgb(0.9, 0.35, 0.15)),
                            HealthText,
                        ));
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
        });
}

fn update_hud(
    run: Res<RunData>,
    room_state: Res<RoomState>,
    player_q: Query<&Player>,
    slots_q: Query<&SpellSlots>,
    mut score_q:  Query<&mut Text, (With<ScoreText>,  Without<HealthText>, Without<ManaText>, Without<GoldText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>)>,
    mut health_q: Query<&mut Text, (With<HealthText>, Without<ScoreText>,  Without<ManaText>, Without<GoldText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>)>,
    mut mana_q:   Query<&mut Text, (With<ManaText>,   Without<ScoreText>,  Without<HealthText>, Without<GoldText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>)>,
    mut gold_q:   Query<&mut Text, (With<GoldText>,   Without<ScoreText>,  Without<HealthText>, Without<ManaText>, Without<FloorText>, Without<SpellText>, Without<EnemyText>)>,
    mut floor_q:  Query<&mut Text, (With<FloorText>,  Without<ScoreText>,  Without<HealthText>, Without<ManaText>, Without<GoldText>,  Without<SpellText>, Without<EnemyText>)>,
    mut spell_q:  Query<&mut Text, (With<SpellText>,  Without<ScoreText>,  Without<HealthText>, Without<ManaText>, Without<GoldText>,  Without<FloorText>, Without<EnemyText>)>,
    mut enemy_q:  Query<&mut Text, (With<EnemyText>,  Without<ScoreText>,  Without<HealthText>, Without<ManaText>, Without<GoldText>,  Without<FloorText>, Without<SpellText>)>,
) {
    let player = player_q.get_single().ok();

    if let Ok(mut text) = score_q.get_single_mut() {
        **text = format!("{:06}", run.score);
    }

    if let Ok(mut text) = health_q.get_single_mut() {
        if let Some(p) = player {
            **text = format!("HP: {}/{}", p.health, p.max_health);
        }
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
