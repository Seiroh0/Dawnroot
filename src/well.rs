use bevy::prelude::*;
use crate::{
    constants::*,
    GameState, GameFont, PlayingEntity,
    player::Player,
    room::RoomEntity,
    audio::{PlaySfxEvent, SfxType},
};

pub struct WellPlugin;

impl Plugin for WellPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();
        let assets = WellAssets {
            well_full: asset_server.load("sprites/Well1.png"),
            well_empty: asset_server.load("sprites/Well2.png"),
        };
        app.insert_resource(assets);

        app.add_systems(
            Update,
            (well_interaction, well_prompt_ui)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
pub struct WellAssets {
    pub well_full: Handle<Image>,
    pub well_empty: Handle<Image>,
}

#[derive(Component)]
pub struct HealingWell {
    pub used: bool,
}

/// UI prompt shown when player is near the well.
#[derive(Component)]
pub struct WellPrompt;

/// Marker for the well's sprite child (so we can swap the image).
#[derive(Component)]
pub struct WellSprite;

const WELL_INTERACT_RANGE: f32 = 80.0; // 2 tiles
const WELL_HEAL_AMOUNT: i32 = 2;

/// Spawn a healing well at a given position. Called from room.rs.
pub fn spawn_well(commands: &mut Commands, x: f32, y: f32, assets: &WellAssets) {
    commands.spawn((
        // Invisible collision root
        Sprite {
            color: Color::NONE,
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(x, y + 20.0, Z_PICKUPS + 0.5),
        HealingWell { used: false },
        RoomEntity,
        PlayingEntity,
    )).with_children(|parent| {
        parent.spawn((
            Sprite {
                image: assets.well_full.clone(),
                custom_size: Some(Vec2::new(64.0, 64.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            WellSprite,
        ));
    });
}

fn well_interaction(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut player_q: Query<(&mut Player, &Transform), Without<HealingWell>>,
    mut well_q: Query<(Entity, &Transform, &mut HealingWell), Without<Player>>,
    well_sprite_q: Query<(Entity, &Parent), With<WellSprite>>,
    well_assets: Res<WellAssets>,
    mut ev_sfx: EventWriter<PlaySfxEvent>,
) {
    let Ok((mut player, p_tf)) = player_q.get_single_mut() else { return };

    let gp = gamepads.iter().next();
    let interact = keys.just_pressed(KeyCode::KeyE)
        || keys.just_pressed(KeyCode::Enter)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::South));

    for (well_entity, w_tf, mut well) in &mut well_q {
        if well.used { continue; }

        let dist = (p_tf.translation.xy() - w_tf.translation.xy()).length();
        if dist > WELL_INTERACT_RANGE { continue; }

        if interact {
            // Heal player
            player.health = (player.health + WELL_HEAL_AMOUNT).min(player.max_health);
            well.used = true;

            // Swap sprite to empty well
            for (sprite_entity, parent) in &well_sprite_q {
                if parent.get() == well_entity {
                    commands.entity(sprite_entity).insert(Sprite {
                        image: well_assets.well_empty.clone(),
                        custom_size: Some(Vec2::new(64.0, 64.0)),
                        ..default()
                    });
                }
            }

            // Play healing sound (reuse shield cast sound)
            ev_sfx.send(PlaySfxEvent(SfxType::ShieldCast));
        }
    }
}

fn well_prompt_ui(
    mut commands: Commands,
    player_q: Query<&Transform, With<Player>>,
    well_q: Query<(&Transform, &HealingWell), Without<Player>>,
    existing_prompts: Query<Entity, With<WellPrompt>>,
    font: Res<GameFont>,
) {
    // Despawn existing prompts first
    for e in &existing_prompts {
        commands.entity(e).try_despawn_recursive();
    }

    let Ok(p_tf) = player_q.get_single() else { return };

    for (w_tf, well) in &well_q {
        if well.used { continue; }

        let dist = (p_tf.translation.xy() - w_tf.translation.xy()).length();
        if dist > WELL_INTERACT_RANGE { continue; }

        // Spawn prompt above the well in world space
        commands.spawn((
            Text2d::new("Press E to drink"),
            TextFont {
                font: font.0.clone(),
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgba(0.8, 0.9, 1.0, 0.9)),
            Transform::from_xyz(w_tf.translation.x, w_tf.translation.y + 50.0, Z_HUD - 1.0),
            WellPrompt,
            PlayingEntity,
        ));
    }
}
