use bevy::prelude::*;
use crate::GameState;

pub struct EquipmentPlugin;

impl Plugin for EquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerStats::default())
            .add_event::<RecalcStats>()
            .add_systems(OnEnter(GameState::Playing), init_equipment)
            .add_systems(
                Update,
                recalculate_stats.run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Item definitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ItemId {
    // Weapons
    RustySword,
    SteelBlade,
    FlameEdge,
    FrostFang,
    ThunderCleaver,
    // Armor
    LeatherTunic,
    ChainMail,
    EmberPlate,
    FrostGuard,
    StormArmor,
    // Relics (passive)
    LifeRing,
    ManaStone,
    GoldMagnet,
    SpeedBoots,
    CritCharm,
    // Charms (passive secondary)
    FireAmulet,
    IceAmulet,
    StormAmulet,
    VampireFang,
    IronWill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemSlot {
    Weapon,
    Armor,
    Relic,
    Charm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemSet {
    None,
    Fire,   // FlameEdge + EmberPlate + FireAmulet
    Ice,    // FrostFang + FrostGuard + IceAmulet
    Storm,  // ThunderCleaver + StormArmor + StormAmulet
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemTier {
    Tier1,
    Tier2,
    Tier3,
}

/// Static item data — stats, slot, set, tier, cost.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ItemData {
    pub id: ItemId,
    pub name: &'static str,
    pub description: &'static str,
    pub slot: ItemSlot,
    pub set: ItemSet,
    pub tier: ItemTier,
    pub cost: i32,
    pub modifiers: StatModifiers,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StatModifiers {
    pub attack_flat: i32,
    pub attack_percent: f32,
    pub defense_flat: i32,
    pub defense_percent: f32,
    pub max_health_flat: i32,
    pub max_mana_flat: f32,
    pub mana_regen_percent: f32,
    pub speed_percent: f32,
    pub crit_chance: f32,
    pub gold_bonus_percent: f32,
    pub lifesteal: f32,
}

impl ItemId {
    pub fn data(&self) -> ItemData {
        match self {
            // --- Weapons ---
            ItemId::RustySword => ItemData {
                id: *self, name: "Rusty Sword", description: "A worn blade. Better than fists.",
                slot: ItemSlot::Weapon, set: ItemSet::None, tier: ItemTier::Tier1, cost: 25,
                modifiers: StatModifiers { attack_flat: 1, ..default() },
            },
            ItemId::SteelBlade => ItemData {
                id: *self, name: "Steel Blade", description: "Reliable and sharp.",
                slot: ItemSlot::Weapon, set: ItemSet::None, tier: ItemTier::Tier2, cost: 60,
                modifiers: StatModifiers { attack_flat: 2, attack_percent: 0.1, ..default() },
            },
            ItemId::FlameEdge => ItemData {
                id: *self, name: "Flame Edge", description: "Burns with inner fire.",
                slot: ItemSlot::Weapon, set: ItemSet::Fire, tier: ItemTier::Tier3, cost: 100,
                modifiers: StatModifiers { attack_flat: 3, attack_percent: 0.15, ..default() },
            },
            ItemId::FrostFang => ItemData {
                id: *self, name: "Frost Fang", description: "Cold to the touch.",
                slot: ItemSlot::Weapon, set: ItemSet::Ice, tier: ItemTier::Tier3, cost: 100,
                modifiers: StatModifiers { attack_flat: 2, attack_percent: 0.2, ..default() },
            },
            ItemId::ThunderCleaver => ItemData {
                id: *self, name: "Thunder Cleaver", description: "Crackles with lightning.",
                slot: ItemSlot::Weapon, set: ItemSet::Storm, tier: ItemTier::Tier3, cost: 120,
                modifiers: StatModifiers { attack_flat: 3, attack_percent: 0.1, crit_chance: 0.1, ..default() },
            },
            // --- Armor ---
            ItemId::LeatherTunic => ItemData {
                id: *self, name: "Leather Tunic", description: "Basic protection.",
                slot: ItemSlot::Armor, set: ItemSet::None, tier: ItemTier::Tier1, cost: 30,
                modifiers: StatModifiers { defense_flat: 1, ..default() },
            },
            ItemId::ChainMail => ItemData {
                id: *self, name: "Chain Mail", description: "Heavy but effective.",
                slot: ItemSlot::Armor, set: ItemSet::None, tier: ItemTier::Tier2, cost: 65,
                modifiers: StatModifiers { defense_flat: 2, max_health_flat: 2, ..default() },
            },
            ItemId::EmberPlate => ItemData {
                id: *self, name: "Ember Plate", description: "Forged in volcanic heat.",
                slot: ItemSlot::Armor, set: ItemSet::Fire, tier: ItemTier::Tier3, cost: 110,
                modifiers: StatModifiers { defense_flat: 3, defense_percent: 0.1, ..default() },
            },
            ItemId::FrostGuard => ItemData {
                id: *self, name: "Frost Guard", description: "Chills attackers on contact.",
                slot: ItemSlot::Armor, set: ItemSet::Ice, tier: ItemTier::Tier3, cost: 110,
                modifiers: StatModifiers { defense_flat: 2, max_health_flat: 3, defense_percent: 0.15, ..default() },
            },
            ItemId::StormArmor => ItemData {
                id: *self, name: "Storm Armor", description: "Charged with static.",
                slot: ItemSlot::Armor, set: ItemSet::Storm, tier: ItemTier::Tier3, cost: 115,
                modifiers: StatModifiers { defense_flat: 2, speed_percent: 0.1, defense_percent: 0.1, ..default() },
            },
            // --- Relics ---
            ItemId::LifeRing => ItemData {
                id: *self, name: "Life Ring", description: "Grants vitality.",
                slot: ItemSlot::Relic, set: ItemSet::None, tier: ItemTier::Tier1, cost: 35,
                modifiers: StatModifiers { max_health_flat: 3, ..default() },
            },
            ItemId::ManaStone => ItemData {
                id: *self, name: "Mana Stone", description: "Deepens the mana pool.",
                slot: ItemSlot::Relic, set: ItemSet::None, tier: ItemTier::Tier1, cost: 35,
                modifiers: StatModifiers { max_mana_flat: 30.0, mana_regen_percent: 0.15, ..default() },
            },
            ItemId::GoldMagnet => ItemData {
                id: *self, name: "Gold Magnet", description: "Attracts wealth.",
                slot: ItemSlot::Relic, set: ItemSet::None, tier: ItemTier::Tier2, cost: 50,
                modifiers: StatModifiers { gold_bonus_percent: 0.25, ..default() },
            },
            ItemId::SpeedBoots => ItemData {
                id: *self, name: "Speed Boots", description: "Lighter than air.",
                slot: ItemSlot::Relic, set: ItemSet::None, tier: ItemTier::Tier2, cost: 55,
                modifiers: StatModifiers { speed_percent: 0.2, ..default() },
            },
            ItemId::CritCharm => ItemData {
                id: *self, name: "Crit Charm", description: "Fortune favors the bold.",
                slot: ItemSlot::Relic, set: ItemSet::None, tier: ItemTier::Tier2, cost: 60,
                modifiers: StatModifiers { crit_chance: 0.15, attack_percent: 0.05, ..default() },
            },
            // --- Charms ---
            ItemId::FireAmulet => ItemData {
                id: *self, name: "Fire Amulet", description: "Ember warmth within.",
                slot: ItemSlot::Charm, set: ItemSet::Fire, tier: ItemTier::Tier2, cost: 45,
                modifiers: StatModifiers { attack_percent: 0.1, ..default() },
            },
            ItemId::IceAmulet => ItemData {
                id: *self, name: "Ice Amulet", description: "Glacial resilience.",
                slot: ItemSlot::Charm, set: ItemSet::Ice, tier: ItemTier::Tier2, cost: 45,
                modifiers: StatModifiers { defense_percent: 0.1, max_health_flat: 1, ..default() },
            },
            ItemId::StormAmulet => ItemData {
                id: *self, name: "Storm Amulet", description: "Electric reflexes.",
                slot: ItemSlot::Charm, set: ItemSet::Storm, tier: ItemTier::Tier2, cost: 45,
                modifiers: StatModifiers { speed_percent: 0.1, crit_chance: 0.05, ..default() },
            },
            ItemId::VampireFang => ItemData {
                id: *self, name: "Vampire Fang", description: "Drink deep.",
                slot: ItemSlot::Charm, set: ItemSet::None, tier: ItemTier::Tier3, cost: 80,
                modifiers: StatModifiers { lifesteal: 0.1, attack_flat: 1, ..default() },
            },
            ItemId::IronWill => ItemData {
                id: *self, name: "Iron Will", description: "Unbreakable spirit.",
                slot: ItemSlot::Charm, set: ItemSet::None, tier: ItemTier::Tier3, cost: 75,
                modifiers: StatModifiers { defense_flat: 1, max_health_flat: 5, ..default() },
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Set bonus definitions
// ---------------------------------------------------------------------------

impl ItemSet {
    /// Number of items of this set needed for the 2-piece bonus.
    pub fn bonus_2pc(&self) -> StatModifiers {
        match self {
            ItemSet::Fire => StatModifiers { attack_percent: 0.15, ..default() },
            ItemSet::Ice => StatModifiers { defense_percent: 0.15, max_health_flat: 2, ..default() },
            ItemSet::Storm => StatModifiers { speed_percent: 0.15, crit_chance: 0.1, ..default() },
            ItemSet::None => StatModifiers::default(),
        }
    }

    /// 3-piece bonus (full set).
    pub fn bonus_3pc(&self) -> StatModifiers {
        match self {
            ItemSet::Fire => StatModifiers { attack_percent: 0.25, attack_flat: 2, lifesteal: 0.05, ..default() },
            ItemSet::Ice => StatModifiers { defense_percent: 0.25, defense_flat: 2, max_health_flat: 5, ..default() },
            ItemSet::Storm => StatModifiers { speed_percent: 0.25, crit_chance: 0.2, attack_percent: 0.1, ..default() },
            ItemSet::None => StatModifiers::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Equipment component (on the Player entity)
// ---------------------------------------------------------------------------

#[derive(Component, Default, Clone)]
pub struct Equipment {
    pub weapon: Option<ItemId>,
    pub armor: Option<ItemId>,
    pub relic: Option<ItemId>,
    pub charm: Option<ItemId>,
}

impl Equipment {
    pub fn get_slot(&self, slot: ItemSlot) -> &Option<ItemId> {
        match slot {
            ItemSlot::Weapon => &self.weapon,
            ItemSlot::Armor => &self.armor,
            ItemSlot::Relic => &self.relic,
            ItemSlot::Charm => &self.charm,
        }
    }

    pub fn set_slot(&mut self, slot: ItemSlot, item: Option<ItemId>) {
        match slot {
            ItemSlot::Weapon => self.weapon = item,
            ItemSlot::Armor => self.armor = item,
            ItemSlot::Relic => self.relic = item,
            ItemSlot::Charm => self.charm = item,
        }
    }

    pub fn equipped_items(&self) -> Vec<ItemId> {
        [self.weapon, self.armor, self.relic, self.charm]
            .iter()
            .filter_map(|x| *x)
            .collect()
    }

    /// Equip an item, returning the previously equipped item in that slot (if any).
    pub fn equip(&mut self, item: ItemId) -> Option<ItemId> {
        let slot = item.data().slot;
        let old = *self.get_slot(slot);
        self.set_slot(slot, Some(item));
        old
    }
}

// ---------------------------------------------------------------------------
// Computed player stats (recalculated from base + equipment)
// ---------------------------------------------------------------------------

#[derive(Resource, Debug, Clone)]
#[allow(dead_code)]
pub struct PlayerStats {
    pub attack: i32,
    pub defense: i32,
    pub max_health_bonus: i32,
    pub max_mana_bonus: f32,
    pub mana_regen_mult: f32,
    pub speed_mult: f32,
    pub crit_chance: f32,
    pub gold_bonus: f32,
    pub lifesteal: f32,
    pub active_set_bonuses: Vec<(ItemSet, u8)>, // (set, piece count)
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            attack: 0,
            defense: 0,
            max_health_bonus: 0,
            max_mana_bonus: 0.0,
            mana_regen_mult: 1.0,
            speed_mult: 1.0,
            crit_chance: 0.0,
            gold_bonus: 0.0,
            lifesteal: 0.0,
            active_set_bonuses: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Fire this event to trigger a stat recalculation.
#[derive(Event)]
pub struct RecalcStats;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn init_equipment(
    mut commands: Commands,
    player_q: Query<Entity, With<crate::player::Player>>,
) {
    for entity in &player_q {
        commands.entity(entity).insert(Equipment::default());
    }
}

fn recalculate_stats(
    mut ev: EventReader<RecalcStats>,
    player_equip_q: Query<(&Equipment, &crate::player::Player)>,
    mut stats: ResMut<PlayerStats>,
) {
    // Recalc on event, or on first frame if stats are default
    let should_recalc = ev.read().next().is_some();
    // Drain remaining events
    for _ in ev.read() {}

    if !should_recalc { return; }

    let Ok((equip, player)) = player_equip_q.get_single() else { return };

    // Start from base
    let mut total = StatModifiers::default();

    // Sum all equipped item modifiers
    for item_id in equip.equipped_items() {
        let data = item_id.data();
        add_modifiers(&mut total, &data.modifiers);
    }

    // Count set pieces
    let mut set_counts = std::collections::HashMap::new();
    for item_id in equip.equipped_items() {
        let set = item_id.data().set;
        if set != ItemSet::None {
            *set_counts.entry(set).or_insert(0u8) += 1;
        }
    }

    // Apply set bonuses
    let mut active_sets = Vec::new();
    for (set, count) in &set_counts {
        if *count >= 2 {
            add_modifiers(&mut total, &set.bonus_2pc());
            active_sets.push((*set, *count));
        }
        if *count >= 3 {
            add_modifiers(&mut total, &set.bonus_3pc());
        }
    }

    // Write computed stats (equipment + player shop bonuses)
    *stats = PlayerStats {
        attack: total.attack_flat + player.bonus_attack,
        defense: total.defense_flat + player.bonus_defense,
        max_health_bonus: total.max_health_flat,
        max_mana_bonus: total.max_mana_flat,
        mana_regen_mult: 1.0 + total.mana_regen_percent,
        speed_mult: 1.0 + total.speed_percent + player.bonus_speed,
        crit_chance: total.crit_chance,
        gold_bonus: total.gold_bonus_percent,
        lifesteal: total.lifesteal,
        active_set_bonuses: active_sets,
    };
}

fn add_modifiers(target: &mut StatModifiers, source: &StatModifiers) {
    target.attack_flat += source.attack_flat;
    target.attack_percent += source.attack_percent;
    target.defense_flat += source.defense_flat;
    target.defense_percent += source.defense_percent;
    target.max_health_flat += source.max_health_flat;
    target.max_mana_flat += source.max_mana_flat;
    target.mana_regen_percent += source.mana_regen_percent;
    target.speed_percent += source.speed_percent;
    target.crit_chance += source.crit_chance;
    target.gold_bonus_percent += source.gold_bonus_percent;
    target.lifesteal += source.lifesteal;
}
