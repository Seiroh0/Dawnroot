use bevy::prelude::*;
use crate::{GameState, constants::*, ActiveSaveSlot, LoadedSave, load_slot, delete_slot};

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Title), setup_title)
            .add_systems(OnExit(GameState::Title), cleanup_title)
            .add_systems(
                Update,
                (handle_title_input, handle_slot_input)
                    .run_if(in_state(GameState::Title)),
            )
            .add_systems(OnEnter(GameState::WellIntro), setup_well_intro)
            .add_systems(OnExit(GameState::WellIntro), cleanup_well_intro)
            .add_systems(
                Update,
                (update_well_intro, update_falling_particles)
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

// ── Title screen ──────────────────────────────────────────────────

fn setup_title(mut commands: Commands) {
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

    // Stars
    for _ in 0..60 {
        let x = rng.gen_range(-VIEWPORT_W / 2.0..VIEWPORT_W / 2.0);
        let y = rng.gen_range(20.0..VIEWPORT_H / 2.0);
        let b = rng.gen_range(0.2..0.9_f32);
        let sz = rng.gen_range(1.0..3.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(b, b * 0.9, b * 0.7, b),
                custom_size: Some(Vec2::new(sz, sz)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_BACKGROUND + 0.5),
            TitleEntity,
        ));
    }

    // Moon
    commands.spawn((
        Sprite {
            color: Color::srgba(0.95, 0.75, 0.35, 0.9),
            custom_size: Some(Vec2::new(40.0, 40.0)),
            ..default()
        },
        Transform::from_xyz(280.0, 180.0, Z_BACKGROUND + 0.8),
        TitleEntity,
    ));
    commands.spawn((
        Sprite {
            color: Color::srgb(0.06, 0.03, 0.02),
            custom_size: Some(Vec2::new(34.0, 34.0)),
            ..default()
        },
        Transform::from_xyz(288.0, 184.0, Z_BACKGROUND + 0.9),
        TitleEntity,
    ));

    // Far hills
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

    // Well
    let well_base_y = ground_y + 52.5;
    let well_x = 0.0;
    commands.spawn((
        Sprite { color: Color::srgb(0.35, 0.3, 0.25), custom_size: Some(Vec2::new(64.0, 28.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 14.0, Z_BACKGROUND + 5.0), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.01, 0.01, 0.02), custom_size: Some(Vec2::new(48.0, 20.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 14.0, Z_BACKGROUND + 5.5), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.35, 0.28), custom_size: Some(Vec2::new(70.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 30.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    // Pillars
    commands.spawn((
        Sprite { color: Color::srgb(0.32, 0.27, 0.2), custom_size: Some(Vec2::new(8.0, 60.0)), ..default() },
        Transform::from_xyz(well_x - 30.0, well_base_y + 60.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.32, 0.27, 0.2), custom_size: Some(Vec2::new(8.0, 60.0)), ..default() },
        Transform::from_xyz(well_x + 30.0, well_base_y + 60.0, Z_BACKGROUND + 6.0), TitleEntity,
    ));
    // Roof
    commands.spawn((
        Sprite { color: Color::srgb(0.28, 0.2, 0.14), custom_size: Some(Vec2::new(80.0, 10.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 90.0, Z_BACKGROUND + 6.5), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.3, 0.22, 0.15), custom_size: Some(Vec2::new(50.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 98.0, Z_BACKGROUND + 6.5), TitleEntity,
    ));
    // Rope + bucket
    commands.spawn((
        Sprite { color: Color::srgb(0.5, 0.4, 0.25), custom_size: Some(Vec2::new(2.0, 40.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 50.0, Z_BACKGROUND + 5.8), TitleEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.3, 0.18), custom_size: Some(Vec2::new(10.0, 8.0)), ..default() },
        Transform::from_xyz(well_x, well_base_y + 32.0, Z_BACKGROUND + 5.8), TitleEntity,
    ));

    // Trees
    for &(x_off, h_min, h_max) in &[
        (-320.0_f32, 100.0, 150.0),
        (-240.0, 80.0, 130.0),
        (-160.0, 120.0, 160.0),
        (160.0, 110.0, 155.0),
        (250.0, 90.0, 140.0),
        (340.0, 100.0, 150.0),
    ] {
        let h: f32 = rng.gen_range(h_min..h_max);
        let trunk_w: f32 = rng.gen_range(10.0..18.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.12, 0.07, 0.03),
                custom_size: Some(Vec2::new(trunk_w, h)),
                ..default()
            },
            Transform::from_xyz(x_off, well_base_y + h / 2.0, Z_BACKGROUND + 1.5),
            TitleEntity,
        ));
        let canopy_w: f32 = rng.gen_range(40.0..65.0);
        let canopy_h: f32 = rng.gen_range(35.0..55.0);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.55, 0.25, 0.08),
                custom_size: Some(Vec2::new(canopy_w, canopy_h)),
                ..default()
            },
            Transform::from_xyz(x_off, well_base_y + h - 5.0, Z_BACKGROUND + 1.6),
            TitleEntity,
        ));
    }

    // Fireflies
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
        ));
    }

    // Title logo
    spawn_logo(&mut commands);
    commands.spawn((
        Text2d::new("Into the Depths"),
        TextFont { font_size: 20.0, ..default() },
        TextColor(Color::srgb(0.75, 0.55, 0.3)),
        Transform::from_xyz(0.0, 120.0, Z_HUD),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("A/D: Move | Space: Jump | E: Melee | F: Ranged | Shift: Dash | 1-4: Spells | Gamepad OK"),
        TextFont { font_size: 11.0, ..default() },
        TextColor(Color::srgb(0.45, 0.35, 0.25)),
        Transform::from_xyz(0.0, -200.0, Z_HUD),
        TitleEntity,
    ));
    commands.spawn((
        Text2d::new("- Press SPACE to descend -"),
        TextFont { font_size: 22.0, ..default() },
        TextColor(Color::srgba(0.95, 0.7, 0.25, 1.0)),
        Transform::from_xyz(0.0, -230.0, Z_HUD),
        TitleEntity,
        PromptText,
    ));
}

fn spawn_logo(commands: &mut Commands) {
    // All child transforms are relative to the parent at (0.0, 160.0, Z_HUD).
    // Positive Y = up, positive Z = in front.
    //
    // Layout sketch (relative coords):
    //   Sun center:    (0,  58)  — half-circle built from horizontal bars
    //   Sun rays:      radiating outward from sun center
    //   Backdrop:      (-130, 0) to (130, 0) — dark plaque behind text
    //   Text "DAWNROOT": (0, 0)
    //   Root tendrils: below the backdrop, Y ~ -25 downward
    //   Leaf accents:  scattered around the roots

    let parent = commands.spawn((
        Transform::from_xyz(0.0, 160.0, Z_HUD),
        Visibility::Visible,
        TitleEntity,
    )).id();

    // ── Sun half-circle (layered horizontal bars, bottom to top) ──────────
    // Each bar row is slightly narrower and one step brighter gold going upward.
    // Rows are stacked starting at y_base (the flat bottom of the sun).
    let sun_cx: f32 = 0.0;
    let sun_base_y: f32 = 38.0;   // flat bottom of half-circle
    let bar_h: f32 = 7.0;
    let sun_rows: &[(f32, f32, f32, f32, f32)] = &[
        // (half_width, r, g, b, y_offset_from_base)
        (48.0, 0.55, 0.22, 0.04, 0.0),   // deep amber base
        (46.0, 0.62, 0.28, 0.06, 7.0),
        (43.0, 0.70, 0.35, 0.07, 14.0),
        (38.0, 0.78, 0.44, 0.08, 21.0),
        (32.0, 0.86, 0.55, 0.10, 28.0),
        (24.0, 0.92, 0.65, 0.14, 35.0),
        (15.0, 0.96, 0.76, 0.20, 42.0),  // bright gold tip
        ( 6.0, 1.00, 0.88, 0.35, 49.0),
    ];
    for &(hw, r, g, b, dy) in sun_rows {
        let child = commands.spawn((
            Sprite {
                color: Color::srgb(r, g, b),
                custom_size: Some(Vec2::new(hw * 2.0, bar_h + 1.0)),
                ..default()
            },
            Transform::from_xyz(sun_cx, sun_base_y + dy, 0.2),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // ── Sun glow halo — a wide, very dim amber rectangle behind the sun ──
    {
        let child = commands.spawn((
            Sprite {
                color: Color::srgba(0.70, 0.38, 0.05, 0.18),
                custom_size: Some(Vec2::new(140.0, 80.0)),
                ..default()
            },
            Transform::from_xyz(sun_cx, sun_base_y + 20.0, 0.05),
            TitleEntity,
        )).id();
        commands.entity(parent).add_child(child);
    }

    // ── Sun rays — thin rectangles rotated around sun center ─────────────
    let ray_origin_y: f32 = sun_base_y + 20.0; // approximate visual center of half-sun
    // Angles in radians from vertical. Only upward-facing rays (above the horizon).
    let ray_angles: &[(f32, f32, f32)] = &[
        // (angle from +Y axis,  ray_length, ray_width)
        (-0.90, 52.0, 3.0),  // far left
        (-0.62, 58.0, 2.5),
        (-0.34, 62.0, 3.5),
        (-0.12, 55.0, 2.0),
        ( 0.12, 55.0, 2.0),
        ( 0.34, 62.0, 3.5),
        ( 0.62, 58.0, 2.5),
        ( 0.90, 52.0, 3.0),  // far right
    ];
    for &(angle, length, width) in ray_angles {
        // The ray rectangle is drawn with its center at (length/2) distance from origin.
        let dir_x = angle.sin();  // angle measured from +Y, so sin = x component
        let dir_y = angle.cos();  // cos = y component
        let cx = sun_cx + dir_x * (length / 2.0 + 28.0); // 28 = approx sun radius
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

    // ── Title backdrop — dark sienna plaque behind the text ───────────────
    // Outer border (slightly larger, slightly lighter)
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
    // Inner fill (darker)
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

    // ── "DAWNROOT" text — child of parent so it renders on top ───────────
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

    // ── Root tendrils — organic pixel shapes below the backdrop ──────────
    // Central downward trunk
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
    // Root segments: (x, y, w, h, angle)
    let root_segments: &[(f32, f32, f32, f32, f32)] = &[
        // left side branches
        (-18.0, -30.0,  4.0, 20.0, -0.32),
        (-34.0, -38.0,  3.0, 16.0, -0.55),
        (-50.0, -42.0,  3.0, 12.0, -0.75),
        (-64.0, -44.0,  2.5, 10.0, -0.90),
        (-28.0, -46.0,  2.5, 14.0, -0.20),
        (-80.0, -42.0,  2.0,  8.0, -1.05),
        // right side branches
        ( 18.0, -30.0,  4.0, 20.0,  0.32),
        ( 34.0, -38.0,  3.0, 16.0,  0.55),
        ( 50.0, -42.0,  3.0, 12.0,  0.75),
        ( 64.0, -44.0,  2.5, 10.0,  0.90),
        ( 28.0, -46.0,  2.5, 14.0,  0.20),
        ( 80.0, -42.0,  2.0,  8.0,  1.05),
        // additional inner cross roots
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
    // Root tip dots (small square nubs at the ends of roots)
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

    // ── Leaf accents — small autumn rectangles scattered near roots/text ──
    // Each entry: (x, y, w, h, angle, r, g, b)
    let leaves: &[(f32, f32, f32, f32, f32, f32, f32, f32)] = &[
        // warm orange leaves left
        (-110.0,  10.0, 7.0, 5.0,  0.50, 0.80, 0.32, 0.05),
        (-120.0,  -8.0, 6.0, 4.0,  0.80, 0.72, 0.25, 0.04),
        (-100.0, -18.0, 5.0, 4.0,  0.30, 0.85, 0.38, 0.06),
        (-130.0,   2.0, 5.0, 3.5,  1.10, 0.65, 0.20, 0.03),
        // warm orange-red leaves right
        ( 110.0,  10.0, 7.0, 5.0, -0.50, 0.80, 0.32, 0.05),
        ( 120.0,  -8.0, 6.0, 4.0, -0.80, 0.72, 0.25, 0.04),
        ( 100.0, -18.0, 5.0, 4.0, -0.30, 0.85, 0.38, 0.06),
        ( 130.0,   2.0, 5.0, 3.5, -1.10, 0.65, 0.20, 0.03),
        // gold leaves near sun / top
        ( -55.0,  42.0, 6.0, 4.0,  0.60, 0.90, 0.68, 0.10),
        (  55.0,  42.0, 6.0, 4.0, -0.60, 0.90, 0.68, 0.10),
        ( -35.0,  50.0, 5.0, 3.5,  0.30, 0.95, 0.75, 0.15),
        (  35.0,  50.0, 5.0, 3.5, -0.30, 0.95, 0.75, 0.15),
        // rust-red near root base
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
        // Small stem pixel for each leaf
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

    // ── Horizontal ground line at base of logo — anchors it to the scene ─
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
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<SlotMenuState>();
}

fn handle_title_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut commands: Commands,
    mut slot_state: ResMut<SlotMenuState>,
    mut prompt_q: Query<&mut TextColor, With<PromptText>>,
    time: Res<Time>,
) {
    // Animate prompt pulse
    if let Ok(mut color) = prompt_q.get_single_mut() {
        let alpha = 0.4 + 0.6 * (time.elapsed_secs() * 2.5).sin().max(0.0);
        color.0 = Color::srgba(0.95, 0.7, 0.25, alpha);
    }

    if slot_state.open { return; }

    let gp = gamepads.iter().next();
    let gp_confirm = gp.map_or(false, |g| g.just_pressed(GamepadButton::South) || g.just_pressed(GamepadButton::Start));
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) || gp_confirm {
        slot_state.open = true;
        spawn_slot_menu(&mut commands);
    }
}

fn spawn_slot_menu(commands: &mut Commands) {
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
            // Header
            parent.spawn((
                Text::new("Choose Your Path"),
                TextFont { font_size: 30.0, ..default() },
                TextColor(Color::srgb(0.95, 0.7, 0.25)),
            ));

            parent.spawn(Node { height: Val::Px(16.0), ..default() });

            // 3 save slots
            for i in 0..3 {
                let (label, detail_color) = if let Some(ref save) = slots[i] {
                    let mins = (save.time_played / 60.0) as i32;
                    let secs = (save.time_played % 60.0) as i32;
                    (
                        format!(
                            "[{}]  Slot {} - Floor {} | {}g | {}:{:02}",
                            i + 1, i + 1, save.floor, save.gold, mins, secs
                        ),
                        Color::srgb(0.85, 0.65, 0.25),
                    )
                } else {
                    (
                        format!("[{}]  Slot {} - Empty (New Game)", i + 1, i + 1),
                        Color::srgb(0.55, 0.45, 0.35),
                    )
                };

                parent.spawn((
                    Text::new(label),
                    TextFont { font_size: 19.0, ..default() },
                    TextColor(detail_color),
                ));
            }

            parent.spawn(Node { height: Val::Px(20.0), ..default() });

            parent.spawn((
                Text::new("1/2/3 or X/Y/A to select  |  DEL to erase  |  ESC/B to go back"),
                TextFont { font_size: 13.0, ..default() },
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
) {
    let Some(state) = slot_state else { return };
    if !state.open { return };
    let gp = gamepads.iter().next();

    // ESC / gamepad East(B) closes the slot menu
    let back = keys.just_pressed(KeyCode::Escape) || gp.map_or(false, |g| g.just_pressed(GamepadButton::East));
    if back {
        for e in &ui_q {
            commands.entity(e).despawn_recursive();
        }
        commands.insert_resource(SlotMenuState { open: false });
        return;
    }

    // DEL + digit to erase a slot (keyboard only — gamepad: Select + face button)
    let digit_keys = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3];
    let gp_slot_buttons = [GamepadButton::West, GamepadButton::North, GamepadButton::South];
    let deleting = keys.pressed(KeyCode::Delete) || gp.map_or(false, |g| g.pressed(GamepadButton::Select));

    if deleting {
        for (i, &key) in digit_keys.iter().enumerate() {
            let gp_pressed = gp.map_or(false, |g| g.just_pressed(gp_slot_buttons[i]));
            if keys.just_pressed(key) || gp_pressed {
                delete_slot(i);
                for e in &ui_q {
                    commands.entity(e).despawn_recursive();
                }
                spawn_slot_menu(&mut commands);
                return;
            }
        }
    }

    // Select slot: keyboard 1/2/3 or gamepad X/Y/A (West/North/South)
    for (i, &key) in digit_keys.iter().enumerate() {
        let gp_pressed = !deleting && gp.map_or(false, |g| g.just_pressed(gp_slot_buttons[i]));
        if keys.just_pressed(key) || gp_pressed {
            active_slot.0 = i;

            if let Some(save) = load_slot(i) {
                // Continue: load save and go straight to Playing (skip WellIntro)
                commands.insert_resource(LoadedSave(save));
                next_state.set(GameState::Playing);
            } else {
                // New game: go through WellIntro
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
struct FallingParticle {
    vy: f32,
    lifetime: f32,
}

#[derive(Resource)]
struct IntroState {
    phase: IntroPhase,
    timer: f32,
    player_start_y: f32,
}

#[derive(PartialEq)]
enum IntroPhase {
    WalkToWell,
    JumpIn,
    FallDarkness,
    LandInCave,
}

fn setup_well_intro(mut commands: Commands) {
    let ground_y = -VIEWPORT_H / 2.0 + 50.0;
    let well_base_y = ground_y + 52.5;

    commands.insert_resource(IntroState {
        phase: IntroPhase::WalkToWell,
        timer: 0.0,
        player_start_y: well_base_y + 16.0,
    });

    commands.spawn((Camera2d, IntroEntity));

    // Sky
    commands.spawn((
        Sprite { color: Color::srgb(0.06, 0.03, 0.02), custom_size: Some(Vec2::new(VIEWPORT_W, VIEWPORT_H)), ..default() },
        Transform::from_xyz(0.0, 0.0, Z_BACKGROUND), IntroEntity,
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

    // Well
    commands.spawn((
        Sprite { color: Color::srgb(0.35, 0.3, 0.25), custom_size: Some(Vec2::new(64.0, 28.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 14.0, Z_BACKGROUND + 5.0), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.01, 0.01, 0.02), custom_size: Some(Vec2::new(48.0, 20.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 14.0, Z_BACKGROUND + 5.5), IntroEntity,
    ));
    commands.spawn((
        Sprite { color: Color::srgb(0.4, 0.35, 0.28), custom_size: Some(Vec2::new(70.0, 8.0)), ..default() },
        Transform::from_xyz(0.0, well_base_y + 30.0, Z_BACKGROUND + 6.0), IntroEntity,
    ));

    // Intro player
    commands.spawn((
        Transform::from_xyz(-280.0, well_base_y + 16.0, Z_PLAYER),
        Visibility::Visible,
        IntroEntity,
        IntroPlayer,
    )).with_children(|p| {
        // Body
        p.spawn((
            Sprite { color: Color::srgb(0.50, 0.30, 0.12), custom_size: Some(Vec2::new(14.0, 14.0)), ..default() },
            Transform::from_xyz(0.0, 0.0, 0.1), IntroEntity, IntroPlayerPart,
        ));
        // Belt
        p.spawn((
            Sprite { color: Color::srgb(0.45, 0.30, 0.15), custom_size: Some(Vec2::new(14.0, 3.0)), ..default() },
            Transform::from_xyz(0.0, -5.0, 0.15), IntroEntity, IntroPlayerPart,
        ));
        // Head
        p.spawn((
            Sprite { color: Color::srgb(0.78, 0.62, 0.48), custom_size: Some(Vec2::new(12.0, 11.0)), ..default() },
            Transform::from_xyz(0.0, 12.0, 0.2), IntroEntity, IntroPlayerPart,
        )).with_children(|head| {
            head.spawn((
                Sprite { color: Color::srgb(0.9, 0.92, 0.95), custom_size: Some(Vec2::new(2.5, 3.0)), ..default() },
                Transform::from_xyz(-2.5, 0.5, 0.1), IntroEntity, IntroPlayerPart,
            ));
            head.spawn((
                Sprite { color: Color::srgb(0.9, 0.92, 0.95), custom_size: Some(Vec2::new(2.5, 3.0)), ..default() },
                Transform::from_xyz(2.5, 0.5, 0.1), IntroEntity, IntroPlayerPart,
            ));
            head.spawn((
                Sprite { color: Color::srgb(0.40, 0.22, 0.10), custom_size: Some(Vec2::new(14.0, 5.0)), ..default() },
                Transform::from_xyz(0.0, 4.5, 0.15), IntroEntity, IntroPlayerPart,
            ));
        });
        // Legs
        p.spawn((
            Sprite { color: Color::srgb(0.28, 0.22, 0.16), custom_size: Some(Vec2::new(5.0, 10.0)), ..default() },
            Transform::from_xyz(-3.5, -12.0, 0.0), IntroEntity, IntroPlayerPart, IntroLegL,
        ));
        p.spawn((
            Sprite { color: Color::srgb(0.26, 0.20, 0.14), custom_size: Some(Vec2::new(5.0, 10.0)), ..default() },
            Transform::from_xyz(3.5, -12.0, 0.0), IntroEntity, IntroPlayerPart, IntroLegR,
        ));
        // Sword
        p.spawn((
            Sprite { color: Color::srgb(0.68, 0.70, 0.74), custom_size: Some(Vec2::new(3.0, 16.0)), ..default() },
            Transform::from_xyz(10.0, 3.0, 0.35), IntroEntity, IntroPlayerPart,
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
    mut player_q: Query<&mut Transform, (With<IntroPlayer>, Without<IntroLegL>, Without<IntroLegR>)>,
    mut parts_q: Query<&mut Sprite, (With<IntroPlayerPart>, Without<DarknessOverlay>)>,
    mut legl_q: Query<&mut Transform, (With<IntroLegL>, Without<IntroPlayer>, Without<IntroLegR>)>,
    mut legr_q: Query<&mut Transform, (With<IntroLegR>, Without<IntroPlayer>, Without<IntroLegL>)>,
    mut darkness_q: Query<&mut Sprite, (With<DarknessOverlay>, Without<IntroPlayerPart>)>,
    mut next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    state.timer += dt;

    let Ok(mut p_tf) = player_q.get_single_mut() else { return };

    match state.phase {
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

            if p_tf.translation.x >= -5.0 {
                p_tf.translation.x = 0.0;
                if let Ok(mut ll) = legl_q.get_single_mut() {
                    ll.translation = Vec3::new(-3.5, -12.0, 0.0);
                }
                if let Ok(mut rl) = legr_q.get_single_mut() {
                    rl.translation = Vec3::new(3.5, -12.0, 0.0);
                }
                state.phase = IntroPhase::JumpIn;
                state.timer = 0.0;
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

            if state.timer > 0.3 && state.timer < 1.8 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for _ in 0..2 {
                    let px = rng.gen_range(-80.0..80.0_f32);
                    let speed = rng.gen_range(300.0..500.0_f32);
                    let br = rng.gen_range(0.15..0.35_f32);
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(br, br * 0.9, br * 0.7, 0.7),
                            custom_size: Some(Vec2::new(3.0, rng.gen_range(6.0..14.0))),
                            ..default()
                        },
                        Transform::from_xyz(px, VIEWPORT_H / 2.0 + 10.0, Z_HUD - 0.5),
                        IntroEntity,
                        FallingParticle { vy: -speed, lifetime: 1.2 },
                    ));
                }
            }

            if state.timer >= 2.2 {
                state.phase = IntroPhase::LandInCave;
                state.timer = 0.0;
            }
        }

        IntroPhase::LandInCave => {
            if let Ok(mut ds) = darkness_q.get_single_mut() {
                if state.timer < 0.1 {
                    ds.color = Color::srgba(0.12, 0.08, 0.05, 0.85);
                } else {
                    ds.color = Color::srgba(0.0, 0.0, 0.0, 1.0);
                }
            }

            if state.timer >= 0.6 {
                next_state.set(GameState::Playing);
            }
        }
    }
}

fn cleanup_well_intro(
    mut commands: Commands,
    q: Query<Entity, With<IntroEntity>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
    commands.remove_resource::<IntroState>();
}

fn update_falling_particles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FallingParticle)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut p) in &mut query {
        tf.translation.y += p.vy * dt;
        p.lifetime -= dt;
        if p.lifetime <= 0.0 || tf.translation.y < -VIEWPORT_H / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}
