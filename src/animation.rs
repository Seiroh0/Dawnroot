use bevy::prelude::*;
use crate::{GameState, player::Player};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                animate_sprites,
                player_animation_controller,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AnimState {
    Idle,
    Run,
    Jump,
    Fall,
    Attack,
    Dash,
}

#[derive(Component)]
pub struct SpriteAnimation {
    pub state: AnimState,
    pub frame: usize,
    pub timer: f32,
    pub frame_duration: f32,
    pub frame_count: usize,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self {
            state: AnimState::Idle,
            frame: 0,
            timer: 0.0,
            frame_duration: 0.12,
            frame_count: 4,
        }
    }
}

fn animate_sprites(
    mut query: Query<(&mut SpriteAnimation, &mut Sprite)>,
    time: Res<Time>,
) {
    for (mut anim, mut sprite) in &mut query {
        anim.timer += time.delta_secs();
        if anim.timer >= anim.frame_duration {
            anim.timer -= anim.frame_duration;
            anim.frame = (anim.frame + 1) % anim.frame_count;
        }

        // Color-based animation feedback (until real sprites are loaded)
        let base = match anim.state {
            AnimState::Idle => Color::srgb(0.2, 0.7, 0.3),
            AnimState::Run => Color::srgb(0.25, 0.75, 0.35),
            AnimState::Jump => Color::srgb(0.3, 0.8, 0.4),
            AnimState::Fall => Color::srgb(0.15, 0.6, 0.25),
            AnimState::Attack => Color::srgb(0.9, 0.7, 0.2),
            AnimState::Dash => Color::srgb(0.4, 0.9, 0.5),
        };
        sprite.color = base;
    }
}

fn player_animation_controller(
    mut query: Query<(&Player, &mut SpriteAnimation, &mut Transform)>,
) {
    for (player, mut anim, mut tf) in &mut query {
        let new_state = if player.is_dashing {
            AnimState::Dash
        } else if player.melee_cooldown > 0.2 {
            AnimState::Attack
        } else if !player.is_on_floor && player.vy > 0.0 {
            AnimState::Jump
        } else if !player.is_on_floor && player.vy <= 0.0 {
            AnimState::Fall
        } else if player.vx.abs() > 10.0 {
            AnimState::Run
        } else {
            AnimState::Idle
        };

        if new_state != anim.state {
            anim.state = new_state;
            anim.frame = 0;
            anim.timer = 0.0;
        }

        // Flip sprite based on facing
        let scale_x = if player.facing < 0.0 { -1.0 } else { 1.0 };
        tf.scale.x = scale_x;
    }
}
