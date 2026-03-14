use bevy::prelude::*;
use bevy::audio::{PlaybackMode, Volume};
use crate::{
    GameState,
    player::{PlayerAttack, PlayerDamaged, PlayerDied, PlayerDashed, PlayerLanded},
    enemy::EnemyDefeated,
    spell::SpellCast,
    room::RoomCleared,
};

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BgmHandle::default())
            .add_systems(OnEnter(GameState::Playing), start_bgm)
            .add_systems(OnExit(GameState::Playing), stop_bgm)
            .add_systems(OnExit(GameState::GameOver), stop_bgm)
            .add_systems(
                Update,
                (
                    sfx_player_attack,
                    sfx_player_damaged,
                    sfx_player_died,
                    sfx_player_dash,
                    sfx_player_land,
                    sfx_enemy_defeated,
                    sfx_spell_cast,
                    sfx_room_cleared,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ---------------------------------------------------------------------------
// BGM
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct BgmHandle {
    entity: Option<Entity>,
}

fn start_bgm(
    mut commands: Commands,
    mut bgm: ResMut<BgmHandle>,
    asset_server: Res<AssetServer>,
) {
    // Stop old BGM if any
    if let Some(e) = bgm.entity.take() {
        commands.entity(e).despawn_recursive();
    }

    let entity = commands.spawn((
        AudioPlayer::<AudioSource>(asset_server.load("audio/GAME_SOUND.mp3")),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::new(0.35),
            ..default()
        },
    )).id();
    bgm.entity = Some(entity);
}

fn stop_bgm(
    mut commands: Commands,
    mut bgm: ResMut<BgmHandle>,
) {
    if let Some(e) = bgm.entity.take() {
        commands.entity(e).despawn_recursive();
    }
}

// ---------------------------------------------------------------------------
// Procedural SFX generation
// ---------------------------------------------------------------------------

fn make_sfx(
    audio_sources: &mut Assets<AudioSource>,
    frequency: f32,
    duration_secs: f32,
    volume: f32,
    decay: bool,
    noise_mix: f32,
) -> Handle<AudioSource> {
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(num_samples * 2); // stereo i16

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let env = if decay {
            (1.0 - t / duration_secs).max(0.0)
        } else {
            1.0
        };

        // Simple sine wave + optional noise
        let sine = (t * frequency * std::f32::consts::TAU).sin();
        let noise = if noise_mix > 0.0 {
            // Simple pseudo-noise using sin of large frequency
            (t * 7919.0 * std::f32::consts::TAU).sin() * 0.5
                + (t * 13397.0 * std::f32::consts::TAU).sin() * 0.3
                + (t * 23311.0 * std::f32::consts::TAU).sin() * 0.2
        } else {
            0.0
        };

        let sample = (sine * (1.0 - noise_mix) + noise * noise_mix) * env * volume;
        let s = (sample * 32000.0).clamp(-32000.0, 32000.0) as i16;
        // Stereo: same sample both channels
        samples.push(s);
        samples.push(s);
    }

    // Build WAV in memory
    let wav_data = build_wav(sample_rate, 2, &samples);
    let source = AudioSource { bytes: wav_data.into() };
    audio_sources.add(source)
}

fn build_wav(sample_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;
    let byte_rate = sample_rate * channels as u32 * 2;
    let block_align = channels * 2;

    let mut buf = Vec::with_capacity(file_size as usize + 8);
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    buf.extend_from_slice(&1u16.to_le_bytes());  // PCM
    buf.extend_from_slice(&channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}

// ---------------------------------------------------------------------------
// SFX cache – generated once, reused
// ---------------------------------------------------------------------------

#[derive(Resource)]
struct SfxCache {
    attack: Handle<AudioSource>,
    damaged: Handle<AudioSource>,
    died: Handle<AudioSource>,
    dash: Handle<AudioSource>,
    land: Handle<AudioSource>,
    enemy_kill: Handle<AudioSource>,
    spell_fire: Handle<AudioSource>,
    spell_ice: Handle<AudioSource>,
    spell_lightning: Handle<AudioSource>,
    spell_shield: Handle<AudioSource>,
    room_clear: Handle<AudioSource>,
}

fn get_or_create_sfx(
    commands: &mut Commands,
    cache: Option<Res<SfxCache>>,
    audio_sources: &mut Assets<AudioSource>,
) -> SfxCache {
    if let Some(c) = cache {
        return SfxCache {
            attack: c.attack.clone(),
            damaged: c.damaged.clone(),
            died: c.died.clone(),
            dash: c.dash.clone(),
            land: c.land.clone(),
            enemy_kill: c.enemy_kill.clone(),
            spell_fire: c.spell_fire.clone(),
            spell_ice: c.spell_ice.clone(),
            spell_lightning: c.spell_lightning.clone(),
            spell_shield: c.spell_shield.clone(),
            room_clear: c.room_clear.clone(),
        };
    }

    let cache = SfxCache {
        // Sword slash: short high-freq noise burst
        attack: make_sfx(audio_sources, 800.0, 0.08, 0.4, true, 0.7),
        // Hit: mid-freq thud
        damaged: make_sfx(audio_sources, 200.0, 0.15, 0.5, true, 0.3),
        // Death: low descending
        died: make_sfx(audio_sources, 120.0, 0.5, 0.6, true, 0.2),
        // Dash: whoosh
        dash: make_sfx(audio_sources, 400.0, 0.12, 0.3, true, 0.8),
        // Land: soft thud
        land: make_sfx(audio_sources, 150.0, 0.06, 0.2, true, 0.4),
        // Enemy kill: satisfying pop
        enemy_kill: make_sfx(audio_sources, 600.0, 0.1, 0.35, true, 0.3),
        // Fireball: warm whoosh
        spell_fire: make_sfx(audio_sources, 350.0, 0.2, 0.4, true, 0.5),
        // Ice: sharp crystalline
        spell_ice: make_sfx(audio_sources, 1200.0, 0.15, 0.3, true, 0.4),
        // Lightning: electric crack
        spell_lightning: make_sfx(audio_sources, 80.0, 0.25, 0.5, true, 0.6),
        // Shield: resonant hum
        spell_shield: make_sfx(audio_sources, 250.0, 0.3, 0.3, false, 0.1),
        // Room clear: victory chime (high)
        room_clear: make_sfx(audio_sources, 880.0, 0.3, 0.4, true, 0.0),
    };

    commands.insert_resource(SfxCache {
        attack: cache.attack.clone(),
        damaged: cache.damaged.clone(),
        died: cache.died.clone(),
        dash: cache.dash.clone(),
        land: cache.land.clone(),
        enemy_kill: cache.enemy_kill.clone(),
        spell_fire: cache.spell_fire.clone(),
        spell_ice: cache.spell_ice.clone(),
        spell_lightning: cache.spell_lightning.clone(),
        spell_shield: cache.spell_shield.clone(),
        room_clear: cache.room_clear.clone(),
    });

    cache
}

fn play_sfx(commands: &mut Commands, handle: &Handle<AudioSource>) {
    commands.spawn((
        AudioPlayer::<AudioSource>(handle.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(0.6),
            ..default()
        },
    ));
}

// ---------------------------------------------------------------------------
// SFX listener systems
// ---------------------------------------------------------------------------

fn sfx_player_attack(
    mut commands: Commands,
    mut ev: EventReader<PlayerAttack>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {}); // drain
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.attack);
}

fn sfx_player_damaged(
    mut commands: Commands,
    mut ev: EventReader<PlayerDamaged>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.damaged);
}

fn sfx_player_died(
    mut commands: Commands,
    mut ev: EventReader<PlayerDied>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.died);
}

fn sfx_player_dash(
    mut commands: Commands,
    mut ev: EventReader<PlayerDashed>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.dash);
}

fn sfx_player_land(
    mut commands: Commands,
    mut ev: EventReader<PlayerLanded>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.land);
}

fn sfx_enemy_defeated(
    mut commands: Commands,
    mut ev: EventReader<EnemyDefeated>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.enemy_kill);
}

fn sfx_spell_cast(
    mut commands: Commands,
    mut ev: EventReader<SpellCast>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    // Collect first event's spell type, then drain
    let spell = {
        let mut spell_id = None;
        for event in ev.read() {
            if spell_id.is_none() {
                spell_id = Some(event.spell);
            }
        }
        spell_id
    };
    let Some(spell) = spell else { return; };

    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    let handle = match spell {
        crate::spell::SpellId::Fireball => &c.spell_fire,
        crate::spell::SpellId::IceShards => &c.spell_ice,
        crate::spell::SpellId::Lightning => &c.spell_lightning,
        crate::spell::SpellId::Shield => &c.spell_shield,
    };
    play_sfx(&mut commands, handle);
}

fn sfx_room_cleared(
    mut commands: Commands,
    mut ev: EventReader<RoomCleared>,
    cache: Option<Res<SfxCache>>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    if ev.read().next().is_none() { return; }
    ev.read().for_each(|_| {});
    let c = get_or_create_sfx(&mut commands, cache, &mut audio_sources);
    play_sfx(&mut commands, &c.room_clear);
}
