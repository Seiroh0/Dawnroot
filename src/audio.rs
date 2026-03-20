use bevy::prelude::*;

pub struct GameAudioPlugin;

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
    // ice_cast: m4a is not supported by Bevy's Symphonia backend — skipped
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
    IceCast, // no sound loaded — silently skipped at runtime
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
) {
    for event in ev.read() {
        let handle: Option<Handle<AudioSource>> = match event.0 {
            SfxType::MeleeHit => Some(audio_assets.melee_hit.clone()),
            SfxType::CoinPickup => Some(audio_assets.coin_pickup.clone()),
            SfxType::FireballCast => Some(audio_assets.fireball_cast.clone()),
            SfxType::ShieldCast => Some(audio_assets.shield_cast.clone()),
            SfxType::LightningCast => Some(audio_assets.lightning_cast.clone()),
            SfxType::IceCast => None, // m4a not supported
        };

        if let Some(h) = handle {
            // Add slight pitch variation to melee and coin hits for variety.
            // rand::random::<f32>() is used here because `gen` is a reserved
            // keyword in Rust 2024 edition.
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
                    ..default()
                },
            ));
        }
    }
}

fn start_bgm(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    existing: Query<&BgmMarker>,
    resuming: Option<Res<crate::ResumingFromPause>>,
) {
    // Do not restart BGM when returning from the pause menu.
    if resuming.is_some() {
        return;
    }
    // BGM is already playing (e.g. level restart without returning to title).
    if existing.iter().next().is_some() {
        return;
    }

    commands.spawn((
        AudioPlayer::<AudioSource>(audio_assets.bgm.clone()),
        PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
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
                .load("audio/sfx/368511__jofae__8-bit-fireball.mp3"),
            shield_cast: asset_server
                .load("audio/sfx/459782__metzik__deflector-shield.wav"),
            lightning_cast: asset_server
                .load("audio/sfx/652690__ayadrevis__thunder-strike.ogg"),
            // ice_cast (814954__imataco__ice-shatter-sfx.m4a) is not loaded
            // because Bevy's Symphonia backend does not support the m4a/AAC
            // container. SfxType::IceCast will be silently ignored.
            bgm: asset_server.load("audio/music/DungeonOfFate.mp3"),
        };

        app.insert_resource(assets);

        app.add_event::<PlaySfxEvent>()
            .add_systems(Update, sfx_system)
            .add_systems(OnEnter(crate::GameState::Playing), start_bgm)
            .add_systems(OnEnter(crate::GameState::Title), stop_bgm)
            .add_systems(OnEnter(crate::GameState::GameOver), stop_bgm);
    }
}
