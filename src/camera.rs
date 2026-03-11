use bevy::prelude::*;
use crate::{constants::*, GameState, PlayingEntity, player::Player};

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

#[derive(Component)]
pub struct ScreenShake {
    pub strength: f32,
    pub timer: f32,
}

fn spawn_camera(mut commands: Commands) {
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
        PlayingEntity,
    ));
}

fn follow_player(
    player_q: Query<&Transform, (With<Player>, Without<GameCamera>)>,
    mut cam_q: Query<(&mut Transform, &mut GameCamera, &mut ScreenShake)>,
    time: Res<Time>,
) {
    let Ok(player_tf) = player_q.get_single() else { return };
    let Ok((mut cam_tf, mut cam, mut shake)) = cam_q.get_single_mut() else { return };

    let dt = time.delta_secs();
    let speed = (dt * 5.0).clamp(0.0, 1.0);

    // Target: follow player with slight lookahead
    let target_x = player_tf.translation.x;
    let target_y = player_tf.translation.y + 40.0;

    cam.base_x += (target_x - cam.base_x) * speed;
    cam.base_y += (target_y - cam.base_y) * speed;

    // Clamp camera to room bounds
    let half_vw = VIEWPORT_W / 2.0;
    let half_vh = VIEWPORT_H / 2.0;

    // If room fits in viewport, center it. Otherwise clamp.
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

    // Screen shake
    let mut shake_x = 0.0;
    let mut shake_y = 0.0;
    if shake.timer > 0.0 {
        shake.timer = (shake.timer - dt).max(0.0);
        shake.strength *= (-12.0 * dt).exp();
        shake_x = (rand::random::<f32>() * 2.0 - 1.0) * shake.strength;
        shake_y = (rand::random::<f32>() * 2.0 - 1.0) * shake.strength;
    }

    cam_tf.translation.x = cam_x + shake_x;
    cam_tf.translation.y = cam_y + shake_y;
}

pub fn trigger_shake(shake: &mut ScreenShake, strength: f32, duration: f32) {
    shake.strength = strength;
    shake.timer = duration;
}
