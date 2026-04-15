use bevy::prelude::*;
use crate::{GameState, GameFont, constants::*, ActiveSaveSlot, LoadedSave, load_slot, delete_slot, player::PlayerSpriteAssets, MetaProgression};
use crate::audio::AudioSettings;
use crate::pause_menu::{SettingsPanel, SettingsState, spawn_settings_panel, settings_input};

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), setup_title)
            .add_systems(OnExit(GameState::Title), cleanup_title)
            .add_systems(
                Update,
                (
                    handle_title_input,
                    handle_slot_input,
                    handle_title_settings_input,
                    settings_input,
                    parallax_mouse_tracking,
                    pulse_text_alpha,
                    well_bob_animate,
                    well_glow_animate,
                    well_particle_emit,
                    update_well_particles,
                    tree_sway_animate,
                    star_twinkle_animate,
                    fade_in_delay_system,
                    update_falling_leaves,
                )
                    .run_if(in_state(GameState::Title)),
            )
            .add_systems(OnEnter(GameState::WellIntro), setup_well_intro)
            .add_systems(OnExit(GameState::WellIntro), cleanup_well_intro)
            .add_systems(
                Update,
                (
                    update_well_intro,
                    update_falling_particles,
                    update_intro_afterimages,
                    update_intro_dust,
                    intro_camera_shake,
                    animate_intro_sprite,
                    tree_sway_animate,
                    star_twinkle_animate,
                    well_glow_animate,
                    well_bob_animate,
                    update_falling_leaves,
                )
                    .run_if(in_state(GameState::WellIntro)),
            );
    }
}

#[derive(Component)]
struct TitleEntity;

#[derive(Component)]
struct PromptText;

#[derive(Component)]
struct SlotUI;

#[derive(Resource)]
struct SlotMenuState {
    open: bool,
}

// ── Module 1: Parallax ───────────────────────────────────────────

/// Entities with this component shift based on mouse position for depth.
#[derive(Component)]
struct ParallaxLayer {
    depth: f32,
    base_x: f32,
}

// ── Module 2: Pulsing Alpha ──────────────────────────────────────

#[derive(Component)]
struct PulsingAlpha {
    speed: f32,
    min_alpha: f32,
    max_alpha: f32,
}

// ── Module 3: Well Effects ───────────────────────────────────────

/// Gentle bobbing animation for the well sprite on the title screen.
#[derive(Component)]
struct WellBob {
    timer: f32,
    base_y: f32,
}

#[derive(Component)]
struct WellGlow {
    timer: f32,
}

#[derive(Component)]
struct WellParticle {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

#[derive(Resource)]
struct WellParticleTimer {
    timer: f32,
    well_x: f32,
    well_y: f32,
}

// ── Module 4: Intro Juice ────────────────────────────────────────

// ── Falling autumn leaves ────────────────────────────────────────

#[derive(Component)]
struct FallingLeaf {
    vx: f32,
    vy: f32,
    spin: f32,
    sway_phase: f32,
    lifetime: f32,
    max_lifetime: f32,
}

// ── Module 5: Tree Sway ──────────────────────────────────────────

#[derive(Component)]
struct TreeSway {
    speed: f32,
    phase: f32,
    base_angle: f32,
}

// ── Module 6: Twinkling Stars ────────────────────────────────────

#[derive(Component)]
struct TwinklingStar {
    speed: f32,
    phase: f32,
    base_alpha: f32,
}

// ── Module 7: Fade-in Delay ──────────────────────────────────────

#[derive(Component)]
struct FadeInDelay {
    delay: f32,
    elapsed: f32,
    fade_duration: f32,
    target_alpha: f32,
}

#[derive(Component)]
struct IntroAfterimage {
    lifetime: f32,
    max_lifetime: f32,
}

#[derive(Component)]
struct IntroDustPuff {
    vx: f32,
    vy: f32,
    lifetime: f32,
    max_lifetime: f32,
}

// ── Title screen ──────────────────────────────────────────────────

fn setup_title(
    mut commands: Commands,
    font: Res<GameFont>,
    meta: Res<MetaProgression>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((Camera2d, TitleEntity));
    commands.insert_resource(SlotMenuState { open: false });

    let mut rng = rand::thread_rng();
    use rand::Rng;

    // Night sky
    commands.spawn((
        Sprite {
            color: Color::srgb(0.06, 0.03, 0.02),
            custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND),
        TitleEntity,
    ));

    // Stars (parallax: very far back) — 110 stars with size weighting + twinkle
    for _ in 0..110 {
        let x = rng.gen_range(-VIEWPORT_W / 2.0..VIEWPORT_W / 2.0);
        let y = rng.gen_range(-20.0..VIEWPORT_H / 2.0);
        let b = rng.gen_range(0.25..0.95_f32);
        // Weighted size: 60% at 1px, 25% at 2px, 15% at 3px
        let roll = rng.gen_range(0.0..1.0_f32);
        let sz: f32 = if roll < 0.60 { 1.0 } else if roll < 0.85 { 2.0 } else { 3.0 };
        let speed = rng.gen_range(0.4..1.6_f32);
        let phase = rng.gen_range(0.0..std::f32::consts::TAU);
        let base_alpha = b;
        commands.spawn((
            Sprite {
                color: Color::srgba(b, b * 0.9, b * 0.7, base_alpha),
                custom_size: Some(Vec2::new(sz, sz)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 0.5),
            TitleEntity,
            ParallaxLayer { depth: 0.1, base_x: x },
            TwinklingStar { speed, phase, base_alpha },
        ));
    }

    // Moon (parallax: far) — sprite with soft glow halos
    let moon_x = 280.0;
    let moon_y = 180.0;
    // Outer softest glow
    commands.spawn((
        Sprite {
            color: Color::srgba(0.9, 0.88, 0.7, 0.04),
            custom_size: Some(Vec2::new(130.0, 130.0)),
            ..default()
        },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.6),
        TitleEntity,
        ParallaxLayer { depth: 0.15, base_x: moon_x },
    ));
    // Inner glow
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.97, 0.8, 0.07),
            custom_size: Some(Vec2::new(88.0, 88.0)),
            ..default()
        },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.7),
        TitleEntity,
        ParallaxLayer { depth: 0.15, base_x: moon_x },
    ));
    // Moon sprite
    commands.spawn((
        Sprite {
            image: asset_server.load("moon.png"),
            custom_size: Some(Vec2::new(72.0, 72.0)),
            ..default()
        },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.8),
        TitleEntity,
        ParallaxLayer { depth: 0.15, base_x: moon_x },
    ));

    // Far hills (parallax: background)
    for &(x, w, h, g) in &[
        (-200.0, 300.0, 80.0, 0.04_f32),
        (100.0, 350.0, 100.0, 0.035),
        (350.0, 280.0, 70.0, 0.045),
    ] {
        commands.spawn((
            Sprite {
                color: Color::srgb(g + 0.02, g, g * 0.6),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::from_xyz(x, -VIEWPORT_H / 2.0 + 60.0 + h / 2.0 - 20.0, Z_BACKGROUND + 1.0),
            TitleEntity,
            ParallaxLayer { depth: 0.25, base_x: x },
        ));
    }

    // Ground
    let ground_y = -VIEWPORT_H / 2.0 + 50.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(0.12, 0.08, 0.04),
            custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y, Z_BACKGROUND + 2.0),
        TitleEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.22, 0.15, 0.06),
            custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 5.0)),
            ..default()
        },
        Transform::from_xyz(0.0, ground_y + 52.5, Z_BACKGROUND + 3.0),
        TitleEntity,
    ));

    // Ground decorations: stones and grass tufts along the ground line
    {
        let deco_y = ground_y + 52.5; // top of ground line
        // Stones
        for _ in 0..6 {
            // Avoid center 60px where the well sits
            let x_range_pick = rng.gen_range(0..2_u8);
            let x: f32 = if x_range_pick == 0 {
                rng.gen_range(-VIEWPORT_W / 2.0 + 20.0..-35.0)
            } else {
                rng.gen_range(35.0..VIEWPORT_W / 2.0 - 20.0)
            };
            let sw = rng.gen_range(3.0..7.0_f32);
            let sh = rng.gen_range(2.0..5.0_f32);
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.18, 0.14, 0.10),
                    custom_size: Some(Vec2::new(sw, sh)),
                    ..default()
                },
                Transform::from_xyz(x, deco_y + sh / 2.0 + rng.gen_range(1.0..4.0_f32), Z_BACKGROUND + 3.5),
                TitleEntity,
            ));
        }
        // Grass tufts
        for _ in 0..8 {
            let x_range_pick = rng.gen_range(0..2_u8);
            let x: f32 = if x_range_pick == 0 {
                rng.gen_range(-VIEWPORT_W / 2.0 + 20.0..-35.0)
            } else {
                rng.gen_range(35.0..VIEWPORT_W / 2.0 - 20.0)
            };
            let gw = rng.gen_range(2.0..5.0_f32);
            let gh = rng.gen_range(4.0..9.0_f32);
            let angle = rng.gen_range(-0.25..0.25_f32);
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.15, 0.22, 0.08),
                    custom_size: Some(Vec2::new(gw, gh)),
                    ..default()
                },
                Transform {
                    translation: Vec3::new(x, deco_y + gh / 2.0 + rng.gen_range(0.5..2.5_f32), Z_BACKGROUND + 3.5),
                    rotation: Quat::from_rotation_z(angle),
                    scale: Vec3::ONE,
                },
                TitleEntity,
            ));
        }
    }

    // Well sprite (parallax: foreground)
    // Use Well2 (empty) if player has completed at least one run, else Well1 (full)
    let well_base_y = ground_y + 52.5;
    let well_x = 0.0;
    let well_size = 96.0; // ~2x native sprite size
    let well_sprite_y = well_base_y + well_size / 2.0; // bottom edge sits on ground
    let well_texture: Handle<Image> = asset_server.load("well.png");
    commands.spawn((
        Sprite {
            image: well_texture,
            custom_size: Some(Vec2::new(well_size, well_size)),
            ..default()
        },
        Transform::from_xyz(well_x, well_sprite_y, Z_BACKGROUND + 5.5),
        TitleEntity,
        ParallaxLayer { depth: 0.7, base_x: well_x },
        WellBob { timer: 0.0, base_y: well_sprite_y },
    ));

    // ── Module 3: Well glow sprite (golden upward light) ─────────
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.8, 0.2, 0.06),
            custom_size: Some(Vec2::new(60.0, 120.0)),
            ..default()
        },
        Transform::from_xyz(well_x, well_base_y + 70.0, Z_BACKGROUND + 4.9),
        TitleEntity,
        WellGlow { timer: 0.0 },
        ParallaxLayer { depth: 0.7, base_x: well_x },
    ));
    // Second narrower glow layer
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 0.9, 0.4, 0.04),
            custom_size: Some(Vec2::new(30.0, 90.0)),
            ..default()
        },
        Transform::from_xyz(well_x, well_base_y + 55.0, Z_BACKGROUND + 4.95),
        TitleEntity,
        WellGlow { timer: 1.5 },
        ParallaxLayer { depth: 0.7, base_x: well_x },
    ));

    // Well particle emitter resource
    commands.insert_resource(WellParticleTimer {
        timer: 0.0,
        well_x,
        well_y: well_base_y + 25.0,
    });

    // Trees — 3 depth layers creating forest-clearing illusion around the well.
    // Layer A: far background  — tiny, very dark, semi-transparent, slow parallax
    // Layer B: mid-distance    — medium, dark-tinted, moderate parallax
    // Layer C: near foreground — large, bright autumn tones, fast parallax
    //
    // (x_off, w, h, z_off, parallax_depth, r, g, b, alpha)
    let tree_handle: Handle<Image> = asset_server.load("tree.png");

    // Autumn canopy palette (used for leaf particles below)
    let autumn_colors: [(f32, f32, f32); 5] = [
        (0.85, 0.40, 0.05),
        (0.75, 0.18, 0.08),
        (0.90, 0.70, 0.10),
        (0.65, 0.28, 0.08),
        (0.72, 0.50, 0.12),
    ];

    // (x, w, h, z, parallax, r, g, b, alpha)
    #[allow(clippy::type_complexity)]
    let tree_defs: &[(f32, f32, f32, f32, f32, f32, f32, f32, f32)] = &[
        // ── Layer A: far background (tiny, very dark, faded) ──────────────
        (-430.0, 42.0,  52.0, Z_BACKGROUND + 2.2, 0.12, 0.22, 0.10, 0.04, 0.35),
        (-370.0, 36.0,  44.0, Z_BACKGROUND + 2.2, 0.12, 0.20, 0.09, 0.03, 0.30),
        (-310.0, 48.0,  58.0, Z_BACKGROUND + 2.3, 0.14, 0.25, 0.11, 0.04, 0.38),
        (-260.0, 38.0,  46.0, Z_BACKGROUND + 2.2, 0.12, 0.20, 0.09, 0.03, 0.28),
        (-210.0, 44.0,  54.0, Z_BACKGROUND + 2.3, 0.14, 0.22, 0.10, 0.04, 0.32),
        (  -90.0, 40.0, 48.0, Z_BACKGROUND + 2.1, 0.10, 0.20, 0.09, 0.03, 0.25), // behind well L
        (  -30.0, 34.0, 42.0, Z_BACKGROUND + 2.0, 0.08, 0.18, 0.08, 0.02, 0.20), // behind well C
        (   30.0, 34.0, 42.0, Z_BACKGROUND + 2.0, 0.08, 0.18, 0.08, 0.02, 0.20), // behind well C
        (   90.0, 40.0, 48.0, Z_BACKGROUND + 2.1, 0.10, 0.20, 0.09, 0.03, 0.25), // behind well R
        ( 210.0, 44.0,  54.0, Z_BACKGROUND + 2.3, 0.14, 0.22, 0.10, 0.04, 0.32),
        ( 260.0, 38.0,  46.0, Z_BACKGROUND + 2.2, 0.12, 0.20, 0.09, 0.03, 0.28),
        ( 310.0, 48.0,  58.0, Z_BACKGROUND + 2.3, 0.14, 0.25, 0.11, 0.04, 0.38),
        ( 370.0, 36.0,  44.0, Z_BACKGROUND + 2.2, 0.12, 0.20, 0.09, 0.03, 0.30),
        ( 430.0, 42.0,  52.0, Z_BACKGROUND + 2.2, 0.12, 0.22, 0.10, 0.04, 0.35),

        // ── Layer B: mid-distance (medium, dark autumn tint) ──────────────
        (-390.0, 68.0,  82.0, Z_BACKGROUND + 3.0, 0.25, 0.38, 0.15, 0.03, 0.55),
        (-330.0, 62.0,  76.0, Z_BACKGROUND + 3.0, 0.25, 0.32, 0.12, 0.04, 0.50),
        (-275.0, 72.0,  88.0, Z_BACKGROUND + 3.1, 0.28, 0.42, 0.18, 0.05, 0.60),
        (-225.0, 60.0,  73.0, Z_BACKGROUND + 3.0, 0.25, 0.30, 0.11, 0.03, 0.48),
        (-180.0, 75.0,  92.0, Z_BACKGROUND + 3.1, 0.28, 0.45, 0.20, 0.06, 0.62),
        ( 180.0, 75.0,  92.0, Z_BACKGROUND + 3.1, 0.28, 0.45, 0.20, 0.06, 0.62),
        ( 225.0, 60.0,  73.0, Z_BACKGROUND + 3.0, 0.25, 0.30, 0.11, 0.03, 0.48),
        ( 275.0, 72.0,  88.0, Z_BACKGROUND + 3.1, 0.28, 0.42, 0.18, 0.05, 0.60),
        ( 330.0, 62.0,  76.0, Z_BACKGROUND + 3.0, 0.25, 0.32, 0.12, 0.04, 0.50),
        ( 390.0, 68.0,  82.0, Z_BACKGROUND + 3.0, 0.25, 0.38, 0.15, 0.03, 0.55),

        // ── Layer C: near foreground (large, rich autumn, full opacity) ───
        (-440.0, 100.0, 122.0, Z_BACKGROUND + 4.0, 0.45, 0.85, 0.40, 0.05, 0.92),
        (-365.0,  90.0, 110.0, Z_BACKGROUND + 4.0, 0.45, 0.75, 0.18, 0.08, 0.90),
        (-295.0, 110.0, 134.0, Z_BACKGROUND + 4.1, 0.48, 0.90, 0.70, 0.10, 0.95),
        (-230.0,  95.0, 116.0, Z_BACKGROUND + 4.0, 0.45, 0.65, 0.28, 0.08, 0.90),
        (-165.0, 118.0, 144.0, Z_BACKGROUND + 4.2, 0.50, 0.85, 0.40, 0.05, 1.00),
        ( 165.0, 118.0, 144.0, Z_BACKGROUND + 4.2, 0.50, 0.85, 0.40, 0.05, 1.00),
        ( 230.0,  95.0, 116.0, Z_BACKGROUND + 4.0, 0.45, 0.65, 0.28, 0.08, 0.90),
        ( 295.0, 110.0, 134.0, Z_BACKGROUND + 4.1, 0.48, 0.90, 0.70, 0.10, 0.95),
        ( 365.0,  90.0, 110.0, Z_BACKGROUND + 4.0, 0.45, 0.75, 0.18, 0.08, 0.90),
        ( 440.0, 100.0, 122.0, Z_BACKGROUND + 4.0, 0.45, 0.85, 0.40, 0.05, 0.92),
    ];

    for &(x_off, tw, th, z, parallax, tr, tg, tb, ta) in tree_defs {
        let sway_speed = rng.gen_range(0.25..0.55_f32);
        let sway_phase = rng.gen_range(0.0..std::f32::consts::TAU);
        let base_angle: f32 = rng.gen_range(-0.015..0.015);
        let sprite_y = well_base_y + th / 2.0;

        commands.spawn((
            Sprite {
                image: tree_handle.clone(),
                color: Color::srgba(tr, tg, tb, ta),
                custom_size: Some(Vec2::new(tw, th)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x_off, sprite_y, z),
                rotation: Quat::from_rotation_z(base_angle),
                scale: Vec3::ONE,
            },
            TitleEntity,
            ParallaxLayer { depth: parallax, base_x: x_off },
            TreeSway { speed: sway_speed, phase: sway_phase, base_angle },
        ));
    }

    // Falling autumn leaf particles (subtle, 7 initial)
    for _ in 0..7 {
        let ci = rng.gen_range(0..autumn_colors.len());
        let (lr, lg, lb) = autumn_colors[ci];
        let x = rng.gen_range(-VIEWPORT_W / 2.5..VIEWPORT_W / 2.5);
        let y = rng.gen_range(-VIEWPORT_H / 4.0..VIEWPORT_H / 2.5);
        let sz = rng.gen_range(2.5..5.0_f32);
        let lt = rng.gen_range(4.0..9.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(lr, lg, lb, 0.7),
                custom_size: Some(Vec2::new(sz, sz * 0.6)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x, y, Z_BACKGROUND + 6.0),
                rotation: Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU)),
                scale: Vec3::ONE,
            },
            TitleEntity,
            FallingLeaf {
                vx: rng.gen_range(-12.0..12.0),
                vy: rng.gen_range(-18.0..-8.0),
                spin: rng.gen_range(-1.5..1.5),
                sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
                lifetime: lt,
                max_lifetime: lt,
            },
        ));
    }

    // Fireflies (parallax: mid-foreground)
    for _ in 0..8 {
        let x = rng.gen_range(-VIEWPORT_W / 3.0..VIEWPORT_W / 3.0);
        let y = rng.gen_range(well_base_y + 10.0..well_base_y + 120.0);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.9, 0.6, 0.15, 0.6),
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 7.0),
            TitleEntity,
            ParallaxLayer { depth: 0.6, base_x: x },
        ));
    }

    // Title logo
    spawn_logo(&mut commands, &asset_server);
    let f = font.0.clone();
    // Subtitle backdrop
    commands.spawn((
        Sprite {
            color: Color::srgba(0.06, 0.03, 0.02, 0.75),
            custom_size: Some(Vec2::new(220.0, 28.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 120.0, Z_HUD - 0.1),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("Into the Depths"),
        TextFont { font: f.clone(), font_size: 12.0, ..default() },
        TextColor(Color::srgb(0.85, 0.65, 0.35)),
        Transform::from_xyz(0.0, 120.0, Z_HUD),
        TitleEntity,
    ));
    // Controls text — fades in after 2 seconds
    commands.spawn((
        Text2d::new("A/D Move  Space Jump  J Melee  K Block  Shift Dash  1-4 Spells  Gamepad OK"),
        TextFont { font: f.clone(), font_size: 6.0, ..default() },
        TextColor(Color::srgba(0.45, 0.35, 0.25, 0.0)),
        Transform::from_xyz(0.0, -200.0, Z_HUD),
        TitleEntity,
        FadeInDelay { delay: 2.0, elapsed: 0.0, fade_duration: 1.0, target_alpha: 1.0 },
    ));

    // "Press SPACE" with PulsingAlpha — text changes based on whether any save exists
    let any_save = load_slot(0).is_some() || load_slot(1).is_some() || load_slot(2).is_some();
    let prompt_text = if any_save {
        "- Press SPACE to continue -"
    } else {
        "- Press SPACE to descend -"
    };
    commands.spawn((
        Text2d::new(prompt_text),
        TextFont { font: f.clone(), font_size: 10.0, ..default() },
        TextColor(Color::srgba(0.95, 0.7, 0.25, 1.0)),
        Transform::from_xyz(0.0, -230.0, Z_HUD),
        TitleEntity,
        PromptText,
        PulsingAlpha { speed: 2.5, min_alpha: 0.3, max_alpha: 1.0 },
    ));

    // Bottom hints: "[S] Settings    [Q] Quit"
    commands.spawn((
        Text2d::new("[S] Settings    [Q] Quit"),
        TextFont { font: f.clone(), font_size: 7.0, ..default() },
        TextColor(Color::srgba(0.55, 0.45, 0.30, 0.7)),
        Transform::from_xyz(0.0, -248.0, Z_HUD),
        TitleEntity,
    ));

    // Best run display (bottom-left), only if runs completed
    if meta.runs_completed > 0 {
        commands.spawn((
            Text2d::new(format!("Best: Floor {}  |  Runs: {}", meta.best_floor, meta.runs_completed)),
            TextFont { font: f.clone(), font_size: 7.0, ..default() },
            TextColor(Color::srgb(0.45, 0.38, 0.3)),
            Transform::from_xyz(-VIEWPORT_W / 2.0 + 100.0, -248.0, Z_HUD),
            TitleEntity,
        ));
    }
}

fn spawn_logo(commands: &mut Commands, asset_server: &AssetServer) {
    let parent = commands.spawn((
        Transform::from_xyz(0.0, 160.0, Z_HUD),
        Visibility::Visible,
        TitleEntity,
    )).id();

    // ── Logo image (loaded from assets/logo.png) ─────────────────────
    let logo_size = 120.0; // fits nicely above DAWNROOT text
    let logo_y = 52.0; // centered above text
    {
        // Glow halo behind logo
        let child = commands.spawn((
            Sprite {
                color: Color::srgba(0.70, 0.38, 0.05, 0.18),
                custom_size: Some(Vec2::new(180.0, 160.0)),
                ..default()
            },
            Transform::from_xyz(0.0, logo_y, 0.05),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }
    {
        let child = commands.spawn((
            Sprite {
                image: asset_server.load("logo.png"),
                custom_size: Some(Vec2::new(logo_size, logo_size)),
                ..default()
            },
            Transform::from_xyz(0.0, logo_y, 0.3),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // Sun rays (emanate from behind the logo)
    let sun_cx: f32 = 0.0;
    let ray_origin_y: f32 = logo_y;
    let ray_angles: &[(f32, f32, f32)] = &[
        (-0.90, 52.0, 3.0),
        (-0.62, 58.0, 2.5),
        (-0.34, 62.0, 3.5),
        (-0.12, 55.0, 2.0),
        ( 0.12, 55.0, 2.0),
        ( 0.34, 62.0, 3.5),
        ( 0.62, 58.0, 2.5),
        ( 0.90, 52.0, 3.0),
    ];
    for &(angle, length, width) in ray_angles {
        let dir_x = angle.sin();
        let dir_y = angle.cos();
        let cx = sun_cx + dir_x * (length / 2.0 + 28.0);
        let cy = ray_origin_y + dir_y * (length / 2.0 + 28.0);
        let child = commands.spawn((
            Sprite {
                color: Color::srgba(0.88, 0.58, 0.12, 0.55),
                custom_size: Some(Vec2::new(width, length)),
                ..default()
            },
            Transform {
                translation: Vec3::new(cx, cy, 0.1),
                rotation: Quat::from_rotation_z(-angle),
                scale: Vec3::ONE,
            },
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // Title backdrop
    {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.28, 0.14, 0.06),
                custom_size: Some(Vec2::new(272.0, 56.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 2.0, 0.35),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }
    {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.09, 0.05, 0.02),
                custom_size: Some(Vec2::new(264.0, 48.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 2.0, 0.40),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // "DAWNROOT" text
    {
        let child = commands.spawn((
            Text2d::new("DAWNROOT"),
            TextFont { font_size: 64.0, ..default() },
            TextColor(Color::srgb(0.95, 0.82, 0.42)),
            Transform::from_xyz(0.0, 0.0, 1.0),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // Root tendrils
    {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.22, 0.12, 0.05),
                custom_size: Some(Vec2::new(5.0, 28.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -27.0, 0.45),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }
    let root_segments: &[(f32, f32, f32, f32, f32)] = &[
        (-18.0, -30.0,  4.0, 20.0, -0.32),
        (-34.0, -38.0,  3.0, 16.0, -0.55),
        (-50.0, -42.0,  3.0, 12.0, -0.75),
        (-64.0, -44.0,  2.5, 10.0, -0.90),
        (-28.0, -46.0,  2.5, 14.0, -0.20),
        (-80.0, -42.0,  2.0,  8.0, -1.05),
        ( 18.0, -30.0,  4.0, 20.0,  0.32),
        ( 34.0, -38.0,  3.0, 16.0,  0.55),
        ( 50.0, -42.0,  3.0, 12.0,  0.75),
        ( 64.0, -44.0,  2.5, 10.0,  0.90),
        ( 28.0, -46.0,  2.5, 14.0,  0.20),
        ( 80.0, -42.0,  2.0,  8.0,  1.05),
        (-10.0, -40.0,  2.5, 10.0, -0.15),
        ( 10.0, -40.0,  2.5, 10.0,  0.15),
    ];
    for &(x, y, w, h, angle) in root_segments {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.20, 0.11, 0.04),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x, y, 0.45),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE,
            },
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }
    let root_tips: &[(f32, f32)] = &[
        (-82.0, -47.0), (-66.0, -51.0), (-52.0, -51.0),
        ( 82.0, -47.0), ( 66.0, -51.0), ( 52.0, -51.0),
        (-30.0, -56.0), ( 30.0, -56.0), (0.0, -53.0),
    ];
    for &(x, y) in root_tips {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.17, 0.09, 0.03),
                custom_size: Some(Vec2::new(3.0, 3.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.46),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // Leaf accents
    let leaves: &[(f32, f32, f32, f32, f32, f32, f32, f32)] = &[
        (-110.0,  10.0, 7.0, 5.0,  0.50, 0.80, 0.32, 0.05),
        (-120.0,  -8.0, 6.0, 4.0,  0.80, 0.72, 0.25, 0.04),
        (-100.0, -18.0, 5.0, 4.0,  0.30, 0.85, 0.38, 0.06),
        (-130.0,   2.0, 5.0, 3.5,  1.10, 0.65, 0.20, 0.03),
        ( 110.0,  10.0, 7.0, 5.0, -0.50, 0.80, 0.32, 0.05),
        ( 120.0,  -8.0, 6.0, 4.0, -0.80, 0.72, 0.25, 0.04),
        ( 100.0, -18.0, 5.0, 4.0, -0.30, 0.85, 0.38, 0.06),
        ( 130.0,   2.0, 5.0, 3.5, -1.10, 0.65, 0.20, 0.03),
        ( -55.0,  42.0, 6.0, 4.0,  0.60, 0.90, 0.68, 0.10),
        (  55.0,  42.0, 6.0, 4.0, -0.60, 0.90, 0.68, 0.10),
        ( -35.0,  50.0, 5.0, 3.5,  0.30, 0.95, 0.75, 0.15),
        (  35.0,  50.0, 5.0, 3.5, -0.30, 0.95, 0.75, 0.15),
        ( -45.0, -48.0, 5.0, 3.5,  0.70, 0.68, 0.18, 0.04),
        (  45.0, -48.0, 5.0, 3.5, -0.70, 0.68, 0.18, 0.04),
        ( -22.0, -54.0, 4.0, 3.0,  0.20, 0.72, 0.22, 0.04),
        (  22.0, -54.0, 4.0, 3.0, -0.20, 0.72, 0.22, 0.04),
    ];
    for &(x, y, w, h, angle, r, g, b) in leaves {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(r, g, b),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x, y, 0.50),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE,
            },
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
        let stem = commands.spawn((
            Sprite {
                color: Color::srgb(0.22, 0.12, 0.05),
                custom_size: Some(Vec2::new(1.5, 4.0)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x + angle.sin() * -3.0, y + angle.cos() * -3.0, 0.49),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE,
            },
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(stem);
    }

    // Horizontal ground line at base of logo
    {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(0.30, 0.15, 0.05),
                custom_size: Some(Vec2::new(280.0, 2.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -18.0, 0.44),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }
}

fn cleanup_title(mut commands: Commands, q: Query<Entity, With<TitleEntity>>) {
    for e in &q {
        commands.entity(e).try_despawn_recursive();
    }
    commands.remove_resource::<SlotMenuState>();
    commands.remove_resource::<WellParticleTimer>();
}

// ── Module 1: Parallax mouse tracking ────────────────────────────

fn parallax_mouse_tracking(
    window_q: Query<&Window>,
    mut parallax_q: Query<(&mut Transform, &ParallaxLayer)>,
) {
    let Ok(window) = window_q.get_single() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    // Normalize cursor to -1..1 range (center = 0)
    let norm_x = (cursor.x / window.width() - 0.5) * 2.0;
    let max_offset = 18.0;

    for (mut tf, layer) in &mut parallax_q {
        let offset = -norm_x * max_offset * layer.depth;
        tf.translation.x = layer.base_x + offset;
    }
}

// ── Module 2: Pulsing alpha system ───────────────────────────────

fn pulse_text_alpha(
    time: Res<Time>,
    mut query: Query<(&PulsingAlpha, &mut TextColor)>,
) {
    let t = time.elapsed_secs();
    for (pulse, mut color) in &mut query {
        let wave = (t * pulse.speed).sin() * 0.5 + 0.5; // 0..1
        let alpha = pulse.min_alpha + wave * (pulse.max_alpha - pulse.min_alpha);
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ── Module 3: Well glow + particles ──────────────────────────────

fn well_bob_animate(
    time: Res<Time>,
    mut query: Query<(&mut WellBob, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (mut bob, mut tf) in &mut query {
        bob.timer += dt;
        // Gentle bob: 3px amplitude, slow speed
        let offset = (bob.timer * 1.2).sin() * 3.0;
        tf.translation.y = bob.base_y + offset;
    }
}

fn well_glow_animate(
    time: Res<Time>,
    mut query: Query<(&mut WellGlow, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut glow, mut sprite) in &mut query {
        glow.timer += dt;
        let pulse = (glow.timer * 1.8).sin() * 0.03 + 0.06;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, pulse.max(0.02));
    }
}

fn well_particle_emit(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<WellParticleTimer>,
) {
    use rand::Rng;
    let dt = time.delta_secs();
    timer.timer -= dt;
    if timer.timer <= 0.0 {
        timer.timer = 0.15; // spawn every 150ms
        let mut rng = rand::thread_rng();
        let px = timer.well_x + rng.gen_range(-18.0..18.0_f32);
        let py = timer.well_y;
        let brightness = rng.gen_range(0.6..1.0_f32);
        let size = rng.gen_range(1.5..3.5_f32);
        let lt = rng.gen_range(1.5..3.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(1.0 * brightness, 0.8 * brightness, 0.2 * brightness, 0.5),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_xyz(px, py, Z_BACKGROUND + 5.7),
            TitleEntity,
            WellParticle {
                vx: rng.gen_range(-8.0..8.0),
                vy: rng.gen_range(20.0..45.0),
                lifetime: lt,
                max_lifetime: lt,
            },
        ));
    }
}

fn update_well_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut WellParticle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut p) in &mut query {
        p.lifetime -= dt;
        if p.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.x += p.vx * dt;
        tf.translation.y += p.vy * dt;
        p.vx *= 0.98; // gentle lateral friction
        let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0) * 0.5;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ── Module 5: Tree sway ───────────────────────────────────────────

fn tree_sway_animate(
    time: Res<Time>,
    mut query: Query<(&TreeSway, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (sway, mut tf) in &mut query {
        // ±2 degrees (0.035 rad) gentle oscillation
        let angle = sway.base_angle + (t * sway.speed + sway.phase).sin() * 0.035;
        tf.rotation = Quat::from_rotation_z(angle);
    }
}

// ── Module 6: Star twinkle ────────────────────────────────────────

fn star_twinkle_animate(
    time: Res<Time>,
    mut query: Query<(&TwinklingStar, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (star, mut sprite) in &mut query {
        let wave = (t * star.speed + star.phase).sin() * 0.5 + 0.5; // 0..1
        let alpha = (star.base_alpha + wave * 0.3 - 0.15).clamp(0.05, 1.0);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ── Module 7: Fade-in delay ───────────────────────────────────────

fn fade_in_delay_system(
    time: Res<Time>,
    mut query: Query<(&mut FadeInDelay, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    for (mut fid, mut color) in &mut query {
        fid.elapsed += dt;
        let alpha = if fid.elapsed < fid.delay {
            0.0
        } else {
            let t = ((fid.elapsed - fid.delay) / fid.fade_duration).clamp(0.0, 1.0);
            t * fid.target_alpha
        };
        let c = color.0.to_srgba();
        color.0 = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ── Falling leaf update ───────────────────────────────────────────

fn update_falling_leaves(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut FallingLeaf)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for (entity, mut tf, mut sprite, mut leaf) in &mut query {
        leaf.lifetime -= dt;
        if leaf.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Lateral sway
        let sway = (t * 1.5 + leaf.sway_phase).sin() * 15.0;
        tf.translation.x += (leaf.vx + sway) * dt;
        tf.translation.y += leaf.vy * dt;
        tf.rotation *= Quat::from_rotation_z(leaf.spin * dt);
        // Fade out in last 20% of lifetime
        let alpha = (leaf.lifetime / leaf.max_lifetime).clamp(0.0, 1.0).min(1.0) * 0.7;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

// ── Title input (Module 2: pulse extracted to separate system) ───

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    mut slot_state: ResMut<SlotMenuState>,
    font: Res<GameFont>,
    settings_q: Query<Entity, With<SettingsPanel>>,
    audio: Res<AudioSettings>,
    windows: Query<&Window>,
    mut app_exit: EventWriter<AppExit>,
) {
    // Block all title input while slot menu or settings are open.
    if slot_state.open { return; }
    if settings_q.iter().next().is_some() { return; }

    let gp = gamepads.iter().next();
    let gp_confirm = gp.map_or(false, |g| g.just_pressed(GamepadButton::South) || g.just_pressed(GamepadButton::Start));

    // Space / Enter → open save-slot menu.
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) || gp_confirm {
        slot_state.open = true;
        spawn_slot_menu(&mut commands, &font.0);
        return;
    }

    // S or Select → open settings overlay directly from title screen.
    let open_settings = keys.just_pressed(KeyCode::KeyS)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::Select));
    if open_settings {
        let is_fullscreen = windows.iter().next().map_or(false, |w| {
            matches!(w.mode, bevy::window::WindowMode::BorderlessFullscreen(_))
        });
        commands.insert_resource(SettingsState::default());
        spawn_settings_panel(&mut commands, &font.0, &audio, is_fullscreen);
        return;
    }

    // Q → quit the game cleanly
    if keys.just_pressed(KeyCode::KeyQ) {
        app_exit.send(AppExit::Success);
    }
}

/// Handles closing the settings overlay opened from the title screen.
/// The shared `settings_input` system handles all the internal navigation;
/// this just makes sure the overlay can also be dismissed by ESC on title.
fn handle_title_settings_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    settings_q: Query<Entity, With<SettingsPanel>>,
    audio: Res<AudioSettings>,
) {
    if settings_q.iter().next().is_none() { return; }

    let gp  = gamepads.iter().next();
    let back = keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Backspace)
        || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if back {
        crate::audio::save_audio_settings(&audio);
        for e in &settings_q {
            commands.entity(e).despawn_recursive();
        }
        commands.remove_resource::<SettingsState>();
    }
}

fn spawn_slot_menu(commands: &mut Commands, font: &Handle<Font>) {
    let slots: [Option<crate::SaveSlotData>; 3] = [
        load_slot(0),
        load_slot(1),
        load_slot(2),
    ];

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.03, 0.02, 0.94)),
            SlotUI,
            TitleEntity,
        ))
        .with_children(|parent| {
            let f = font.clone();
            parent.spawn((
                Text::new("Choose Your Path"),
                TextFont { font: f.clone(), font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.95, 0.7, 0.25)),
            ));

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            for i in 0..3 {
                let (label, detail_color) = if let Some(ref save) = slots[i] {
                    let mins = (save.time_played / 60.0) as i32;
                    let secs = (save.time_played % 60.0) as i32;
                    (
                        format!(
                            "[{}] Slot {} - Floor {} | {}g | {}:{:02}",
                            i + 1, i + 1, save.floor, save.gold, mins, secs
                        ),
                        Color::srgb(0.85, 0.65, 0.25),
                    )
                } else {
                    (
                        format!("[{}] Slot {} - Empty (New Game)", i + 1, i + 1),
                        Color::srgb(0.55, 0.45, 0.35),
                    )
                };

                parent.spawn((
                    Text::new(label),
                    TextFont { font: f.clone(), font_size: 9.0, ..default() },
                    TextColor(detail_color),
                ));
            }

            parent.spawn(Node { height: Val::Px(20.0), ..default() });

            parent.spawn((
                Text::new("1/2/3 or X/Y/A select | DEL erase | ESC/B back"),
                TextFont { font: f.clone(), font_size: 7.0, ..default() },
                TextColor(Color::srgb(0.45, 0.38, 0.3)),
            ));
        });
}

fn handle_slot_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    slot_state: Option<Res<SlotMenuState>>,
    ui_q: Query<Entity, With<SlotUI>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    font: Res<GameFont>,
) {
    let Some(state) = slot_state else { return };
    if !state.open { return };
    let gp = gamepads.iter().next();

    let back = keys.just_pressed(KeyCode::Escape) || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if back {
        for e in &ui_q {
            commands.entity(e).try_despawn_recursive();
        }
        commands.insert_resource(SlotMenuState { open: false });
        return;
    }

    let digit_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3];
    let gp_slot_buttons = [GamepadButton::West, GamepadButton::North, GamepadButton::South];
    let deleting = keys.pressed(KeyCode::Delete) || gp.map_or(false, |g| g.pressed(GamepadButton::Select));

    if deleting {
        for (i, &key) in digit_keys.iter().enumerate() {
            let gp_pressed = gp.map_or(false, |g| g.just_pressed(gp_slot_buttons[i]));
            if keys.just_pressed(key) || gp_pressed {
                delete_slot(i);
                for e in &ui_q {
                    commands.entity(e).try_despawn_recursive();
                }
                spawn_slot_menu(&mut commands, &font.0);
                return;
            }
        }
    }

    for (i, &key) in digit_keys.iter().enumerate() {
        let gp_pressed = !deleting && gp.map_or(false, |g| g.just_pressed(gp_slot_buttons[i]));
        if keys.just_pressed(key) || gp_pressed {
            active_slot.0 = i;

            if let Some(save) = load_slot(i) {
                commands.insert_resource(LoadedSave(save));
                next_state.set(GameState::Playing);
            } else {
                next_state.set(GameState::WellIntro);
            }
            return;
        }
    }
}

// ── Well Intro cutscene ───────────────────────────────────────────

#[derive(Component, Clone, Copy)]
struct IntroEntity;

#[derive(Component)]
struct IntroPlayer;

#[derive(Component)]
struct IntroPlayerPart;

#[derive(Component)]
struct IntroLegL;

#[derive(Component)]
struct IntroLegR;

#[derive(Component)]
struct DarknessOverlay;

#[derive(Component)]
struct IntroWellSprite {
    _unused: f32,  // kept so queries don't break; well is now static
}

/// Animation state for the intro player spritesheet.
#[derive(Component)]
struct IntroSpriteAnim {
    frame: usize,
    timer: f32,
}

#[derive(Component)]
struct FallingParticle {
    vy: f32,
    lifetime: f32,
}

#[derive(Component)]
struct IntroScreenShake {
    strength: f32,
    timer: f32,
}

#[derive(Resource)]
struct IntroState {
    phase: IntroPhase,
    timer: f32,
    player_start_y: f32,
    afterimage_timer: f32,
    dust_spawned: bool,
}

#[derive(PartialEq)]
enum IntroPhase {
    Opening,       // brief atmospheric hold before player walks in
    WalkToWell,    // player walks toward the well
    LookAtWell,    // player pauses at the well edge, glow pulses
    JumpIn,        // player leaps into the well
    FallDarkness,  // screen goes dark as player falls
    LandInCave,    // impact + transition to Playing
}

fn setup_well_intro(mut commands: Commands, sprite_assets: Res<PlayerSpriteAssets>, asset_server: Res<AssetServer>) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let ground_y = -VIEWPORT_H / 2.0 + 50.0;
    let well_base_y = ground_y + 52.5;

    commands.insert_resource(IntroState {
        phase: IntroPhase::Opening,
        timer: 0.0,
        player_start_y: well_base_y + 16.0,
        afterimage_timer: 0.0,
        dust_spawned: false,
    });

    commands.spawn((Camera2d, IntroEntity, IntroScreenShake { strength: 0.0, timer: 0.0 }));

    // Night sky
    commands.spawn((
        Sprite { color: Color::srgb(0.06, 0.03, 0.02), custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)), ..default() },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND), IntroEntity,
    ));

    // Stars
    for _ in 0..90 {
        let x = rng.gen_range(-VIEWPORT_W / 2.0..VIEWPORT_W / 2.0);
        let y = rng.gen_range(0.0..VIEWPORT_H / 2.0);
        let b = rng.gen_range(0.25..0.9_f32);
        let roll = rng.gen_range(0.0..1.0_f32);
        let sz: f32 = if roll < 0.60 { 1.0 } else if roll < 0.85 { 2.0 } else { 3.0 };
        let speed = rng.gen_range(0.4..1.6_f32);
        let phase = rng.gen_range(0.0..std::f32::consts::TAU);
        commands.spawn((
            Sprite {
                color: Color::srgba(b, b * 0.9, b * 0.7, b),
                custom_size: Some(Vec2::new(sz, sz)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 0.5),
            IntroEntity,
            TwinklingStar { speed, phase, base_alpha: b },
        ));
    }

    // Moon
    let moon_x = 280.0_f32;
    let moon_y = 180.0_f32;
    commands.spawn((
        Sprite { color: Color::srgba(0.9, 0.88, 0.7, 0.04), custom_size: Some(Vec2::new(130.0, 130.0)), ..default() },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.6), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgba(1.0, 0.97, 0.8, 0.07), custom_size: Some(Vec2::new(88.0, 88.0)), ..default() },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.7), IntroEntity,
    ));
    commands.spawn((
        Sprite { image: asset_server.load("moon.png"), custom_size: Some(Vec2::new(72.0, 72.0)), ..default() },
        Transform::from_xyz(moon_x, moon_y, Z_BACKGROUND + 0.8), IntroEntity,
    ));

    // Ground
    commands.spawn((
        Sprite { color: Color::srgb(0.12, 0.08, 0.04), custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 100.0)), ..default() },
        Transform::from_xyz(0.0, ground_y, Z_BACKGROUND + 2.0), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.22, 0.15, 0.06), custom_size: Some(Vec2::new(VIEWPORT_W + 20.0, 5.0)), ..default() },
        Transform::from_xyz(0.0, ground_y + 52.5, Z_BACKGROUND + 3.0), IntroEntity,
    ));

    // Forest trees — same 3-layer depth as title screen
    let tree_handle: Handle<Image> = asset_server.load("tree.png");
    // (x, w, h, z, r, g, b, alpha)
    let intro_trees: &[(f32, f32, f32, f32, f32, f32, f32, f32)] = &[
        // Layer A: far background
        (-430.0, 42.0, 52.0, Z_BACKGROUND + 2.2, 0.22, 0.10, 0.04, 0.32),
        (-350.0, 38.0, 46.0, Z_BACKGROUND + 2.2, 0.20, 0.09, 0.03, 0.28),
        (-270.0, 46.0, 56.0, Z_BACKGROUND + 2.3, 0.25, 0.11, 0.04, 0.35),
        ( -90.0, 40.0, 48.0, Z_BACKGROUND + 2.1, 0.20, 0.09, 0.03, 0.22),
        ( -30.0, 34.0, 42.0, Z_BACKGROUND + 2.0, 0.18, 0.08, 0.02, 0.18),
        (  30.0, 34.0, 42.0, Z_BACKGROUND + 2.0, 0.18, 0.08, 0.02, 0.18),
        (  90.0, 40.0, 48.0, Z_BACKGROUND + 2.1, 0.20, 0.09, 0.03, 0.22),
        ( 270.0, 46.0, 56.0, Z_BACKGROUND + 2.3, 0.25, 0.11, 0.04, 0.35),
        ( 350.0, 38.0, 46.0, Z_BACKGROUND + 2.2, 0.20, 0.09, 0.03, 0.28),
        ( 430.0, 42.0, 52.0, Z_BACKGROUND + 2.2, 0.22, 0.10, 0.04, 0.32),
        // Layer B: mid-distance
        (-390.0, 68.0, 82.0, Z_BACKGROUND + 3.0, 0.38, 0.15, 0.03, 0.52),
        (-310.0, 64.0, 78.0, Z_BACKGROUND + 3.0, 0.32, 0.12, 0.04, 0.48),
        (-230.0, 72.0, 88.0, Z_BACKGROUND + 3.1, 0.42, 0.18, 0.05, 0.58),
        (-160.0, 70.0, 86.0, Z_BACKGROUND + 3.1, 0.40, 0.16, 0.04, 0.55),
        ( 160.0, 70.0, 86.0, Z_BACKGROUND + 3.1, 0.40, 0.16, 0.04, 0.55),
        ( 230.0, 72.0, 88.0, Z_BACKGROUND + 3.1, 0.42, 0.18, 0.05, 0.58),
        ( 310.0, 64.0, 78.0, Z_BACKGROUND + 3.0, 0.32, 0.12, 0.04, 0.48),
        ( 390.0, 68.0, 82.0, Z_BACKGROUND + 3.0, 0.38, 0.15, 0.03, 0.52),
        // Layer C: near foreground — full autumn colors
        (-450.0, 102.0, 124.0, Z_BACKGROUND + 4.0, 0.85, 0.40, 0.05, 0.92),
        (-370.0,  90.0, 110.0, Z_BACKGROUND + 4.0, 0.75, 0.18, 0.08, 0.90),
        (-290.0, 112.0, 136.0, Z_BACKGROUND + 4.1, 0.90, 0.70, 0.10, 0.95),
        (-215.0,  96.0, 118.0, Z_BACKGROUND + 4.0, 0.65, 0.28, 0.08, 0.90),
        (-148.0, 120.0, 146.0, Z_BACKGROUND + 4.2, 0.85, 0.40, 0.05, 1.00),
        ( 148.0, 120.0, 146.0, Z_BACKGROUND + 4.2, 0.85, 0.40, 0.05, 1.00),
        ( 215.0,  96.0, 118.0, Z_BACKGROUND + 4.0, 0.65, 0.28, 0.08, 0.90),
        ( 290.0, 112.0, 136.0, Z_BACKGROUND + 4.1, 0.90, 0.70, 0.10, 0.95),
        ( 370.0,  90.0, 110.0, Z_BACKGROUND + 4.0, 0.75, 0.18, 0.08, 0.90),
        ( 450.0, 102.0, 124.0, Z_BACKGROUND + 4.0, 0.85, 0.40, 0.05, 0.92),
    ];
    for &(x_off, tw, th, z, tr, tg, tb, ta) in intro_trees {
        let sway_speed = rng.gen_range(0.25..0.55_f32);
        let sway_phase = rng.gen_range(0.0..std::f32::consts::TAU);
        let base_angle: f32 = rng.gen_range(-0.015..0.015);
        let sprite_y = well_base_y + th / 2.0;
        commands.spawn((
            Sprite {
                image: tree_handle.clone(),
                color: Color::srgba(tr, tg, tb, ta),
                custom_size: Some(Vec2::new(tw, th)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x_off, sprite_y, z),
                rotation: Quat::from_rotation_z(base_angle),
                scale: Vec3::ONE,
            },
            IntroEntity,
            TreeSway { speed: sway_speed, phase: sway_phase, base_angle },
        ));
    }

    // Ground decorations: stones and grass tufts
    let deco_y = well_base_y;
    for _ in 0..6 {
        let side = rng.gen_range(0..2_u8);
        let x: f32 = if side == 0 { rng.gen_range(-VIEWPORT_W / 2.0 + 20.0..-55.0) } else { rng.gen_range(55.0..VIEWPORT_W / 2.0 - 20.0) };
        let sw = rng.gen_range(3.0..7.0_f32);
        let sh = rng.gen_range(2.0..5.0_f32);
        commands.spawn((
            Sprite { color: Color::srgb(0.18, 0.14, 0.10), custom_size: Some(Vec2::new(sw, sh)), ..default() },
            Transform::from_xyz(x, deco_y + sh / 2.0 + rng.gen_range(1.0..4.0_f32), Z_BACKGROUND + 3.5),
            IntroEntity,
        ));
    }
    for _ in 0..8 {
        let side = rng.gen_range(0..2_u8);
        let x: f32 = if side == 0 { rng.gen_range(-VIEWPORT_W / 2.0 + 20.0..-55.0) } else { rng.gen_range(55.0..VIEWPORT_W / 2.0 - 20.0) };
        let gw = rng.gen_range(2.0..5.0_f32);
        let gh = rng.gen_range(4.0..9.0_f32);
        let angle = rng.gen_range(-0.25..0.25_f32);
        commands.spawn((
            Sprite { color: Color::srgb(0.15, 0.22, 0.08), custom_size: Some(Vec2::new(gw, gh)), ..default() },
            Transform {
                translation: Vec3::new(x, deco_y + gh / 2.0 + rng.gen_range(0.5..2.5_f32), Z_BACKGROUND + 3.5),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE,
            },
            IntroEntity,
        ));
    }

    // Well glow (behind the sprite)
    commands.spawn((
        Sprite { color: Color::srgba(1.0, 0.8, 0.2, 0.06), custom_size: Some(Vec2::new(60.0, 120.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 70.0, Z_BACKGROUND + 4.9), IntroEntity,
        WellGlow { timer: 0.0 },
    ));
    commands.spawn((
        Sprite { color: Color::srgba(1.0, 0.9, 0.4, 0.04), custom_size: Some(Vec2::new(30.0, 90.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 55.0, Z_BACKGROUND + 4.95), IntroEntity,
        WellGlow { timer: 1.5 },
    ));

    // Well — static, already present in the clearing from the start
    let well_size = 96.0;
    let well_y = well_base_y + well_size / 2.0;
    commands.spawn((
        Sprite {
            image: asset_server.load("well.png"),
            custom_size: Some(Vec2::new(well_size, well_size)),
            ..default()
        },
        Transform::from_xyz(0.0, well_y, Z_BACKGROUND + 5.5),
        IntroEntity,
        IntroWellSprite { _unused: 0.0 },
        WellBob { timer: 0.0, base_y: well_y },
    ));

    // Falling autumn leaves — same palette as title screen
    let autumn_colors: [(f32, f32, f32); 5] = [
        (0.85, 0.40, 0.05), (0.75, 0.18, 0.08), (0.90, 0.70, 0.10),
        (0.65, 0.28, 0.08), (0.72, 0.50, 0.12),
    ];
    for _ in 0..10 {
        let ci = rng.gen_range(0..autumn_colors.len());
        let (lr, lg, lb) = autumn_colors[ci];
        let x = rng.gen_range(-VIEWPORT_W / 2.5..VIEWPORT_W / 2.5);
        let y = rng.gen_range(-VIEWPORT_H / 4.0..VIEWPORT_H / 2.5);
        let sz = rng.gen_range(2.5..5.0_f32);
        let lt = rng.gen_range(5.0..12.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(lr, lg, lb, 0.7),
                custom_size: Some(Vec2::new(sz, sz * 0.6)),
                ..default()
            },
            Transform {
                translation: Vec3::new(x, y, Z_BACKGROUND + 6.0),
                rotation: Quat::from_rotation_z(rng.gen_range(0.0..std::f32::consts::TAU)),
                scale: Vec3::ONE,
            },
            IntroEntity,
            FallingLeaf {
                vx: rng.gen_range(-12.0..12.0),
                vy: rng.gen_range(-18.0..-8.0),
                spin: rng.gen_range(-1.5..1.5),
                sway_phase: rng.gen_range(0.0..std::f32::consts::TAU),
                lifetime: lt,
                max_lifetime: lt,
            },
        ));
    }

    // Intro player — hidden until Opening phase ends, then walks in
    let run_start_index = 1 * 10;
    let sprite_size = 32.0 * 2.0;
    commands.spawn((
        Transform::from_xyz(-280.0, well_base_y + 16.0, Z_PLAYER),
        Visibility::Hidden,
        IntroEntity,
        IntroPlayer,
    )).with_children(|p| {
        p.spawn((
            Sprite {
                image: sprite_assets.texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: sprite_assets.layout.clone(),
                    index: run_start_index,
                }),
                custom_size: Some(Vec2::new(sprite_size, sprite_size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
            IntroEntity,
            IntroPlayerPart,
            IntroSpriteAnim { frame: 0, timer: 0.0 },
        ));
    });

    // Darkness overlay
    commands.spawn((
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(VIEWPORT_W + 100.0, VIEWPORT_H + 100.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_HUD - 1.0),
        IntroEntity,
        DarknessOverlay,
    ));
}

fn update_well_intro(
    mut commands: Commands,
    mut state: ResMut<IntroState>,
    mut player_q: Query<(&mut Transform, &mut Visibility), (With<IntroPlayer>, Without<IntroLegL>, Without<IntroLegR>, Without<IntroWellSprite>)>,
    mut parts_q: Query<&mut Sprite, (With<IntroPlayerPart>, Without<DarknessOverlay>, Without<WellGlow>)>,
    mut legl_q: Query<&mut Transform, (With<IntroLegL>, Without<IntroPlayer>, Without<IntroLegR>, Without<IntroWellSprite>)>,
    mut legr_q: Query<&mut Transform, (With<IntroLegR>, Without<IntroPlayer>, Without<IntroLegL>, Without<IntroWellSprite>)>,
    mut darkness_q: Query<&mut Sprite, (With<DarknessOverlay>, Without<IntroPlayerPart>, Without<WellGlow>)>,
    mut glow_q: Query<&mut Sprite, (With<WellGlow>, Without<DarknessOverlay>, Without<IntroPlayerPart>)>,
    mut shake_q: Query<&mut IntroScreenShake>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    state.timer += dt;

    // ── Opening: atmospheric hold, player hidden ─────────────────────
    if state.phase == IntroPhase::Opening {
        if state.timer >= 0.8 {
            state.phase = IntroPhase::WalkToWell;
            state.timer = 0.0;
            if let Ok((_, mut vis)) = player_q.get_single_mut() {
                *vis = Visibility::Visible;
            }
        }
        return;
    }

    let Ok((mut p_tf, _)) = player_q.get_single_mut() else { return };

    match state.phase {
        IntroPhase::Opening => unreachable!(),

        IntroPhase::WalkToWell => {
            p_tf.translation.x += 200.0 * dt;
            let bob = (state.timer * 14.0).sin().abs() * 3.0;
            p_tf.translation.y = state.player_start_y + bob;

            let sw_l = (state.timer * 14.0).sin() * 6.0;
            let sw_r = (state.timer * 14.0 + std::f32::consts::PI).sin() * 6.0;

            if let Ok(mut ll) = legl_q.get_single_mut() {
                ll.translation = Vec3::new(-3.5 + sw_l * 0.3, -12.0 + sw_l.abs() * 0.5, 0.0);
            }
            if let Ok(mut rl) = legr_q.get_single_mut() {
                rl.translation = Vec3::new(3.5 + sw_r * 0.3, -12.0 + sw_r.abs() * 0.5, 0.0);
            }

            // Stop about one body-width left of the well
            if p_tf.translation.x >= -38.0 {
                p_tf.translation.x = -38.0;
                p_tf.translation.y = state.player_start_y;
                if let Ok(mut ll) = legl_q.get_single_mut() { ll.translation = Vec3::new(-3.5, -12.0, 0.0); }
                if let Ok(mut rl) = legr_q.get_single_mut() { rl.translation = Vec3::new(3.5, -12.0, 0.0); }
                state.phase = IntroPhase::LookAtWell;
                state.timer = 0.0;
            }
        }

        // ── Player pauses at the well edge, glow pulses brighter ─────
        IntroPhase::LookAtWell => {
            // Subtle idle sway
            p_tf.translation.y = state.player_start_y + (state.timer * 3.0).sin() * 1.2;

            // Pulse the well glow brighter over 0.8s
            let glow_t = (state.timer / 0.8).min(1.0);
            for mut gsp in &mut glow_q {
                let base = gsp.color.to_srgba();
                let boosted_alpha = base.alpha + glow_t * 0.12;
                gsp.color = Color::srgba(base.red, base.green, base.blue, boosted_alpha.min(0.35));
            }

            if state.timer >= 0.9 {
                state.phase = IntroPhase::JumpIn;
                state.timer = 0.0;
                state.afterimage_timer = 0.0;
                // Snap to center for the jump
                p_tf.translation.x = -5.0;
            }
        }

        IntroPhase::JumpIn => {
            let t = state.timer;
            if t < 0.35 {
                let frac = t / 0.35;
                let y_off = 55.0 * (frac * std::f32::consts::PI).sin();
                p_tf.translation.y = state.player_start_y + y_off;
                let stretch = 1.0 + 0.25 * frac;
                p_tf.scale = Vec3::new(1.0 / stretch.sqrt(), stretch, 1.0);
            } else if t < 0.9 {
                let fall_frac = (t - 0.35) / 0.55;
                p_tf.translation.y = state.player_start_y + 10.0 - fall_frac * 90.0;
                p_tf.scale = Vec3::new(0.8, 1.15, 1.0);

                // Afterimage trail: spawn ghost every 0.04s during fall
                state.afterimage_timer -= dt;
                if state.afterimage_timer <= 0.0 {
                    state.afterimage_timer = 0.04;
                    // Body silhouette ghost
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.50, 0.30, 0.12, 0.5),
                            custom_size: Some(Vec2::new(14.0, 30.0)),
                            ..default()
                        },
                        Transform::from_xyz(p_tf.translation.x, p_tf.translation.y, Z_PLAYER - 0.5),
                        IntroEntity,
                        IntroAfterimage { lifetime: 0.5, max_lifetime: 0.5 },
                    ));
                    // Head ghost
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.78, 0.62, 0.48, 0.4),
                            custom_size: Some(Vec2::new(12.0, 11.0)),
                            ..default()
                        },
                        Transform::from_xyz(p_tf.translation.x, p_tf.translation.y + 12.0, Z_PLAYER - 0.6),
                        IntroEntity,
                        IntroAfterimage { lifetime: 0.5, max_lifetime: 0.5 },
                    ));
                }

                let alpha = (1.0 - fall_frac).max(0.0);
                for mut sprite in parts_q.iter_mut() {
                    let c = sprite.color.to_srgba();
                    sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
                }
            } else {
                for mut sprite in parts_q.iter_mut() {
                    sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.0);
                }
                state.phase = IntroPhase::FallDarkness;
                state.timer = 0.0;
            }
        }

        IntroPhase::FallDarkness => {
            if let Ok(mut ds) = darkness_q.get_single_mut() {
                let alpha = (state.timer / 0.6).min(1.0);
                ds.color = Color::srgba(0.0, 0.0, 0.0, alpha);
            }

            // Speed lines: vertical semi-transparent lines rushing upward
            if state.timer > 0.15 && state.timer < 2.0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                // Ramp up intensity early, sustain, then taper off
                let intensity = if state.timer < 0.5 {
                    6
                } else if state.timer < 1.4 {
                    4
                } else {
                    2
                };
                for _ in 0..intensity {
                    let px = rng.gen_range(-200.0..200.0_f32);
                    let speed = rng.gen_range(600.0..1100.0_f32);
                    let br = rng.gen_range(0.2..0.5_f32);
                    let width = rng.gen_range(1.5..4.5_f32);
                    let height = rng.gen_range(18.0..45.0_f32);
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(br, br * 0.85, br * 0.6, 0.75),
                            custom_size: Some(Vec2::new(width, height)),
                            ..default()
                        },
                        Transform::from_xyz(px, -VIEWPORT_H / 2.0 - 20.0, Z_HUD - 0.5),
                        IntroEntity,
                        FallingParticle { vy: speed, lifetime: 1.2 },
                    ));
                }
            }

            // Afterimage ghosts during early darkness (still visible through fade)
            if state.timer < 0.5 {
                state.afterimage_timer -= dt;
                if state.afterimage_timer <= 0.0 {
                    state.afterimage_timer = 0.06;
                    let alpha = (1.0 - state.timer / 0.5) * 0.35;
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.45, 0.28, 0.12, alpha),
                            custom_size: Some(Vec2::new(14.0, 30.0)),
                            ..default()
                        },
                        Transform::from_xyz(
                            (rand::random::<f32>() - 0.5) * 6.0,
                            (rand::random::<f32>() - 0.5) * 40.0,
                            Z_PLAYER - 0.5,
                        ),
                        IntroEntity,
                        IntroAfterimage { lifetime: 0.5, max_lifetime: 0.5 },
                    ));
                }
            }

            if state.timer >= 2.2 {
                state.phase = IntroPhase::LandInCave;
                state.timer = 0.0;
                state.dust_spawned = false;
            }
        }

        IntroPhase::LandInCave => {
            if let Ok(mut ds) = darkness_q.get_single_mut() {
                if state.timer < 0.1 {
                    // Brief bright flash on impact
                    let flash = (1.0 - state.timer / 0.1) * 0.35;
                    ds.color = Color::srgba(0.15 + flash, 0.10 + flash * 0.5, 0.06, 0.8);
                } else {
                    ds.color = Color::srgba(0.0, 0.0, 0.0, 1.0);
                }
            }

            // Camera shake + dust on landing (once)
            if !state.dust_spawned {
                state.dust_spawned = true;

                // Trigger camera shake
                if let Ok(mut shake) = shake_q.get_single_mut() {
                    shake.strength = 8.0;
                    shake.timer = 0.35;
                }

                // Exactly 2 dust particles at player's feet: one left-down, one right-down
                let foot_y = -VIEWPORT_H / 2.0 + 60.0;
                // Left particle
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.75, 0.7, 0.65, 0.85),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_xyz(-2.0, foot_y, Z_HUD - 0.3),
                    IntroEntity,
                    IntroDustPuff {
                        vx: -140.0,
                        vy: -40.0,
                        lifetime: 0.5,
                        max_lifetime: 0.5,
                    },
                ));
                // Right particle
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.7, 0.65, 0.6, 0.85),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_xyz(2.0, foot_y, Z_HUD - 0.3),
                    IntroEntity,
                    IntroDustPuff {
                        vx: 140.0,
                        vy: -40.0,
                        lifetime: 0.5,
                        max_lifetime: 0.5,
                    },
                ));
            }

            if state.timer >= 0.6 {
                next_state.set(GameState::Playing);
            }
        }
    }
}

/// Animate the intro player spritesheet — cycles run frames at 12 FPS.
fn animate_intro_sprite(
    mut anim_q: Query<(&mut IntroSpriteAnim, &mut Sprite)>,
    time: Res<Time>,
) {
    for (mut anim, mut sprite) in &mut anim_q {
        anim.timer += time.delta_secs();
        let fps = 0.083; // ~12 FPS
        let run_frames = 4;
        let run_row = 1;
        let cols = 10;
        while anim.timer >= fps {
            anim.timer -= fps;
            anim.frame = (anim.frame + 1) % run_frames;
        }
        let index = run_row * cols + anim.frame;
        if let Some(ref mut atlas) = sprite.texture_atlas {
            atlas.index = index;
        }
    }
}

fn cleanup_well_intro(
    mut commands: Commands,
    q: Query<Entity, With<IntroEntity>>,
) {
    for e in &q {
        commands.entity(e).try_despawn_recursive();
    }
    commands.remove_resource::<IntroState>();
}

fn update_falling_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FallingParticle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let half_h = VIEWPORT_H / 2.0 + 60.0;
    for (entity, mut tf, mut p) in &mut query {
        tf.translation.y += p.vy * dt;
        p.lifetime -= dt;
        if p.lifetime <= 0.0 || tf.translation.y > half_h || tf.translation.y < -half_h {
            commands.entity(entity).despawn();
        }
    }
}

// ── Module 4: Intro afterimage + dust systems ────────────────────

fn update_intro_afterimages(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut IntroAfterimage)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut sprite, mut ai) in &mut query {
        ai.lifetime -= dt;
        if ai.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let alpha = (ai.lifetime / ai.max_lifetime).clamp(0.0, 1.0) * 0.4;
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, alpha);
    }
}

fn update_intro_dust(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut IntroDustPuff)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut dust) in &mut query {
        dust.lifetime -= dt;
        if dust.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation.x += dust.vx * dt;
        tf.translation.y += dust.vy * dt;
        dust.vy -= 200.0 * dt; // gravity pulls down
        dust.vx *= 0.95;

        let t = (dust.lifetime / dust.max_lifetime).clamp(0.0, 1.0);
        // Shrink scale as particle fades
        tf.scale = Vec3::splat(t);
        let c = sprite.color.to_srgba();
        sprite.color = Color::srgba(c.red, c.green, c.blue, t * 0.85);
    }
}

fn intro_camera_shake(
    mut query: Query<(&mut Transform, &mut IntroScreenShake)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (mut tf, mut shake) in &mut query {
        if shake.timer > 0.0 {
            shake.timer = (shake.timer - dt).max(0.0);
            shake.strength *= (-10.0 * dt).exp();
            let sx = (rand::random::<f32>() * 2.0 - 1.0) * shake.strength;
            let sy = (rand::random::<f32>() * 2.0 - 1.0) * shake.strength;
            tf.translation.x = sx;
            tf.translation.y = sy;
        } else {
            tf.translation.x = 0.0;
            tf.translation.y = 0.0;
        }
    }
}
