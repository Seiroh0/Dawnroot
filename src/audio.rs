use bevy::prelude::*;
use crate::room::RoomState;

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
    pub player_death: Handle<AudioSource>,
    pub bgm_dungeon: Handle<AudioSource>,
    pub bgm_shop: Handle<AudioSource>,
    pub bgm_boss: Handle<AudioSource>,
}

// ---------------------------------------------------------------------------
// SFX events
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SfxType {
    MeleeHit,
    CoinPickup,
    FireballCast,
    ShieldCast,
    LightningCast,
    IceCast,
    PlayerDeath,
    EnemyDeath,
    Jump,
    LevelComplete,
    RelicPickup,
    ShopBuy,
}

#[derive(Event)]
pub struct PlaySfxEvent(pub SfxType);

// ---------------------------------------------------------------------------
// BGM multi-track system
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BgmTrack {
    Dungeon,
    Shop,
    Boss,
    None,
}

#[derive(Component)]
struct BgmMarker {
    track: BgmTrack,
}

/// Tracks the currently playing BGM so we only switch when needed.
#[derive(Resource)]
struct CurrentBgm {
    track: BgmTrack,
}

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
        // Resolve handle and optional speed override for fallback sounds
        let (handle, speed_override): (Option<Handle<AudioSource>>, Option<f32>) = match event.0 {
            SfxType::MeleeHit => (Some(audio_assets.melee_hit.clone()), None),
            SfxType::CoinPickup => (Some(audio_assets.coin_pickup.clone()), None),
            SfxType::FireballCast => (Some(audio_assets.fireball_cast.clone()), None),
            SfxType::ShieldCast => (Some(audio_assets.shield_cast.clone()), None),
            SfxType::LightningCast => (Some(audio_assets.lightning_cast.clone()), None),
            SfxType::PlayerDeath => (Some(audio_assets.player_death.clone()), None),
            // Fallbacks using existing assets with pitch variation
            SfxType::EnemyDeath => (Some(audio_assets.melee_hit.clone()), Some(1.5)),
            SfxType::Jump => (Some(audio_assets.coin_pickup.clone()), Some(0.6)),
            SfxType::LevelComplete => (Some(audio_assets.shield_cast.clone()), Some(1.0)),
            SfxType::RelicPickup => (Some(audio_assets.coin_pickup.clone()), Some(1.3)),
            SfxType::ShopBuy => (Some(audio_assets.coin_pickup.clone()), Some(1.0)),
            SfxType::IceCast => (None, None), // m4a not supported
        };

        if let Some(h) = handle {
            let load_state = asset_server.get_load_state(&h);
            let is_loaded = matches!(load_state, Some(bevy::asset::LoadState::Loaded));
            if !is_loaded {
                continue;
            }

            let vol = settings.sfx_volume * settings.master_volume;
            let speed = speed_override.unwrap_or_else(|| match event.0 {
                SfxType::MeleeHit | SfxType::CoinPickup => {
                    0.9 + rand::random::<f32>() * 0.2
                }
                _ => 1.0,
            });

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

/// Determine which BGM track should play based on game state.
fn determine_bgm_track(
    room_state: Option<&RoomState>,
    shop_state: Option<&crate::shop::ShopUiState>,
) -> BgmTrack {
    // Shop overlay active → shop music
    if let Some(shop) = shop_state {
        if shop.active {
            return BgmTrack::Shop;
        }
    }

    // Check room type
    if let Some(rs) = room_state {
        match rs.current_type {
            crate::room::RoomType::Boss => return BgmTrack::Boss,
            crate::room::RoomType::Shop => return BgmTrack::Shop,
            _ => {}
        }
    }

    BgmTrack::Dungeon
}

fn bgm_track_handle(track: BgmTrack, assets: &AudioAssets) -> Option<Handle<AudioSource>> {
    match track {
        BgmTrack::Dungeon => Some(assets.bgm_dungeon.clone()),
        BgmTrack::Shop => Some(assets.bgm_shop.clone()),
        BgmTrack::Boss => Some(assets.bgm_boss.clone()),
        BgmTrack::None => None,
    }
}

fn start_bgm(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    existing: Query<&BgmMarker>,
    resuming: Option<Res<crate::ResumingFromPause>>,
    room_state: Res<RoomState>,
    shop_state: Option<Res<crate::shop::ShopUiState>>,
) {
    if resuming.is_some() { return; }
    if existing.iter().next().is_some() { return; }

    let track = determine_bgm_track(Some(&room_state), shop_state.as_deref());
    let Some(handle) = bgm_track_handle(track, &audio_assets) else { return };

    // Guard: only play if loaded
    let load_state = asset_server.get_load_state(&handle);
    if !matches!(load_state, Some(bevy::asset::LoadState::Loaded)) {
        return;
    }

    let vol = settings.music_volume * settings.master_volume;
    commands.spawn((
        AudioPlayer::<AudioSource>(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(vol),
            ..default()
        },
        BgmMarker { track },
    ));
    commands.insert_resource(CurrentBgm { track });
}

fn stop_bgm(mut commands: Commands, bgm_q: Query<Entity, With<BgmMarker>>) {
    for entity in &bgm_q {
        commands.entity(entity).try_despawn_recursive();
    }
    commands.insert_resource(CurrentBgm { track: BgmTrack::None });
}

/// Switch BGM tracks when room type or shop state changes during gameplay.
fn switch_bgm_on_context(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    asset_server: Res<AssetServer>,
    settings: Res<AudioSettings>,
    bgm_q: Query<(Entity, &BgmMarker)>,
    current: Option<Res<CurrentBgm>>,
    room_state: Res<RoomState>,
    shop_state: Option<Res<crate::shop::ShopUiState>>,
) {
    let desired = determine_bgm_track(Some(&room_state), shop_state.as_deref());
    let current_track = current.as_ref().map(|c| c.track).unwrap_or(BgmTrack::None);

    if desired == current_track { return; }

    // Stop old BGM
    for (entity, _) in &bgm_q {
        commands.entity(entity).try_despawn_recursive();
    }

    // Start new track
    let Some(handle) = bgm_track_handle(desired, &audio_assets) else {
        commands.insert_resource(CurrentBgm { track: BgmTrack::None });
        return;
    };

    let load_state = asset_server.get_load_state(&handle);
    if !matches!(load_state, Some(bevy::asset::LoadState::Loaded)) {
        // Asset not loaded yet — set track to None, will retry next frame
        commands.insert_resource(CurrentBgm { track: BgmTrack::None });
        return;
    }

    let vol = settings.music_volume * settings.master_volume;
    commands.spawn((
        AudioPlayer::<AudioSource>(handle),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(vol),
            ..default()
        },
        BgmMarker { track: desired },
    ));
    commands.insert_resource(CurrentBgm { track: desired });
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
            player_death: asset_server
                .load("audio/791967__zy7__player-death-via-endless-pit.mp3"),
            bgm_dungeon: asset_server.load("audio/music/DungeonOfFate.mp3"),
            bgm_shop: asset_server.load("audio/629170__holizna__chill-lofi-epiano-loop-80-bpm.wav"),
            bgm_boss: asset_server.load("audio/346200__levelclearer__battle.wav"),
        };

        app.insert_resource(assets);
        app.insert_resource(load_audio_settings());
        app.insert_resource(CurrentBgm { track: BgmTrack::None });

        app.add_event::<PlaySfxEvent>()
            .add_systems(Update, (sfx_system, update_bgm_volume, switch_bgm_on_context).run_if(in_state(crate::GameState::Playing)))
            .add_systems(OnEnter(crate::GameState::Playing), start_bgm)
            .add_systems(OnEnter(crate::GameState::Title), stop_bgm)
            .add_systems(OnEnter(crate::GameState::GameOver), stop_bgm);
    }
}
