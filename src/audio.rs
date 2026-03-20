use bevy::prelude::*;

pub struct GameAudioPlugin;

// ---------------------------------------------------------------------------
// Audio settings (persisted to dawnroot_settings.json)
// ---------------------------------------------------------------------------

#[derive(Resource, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.5,
            sfx_volume: 0.7,
            music_volume: 0.4,
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    std::path::PathBuf::from("dawnroot_settings.json")
}

pub fn save_audio_settings(settings: &AudioSettings) {
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(settings_path(), json);
    }
}

fn load_audio_settings() -> AudioSettings {
    if let Ok(data) = std::fs::read_to_string(settings_path()) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        AudioSettings::default()
    }
}

// ---------------------------------------------------------------------------
// Assets
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct AudioAssets {
    pub melee_hit: Handle<AudioSource>,
    pub coin_pickup: Handle<AudioSource>,
    pub fireball_cast: Handle<AudioSource>,
    pub shield_cast: Handle<AudioSource>,
    pub lightning_cast: Handle<AudioSource>,
    pub bgm: Handle<AudioSource>,
}

// ---------------------------------------------------------------------------
// SFX events
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SfxType {
    MeleeHit,
    CoinPickup,
    FireballCast,
    ShieldCast,
    LightningCast,
    IceCast,
}

#[derive(Event)]
pub struct PlaySfxEvent(pub SfxType);

// ---------------------------------------------------------------------------
// BGM marker
// ---------------------------------------------------------------------------

#[derive(Component)]
struct BgmMarker;

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn sfx_system(
    mut commands: Commands,
    mut ev: EventReader<PlaySfxEvent>,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
) {
    for event in ev.read() {
        let handle: Option<Handle<AudioSource>> = match event.0 {
            SfxType::MeleeHit => Some(audio_assets.melee_hit.clone()),
            SfxType::CoinPickup => Some(audio_assets.coin_pickup.clone()),
            SfxType::FireballCast => Some(audio_assets.fireball_cast.clone()),
            SfxType::ShieldCast => Some(audio_assets.shield_cast.clone()),
            SfxType::LightningCast => Some(audio_assets.lightning_cast.clone()),
            SfxType::IceCast => {
                // m4a not supported by Bevy — skip silently
                None
            }
        };

        if let Some(h) = handle {
            // Guard: only play if the asset has actually loaded successfully
            let load_state = asset_server.get_load_state(&h);
            let is_loaded = matches!(load_state, Some(bevy::asset::LoadState::Loaded));
            if !is_loaded {
                warn!("SFX {:?} asset not yet loaded, skipping playback", event.0 as u8);
                continue;
            }

            let vol = settings.sfx_volume * settings.master_volume;
            let speed = match event.0 {
                SfxType::MeleeHit | SfxType::CoinPickup => {
                    0.9 + rand::random::<f32>() * 0.2
                }
                _ => 1.0,
            };

            commands.spawn((
                AudioPlayer::<AudioSource>(h),
                PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    speed,
                    volume: bevy::audio::Volume::new(vol),
                    ..default()
                },
            ));
        }
    }
}

fn start_bgm(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    existing: Query<&BgmMarker>,
    resuming: Option<Res<crate::ResumingFromPause>>,
) {
    if resuming.is_some() { return; }
    if existing.iter().next().is_some() { return; }

    // Guard: only play if loaded
    let load_state = asset_server.get_load_state(&audio_assets.bgm);
    if !matches!(load_state, Some(bevy::asset::LoadState::Loaded)) {
        warn!("BGM asset not yet loaded, skipping");
        return;
    }

    let vol = settings.music_volume * settings.master_volume;
    commands.spawn((
        AudioPlayer::<AudioSource>(audio_assets.bgm.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(vol),
            ..default()
        },
        BgmMarker,
    ));
}

fn stop_bgm(mut commands: Commands, bgm_q: Query<Entity, With<BgmMarker>>) {
    for entity in &bgm_q {
        commands.entity(entity).try_despawn_recursive();
    }
}

/// Live-update BGM volume when AudioSettings changes.
fn update_bgm_volume(
    settings: Res<AudioSettings>,
    mut bgm_q: Query<&mut PlaybackSettings, With<BgmMarker>>,
) {
    if !settings.is_changed() { return; }
    let vol = settings.music_volume * settings.master_volume;
    for mut pb in &mut bgm_q {
        pb.volume = bevy::audio::Volume::new(vol);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world().resource::<AssetServer>();

        // Use .ogg for fireball instead of potentially corrupt .mp3
        // The thunder-strike.ogg works reliably and sounds good for fire spells
        let assets = AudioAssets {
            melee_hit: asset_server
                .load("audio/sfx/547042__cogfirestudios__hit-impact-sword-3.wav"),
            coin_pickup: asset_server
                .load("audio/sfx/347174__davidsraba__coin-pickup-sound-v-0.wav"),
            fireball_cast: asset_server
                .load("audio/sfx/652690__ayadrevis__thunder-strike.ogg"),
            shield_cast: asset_server
                .load("audio/sfx/459782__metzik__deflector-shield.wav"),
            lightning_cast: asset_server
                .load("audio/sfx/652690__ayadrevis__thunder-strike.ogg"),
            bgm: asset_server.load("audio/music/DungeonOfFate.mp3"),
        };

        app.insert_resource(assets);
        app.insert_resource(load_audio_settings());

        app.add_event::<PlaySfxEvent>()
            .add_systems(Update, (sfx_system, update_bgm_volume))
            .add_systems(OnEnter(crate::GameState::Playing), start_bgm)
            .add_systems(OnEnter(crate::GameState::Title), stop_bgm)
            .add_systems(OnEnter(crate::GameState::GameOver), stop_bgm);
    }
}
