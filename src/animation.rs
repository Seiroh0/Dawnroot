use bevy::prelude::*;
use crate::GameState;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            animate_sprites.run_if(in_state(GameState::Playing)),
        );
    }
}

/// Generic frame-based sprite animation (for future spritesheet support)
#[derive(Component)]
pub struct SpriteAnimation {
    pub frame: usize,
    pub timer: f32,
    pub frame_duration: f32,
    pub frame_count: usize,
}

impl Default for SpriteAnimation {
    fn default() -> Self {
        Self { frame: 0, timer: 0.0, frame_duration: 0.12, frame_count: 4 }
    }
}

fn animate_sprites(
    mut query: Query<&mut SpriteAnimation>,
    time: Res<Time>,
) {
    for mut anim in &mut query {
        anim.timer += time.delta_secs();
        if anim.timer >= anim.frame_duration {
            anim.timer -= anim.frame_duration;
            anim.frame = (anim.frame + 1) % anim.frame_count;
        }
    }
}
