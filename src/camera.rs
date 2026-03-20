use bevy::prelude::*;
use crate::{constants::*, GameState, player::Player, game_feel::{ScreenShakeState, compute_shake_offset}};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_camera)
            .add_systems(
                Update,
                follow_player.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct GameCamera {
    pub base_x: f32,
    pub base_y: f32,
}

// Keep the old ScreenShake component for backwards compatibility with
// combat.rs trigger_shake calls — it feeds into the trauma system.
#[derive(Component)]
pub struct ScreenShake {
    pub strength: f32,
    pub timer: f32,
}

fn spawn_camera(mut commands: Commands, existing: Query<&GameCamera>) {
    // Don't spawn a second camera when resuming from pause
    if existing.iter().next().is_some() { return; }

    commands.spawn((
        Camera2d,
        GameCamera {
            base_x: ROOM_W / 2.0,
            base_y: ROOM_H / 2.0,
        },
        ScreenShake {
            strength: 0.0,
            timer: 0.0,
        },
        // NOTE: No PlayingEntity — camera persists across Playing/Paused cycle.
        // Cleaned up explicitly when leaving gameplay (OnEnter Title/GameOver).
    ));
}

fn follow_player(
    player_q: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut cam_q: Query<(&mut Transform, &mut GameCamera, &mut ScreenShake)>,
    time: Res<Time>,
    mut shake_state: ResMut<ScreenShakeState>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let Ok((mut cam_tf, mut cam, mut shake)) = cam_q.get_single_mut() else { return };

    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();

    // Smooth camera follow with lerp (8.0 = responsive but not instant)
    let lerp_speed = (8.0 * dt).clamp(0.0, 1.0);

    // Target: follow player with slight lookahead
    let target_x = player_tf.translation.x;
    // Vertical offset: look down when falling to show more of the fall
    let fall_offset = player_q
        .get_single()
        .ok()
        .map(|_| 0.0) // We don't have velocity access here directly
        .unwrap_or(0.0);
    let target_y = player_tf.translation.y + 40.0 + fall_offset;

    cam.base_x += (target_x - cam.base_x) * lerp_speed;
    cam.base_y += (target_y - cam.base_y) * lerp_speed;

    // Clamp camera to room bounds
    let half_vw = VIEWPORT_W / 2.0;
    let half_vh = VIEWPORT_H / 2.0;

    let cam_x = if ROOM_W <= VIEWPORT_W {
        ROOM_W / 2.0
    } else {
        cam.base_x.clamp(half_vw, ROOM_W - half_vw)
    };

    let cam_y = if ROOM_H <= VIEWPORT_H {
        ROOM_H / 2.0
    } else {
        cam.base_y.clamp(half_vh, ROOM_H - half_vh)
    };

    // Convert old ScreenShake component to trauma (backwards compat with trigger_shake)
    if shake.timer > 0.0 {
        shake.timer = (shake.timer - dt).max(0.0);
        // Convert strength to trauma: rough mapping
        let trauma_add = (shake.strength * 0.03).min(0.3);
        shake_state.trauma = (shake_state.trauma + trauma_add * dt * 60.0).min(1.0);
        shake.strength *= (-12.0 * dt).exp();
    }

    // Compute trauma-based screen shake offset
    let (shake_x, shake_y) = compute_shake_offset(&mut shake_state, dt, elapsed);

    cam_tf.translation.x = cam_x + shake_x;
    cam_tf.translation.y = cam_y + shake_y;
}

pub fn trigger_shake(shake: &mut ScreenShake, strength: f32, duration: f32) {
    shake.strength = strength;
    shake.timer = duration;
}
