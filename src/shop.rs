use bevy::prelude::*;
use crate::{GameState, GameFont, RunData, PlayingEntity, room::{RoomState, RoomType, RoomTransition}};

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (reset_shop_on_transition, shop_interaction)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn reset_shop_on_transition(
    mut ev: EventReader<RoomTransition>,
    mut commands: Commands,
) {
    for _ in ev.read() {
        commands.insert_resource(ShopSpawned(false));
    }
}

#[derive(Component)]
#[allow(dead_code)]
struct ShopItem {
    name: String,
    cost: i32,
    effect: ShopEffect,
}

#[derive(Clone)]
#[allow(dead_code)]
enum ShopEffect {
    HealFull,
    MaxHpUp,
    DamageUp,
    ManaUp,
    UnlockSpell(usize, crate::spell::SpellId),
}

#[derive(Resource)]
struct ShopSpawned(bool);

fn shop_interaction(
    mut commands: Commands,
    room_state: Res<RoomState>,
    mut run: ResMut<RunData>,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    shop_items: Query<(Entity, &ShopItem, &Transform)>,
    player_q: Query<&Transform, (With<crate::player::Player>, Without<ShopItem>)>,
    mut player_mut: Query<&mut crate::player::Player, Without<crate::spell::SpellSlots>>,
    mut spell_slots_q: Query<&mut crate::spell::SpellSlots, Without<crate::player::Player>>,
    shop_spawned: Option<Res<ShopSpawned>>,
    font: Res<GameFont>,
) {
    if room_state.current_type != RoomType::Shop { return; }

    // Spawn shop items if not yet spawned
    if shop_spawned.map_or(true, |s| !s.0) {
        commands.insert_resource(ShopSpawned(true));
        let items = vec![
            ("Heal Full",  20, ShopEffect::HealFull),
            ("+1 Max HP",  40, ShopEffect::MaxHpUp),
            ("+Mana Pool", 30, ShopEffect::ManaUp),
            ("Fireball",   35, ShopEffect::UnlockSpell(0, crate::spell::SpellId::Fireball)),
            ("Ice Shards", 30, ShopEffect::UnlockSpell(1, crate::spell::SpellId::IceShards)),
            ("Lightning",  50, ShopEffect::UnlockSpell(2, crate::spell::SpellId::Lightning)),
            ("Shield",     45, ShopEffect::UnlockSpell(3, crate::spell::SpellId::Shield)),
        ];

        for (i, (name, cost, effect)) in items.into_iter().enumerate() {
            let x = 120.0 + i as f32 * 110.0;
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.8, 0.7, 0.3),
                    custom_size: Some(Vec2::new(30.0, 30.0)),
                    ..default()
                },
                Transform::from_xyz(x, 100.0, crate::constants::Z_PICKUPS),
                ShopItem {
                    name: name.to_string(),
                    cost,
                    effect,
                },
                PlayingEntity,
            ));

            // Price label
            commands.spawn((
                Text2d::new(format!("{}: {}g", name, cost)),
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
        if dist < 50.0 && run.gold >= item.cost {
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
                ShopEffect::DamageUp => {}
                ShopEffect::UnlockSpell(slot, spell) => {
                    if let Ok(mut slots) = spell_slots_q.get_single_mut() {
                        slots.slots[*slot] = Some(*spell);
                    }
                }
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}
