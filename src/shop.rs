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
            (reset_shop_on_transition, shop_interaction, purchase_feedback_decay)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

// ---------------------------------------------------------------------------
// Shop item definitions
// ---------------------------------------------------------------------------

#[derive(Component)]
struct ShopItem {
    name: String,
    cost: i32,
    effect: ShopEffect,
    tier: ShopTier,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShopTier {
    Tier1,
    Tier2,
    Tier3,
}

#[derive(Clone)]
enum ShopEffect {
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
enum UnlockReq {
    None,
    MinFloor(i32),
    MinRuns(i32),
    MinFloorOrRuns(i32, i32),
}

struct ShopEntry {
    name: &'static str,
    cost: i32,
    effect: ShopEffect,
    tier: ShopTier,
    unlock: UnlockReq,
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

// ---------------------------------------------------------------------------
// Resources & components
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct ShopSpawned(bool);

/// Brief floating text when buying something.
#[derive(Component)]
struct PurchaseFeedback {
    timer: f32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn reset_shop_on_transition(
    mut ev: EventReader<RoomTransition>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        commands.insert_resource(ShopSpawned(false));
    }
}

/// Maximum items shown in a single shop visit.
const MAX_SHOP_SLOTS: usize = 5;

fn shop_interaction(
    mut commands: Commands,
    room_state: Res<RoomState>,
    mut run: ResMut<RunData>,
    meta: Res<MetaProgression>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    shop_items: Query<(Entity, &ShopItem, &Transform)>,
    player_q: Query<&Transform, (With<crate::player::Player>, Without<ShopItem>)>,
    mut player_mut: Query<&mut crate::player::Player, Without<crate::spell::SpellSlots>>,
    mut spell_slots_q: Query<&mut crate::spell::SpellSlots, Without<crate::player::Player>>,
    mut equipment_q: Query<&mut Equipment, With<crate::player::Player>>,
    mut recalc_ev: EventWriter<RecalcStats>,
    shop_spawned: Option<Res<ShopSpawned>>,
    font: Res<GameFont>,
) {
    if room_state.current_type != RoomType::Shop { return; }

    // Spawn shop items if not yet spawned
    if shop_spawned.map_or(true, |s| !s.0) {
        commands.insert_resource(ShopSpawned(true));

        // Filter items by unlock requirements
        let available: Vec<ShopEntry> = all_shop_entries()
            .into_iter()
            .filter(|e| e.unlock.is_met(run.current_floor, &meta))
            .collect();

        // Pick up to MAX_SHOP_SLOTS items, randomized from pool
        let selected = pick_shop_items(&available, run.current_floor);

        for (i, entry) in selected.iter().enumerate() {
            let x = 100.0 + i as f32 * 140.0;

            // Item color based on tier
            let color = match entry.tier {
                ShopTier::Tier1 => Color::srgb(0.8, 0.7, 0.3),
                ShopTier::Tier2 => Color::srgb(0.4, 0.7, 0.9),
                ShopTier::Tier3 => Color::srgb(0.85, 0.5, 0.9),
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(x, 100.0, crate::constants::Z_PICKUPS),
                ShopItem {
                    name: entry.name.to_string(),
                    cost: entry.cost,
                    effect: entry.effect.clone(),
                    tier: entry.tier,
                },
                PlayingEntity,
            ));

            // Tier label color
            let tier_label = match entry.tier {
                ShopTier::Tier1 => "",
                ShopTier::Tier2 => "[T2] ",
                ShopTier::Tier3 => "[T3] ",
            };

            // Price label
            commands.spawn((
                Text2d::new(format!("{}{}: {}g", tier_label, entry.name, entry.cost)),
                TextFont { font: font.0.clone(), font_size: 7.0, ..default() },
                TextColor(Color::srgb(0.9, 0.85, 0.5)),
                Transform::from_xyz(x, 130.0, crate::constants::Z_HUD),
                PlayingEntity,
            ));
        }
    }

    // Purchase on E key / gamepad West(X) when near item
    let gp_buy = gamepads.iter().next().map_or(false, |g| g.just_pressed(GamepadButton::West));
    if !keys.just_pressed(KeyCode::KeyE) && !gp_buy { return; }
    let Ok(p_tf) = player_q.get_single() else { return };

    for (entity, item, tf) in &shop_items {
        let dist = (p_tf.translation.xy() - tf.translation.xy()).length();
        if dist > 50.0 { continue; }

        if run.gold < item.cost {
            // Show "not enough gold" feedback
            spawn_feedback(&mut commands, tf.translation, "Not enough gold!", Color::srgb(0.9, 0.3, 0.2), &font);
            continue;
        }

        run.gold -= item.cost;

        match &item.effect {
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

        // Purchase feedback
        let msg = format!("Bought {}!", item.name);
        spawn_feedback(&mut commands, tf.translation, &msg, Color::srgb(0.3, 0.9, 0.3), &font);
        commands.entity(entity).try_despawn_recursive();
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

/// Pick up to MAX_SHOP_SLOTS items from the available pool.
/// Guarantees at least 1 consumable and tries to include varied categories.
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

    // Always include one consumable (Heal Full, MaxHpUp, or ManaUp)
    let consumables: Vec<usize> = available.iter().enumerate()
        .filter(|(_, e)| matches!(e.effect, ShopEffect::HealFull | ShopEffect::MaxHpUp | ShopEffect::ManaUp))
        .map(|(i, _)| i)
        .collect();
    if !consumables.is_empty() {
        let idx = consumables[rand::random::<usize>() % consumables.len()];
        used_indices.insert(idx);
        selected.push(idx);
    }

    // Fill remaining slots randomly
    let mut remaining: Vec<usize> = (0..available.len())
        .filter(|i| !used_indices.contains(i))
        .collect();

    // Shuffle with Fisher-Yates
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
