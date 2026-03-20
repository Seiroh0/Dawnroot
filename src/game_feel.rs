use bevy::prelude::*;
use crate::GameState;

pub struct GameFeelPlugin;

impl Plugin for GameFeelPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenShakeState { trauma: 0.0, max_offset: 8.0 })
            .insert_resource(HitStop { frames_remaining: 0 })
            .add_event::<ShakeEvent>()
            .add_event::<HitStopEvent>()
            .add_systems(
                Update,
                (
                    receive_shake_events,
                    apply_hit_stop,
                    hit_flash_system,
                    update_attack_pulse,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// Screen Shake (trauma-based)
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct ScreenShakeState {
    pub trauma: f32,
    pub max_offset: f32,
}

#[derive(Event)]
pub struct ShakeEvent {
    pub trauma: f32,
}

fn receive_shake_events(
    mut ev: EventReader<ShakeEvent>,
    mut shake: ResMut<ScreenShakeState>,
) {
    for event in ev.read() {
        shake.trauma = (shake.trauma + event.trauma).min(1.0);
    }
}

/// Called by camera.rs — returns the shake offset for this frame and decays trauma.
pub fn compute_shake_offset(shake: &mut ScreenShakeState, dt: f32, time_secs: f32) -> (f32, f32) {
    if shake.trauma <= 0.001 {
        shake.trauma = 0.0;
        return (0.0, 0.0);
    }

    let t2 = shake.trauma * shake.trauma;
    let offset_x = shake.max_offset * t2 * (time_secs * 47.3).sin();
    let offset_y = shake.max_offset * t2 * (time_secs * 31.7).sin();

    // Decay trauma
    shake.trauma = (shake.trauma - 2.5 * dt).max(0.0);

    (offset_x, offset_y)
}

// ---------------------------------------------------------------------------
// Hit Stop (freeze frames)
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct HitStop {
    pub frames_remaining: u32,
}

#[derive(Event)]
pub struct HitStopEvent {
    pub frames: u32,
}

fn apply_hit_stop(
    mut ev: EventReader<HitStopEvent>,
    mut stop: ResMut<HitStop>,
    mut time: ResMut<Time<Virtual>>,
) {
    for event in ev.read() {
        stop.frames_remaining = stop.frames_remaining.max(event.frames);
    }

    if stop.frames_remaining > 0 {
        time.set_relative_speed(0.05);
        stop.frames_remaining -= 1;
    } else {
        time.set_relative_speed(1.0);
    }
}

// ---------------------------------------------------------------------------
// Hit Flash (white flash on damaged entities)
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    pub original_color: Color,
}

fn hit_flash_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut sprite, mut flash) in &mut query {
        flash.timer -= dt;
        if flash.timer > 0.05 {
            sprite.color = Color::WHITE;
        } else if flash.timer > 0.0 {
            sprite.color = flash.original_color;
        } else {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

// ---------------------------------------------------------------------------
// Attack pulse (scale pop when no attack spritesheet available)
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct AttackPulse {
    pub timer: f32,
    pub duration: f32,
}

fn update_attack_pulse(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut AttackPulse)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut pulse) in &mut query {
        pulse.timer += dt;
        let t = (pulse.timer / pulse.duration).clamp(0.0, 1.0);

        // Scale pop: 1.0 → 1.3 → 1.0
        let scale = if t < 0.5 {
            1.0 + 0.3 * (t / 0.5)
        } else {
            1.3 - 0.3 * ((t - 0.5) / 0.5)
        };

        // Only modify x/y scale, preserve z and sign for facing
        let sign_x = tf.scale.x.signum();
        tf.scale.x = sign_x * scale;
        tf.scale.y = scale;

        if t >= 1.0 {
            let sign_x = tf.scale.x.signum();
            tf.scale = Vec3::new(sign_x, 1.0, 1.0);
            commands.entity(entity).remove::<AttackPulse>();
        }
    }
}
