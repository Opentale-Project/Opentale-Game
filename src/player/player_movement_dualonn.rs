use std::cmp::PartialEq;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowMode};
use bevy::render::mesh::PlaneMeshBuilder;
use avian3d::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_ichun::{IchunPlugin, input::KccInputConfig, kcc::Kcc, movement::KccMovementConfig};

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct FreeCamera;

#[derive(Component)]
struct FreeCameraController {
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
}

#[derive(Resource)]
struct CameraControl {
    pitch: f32,
    sensitivity: f32,
}

#[derive(Resource)]
struct FreeCamModes{
    mode: String,
}

#[derive(Resource)]
struct CameraOffsets{
    fp: Vec3,
    tp: Vec3,
    os: Vec3,
}
#[derive(Resource)]
struct PlayerDefaults{
    pub player_height: f32,
    pub player_radius: f32,
}
#[derive(Resource)]
enum CameraPOV {
    FirstPerson,
    ThirdPerson,
    OverShoulder,
}
#[derive(Resource)]
struct Config {
    keybind_player_camera_switchmode: KeyCode,
    keybind_player_move_up: KeyCode,
    keybind_player_move_left: KeyCode,
    keybind_player_move_down: KeyCode,
    keybind_player_move_right: KeyCode,
    keybind_freecam_toggle: KeyCode,
    keybind_freecam_move_forward: KeyCode,
    keybind_freecam_move_left: KeyCode,
    keybind_freecam_move_backward: KeyCode,
    keybind_freecam_move_right: KeyCode,
    keybind_freecam_move_up: KeyCode,
    keybind_freecam_move_down: KeyCode,
    keybind_freecam_speed_exslow: KeyCode,
    keybind_freecam_speed_slow: KeyCode,
    keybind_freecam_speed_medium: KeyCode,
    keybind_freecam_speed_fast: KeyCode,
    keybind_freecam_speed_exfast: KeyCode,
    keybind_freecam_speed_insane: KeyCode,
    keybind_freecam_mode_locked: KeyCode,
    keybind_freecam_mode_fllwbody: KeyCode,
    keybind_freecam_mode_fllwhead: KeyCode,
    //keybind_player_interact
    //keybind_ui_menu
    //etc
    free_cam_speed: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybind_player_camera_switchmode: KeyCode::F5,
            keybind_player_move_up: KeyCode::KeyW,
            keybind_player_move_left: KeyCode::KeyA,
            keybind_player_move_down: KeyCode::KeyS,
            keybind_player_move_right: KeyCode::KeyD,
            keybind_freecam_move_forward: KeyCode::KeyW,
            keybind_freecam_move_left: KeyCode::KeyA,
            keybind_freecam_move_backward: KeyCode::KeyS,
            keybind_freecam_move_right: KeyCode::KeyD,
            keybind_freecam_move_up: KeyCode::Space,
            keybind_freecam_move_down: KeyCode::ControlLeft,
            keybind_freecam_toggle: KeyCode::F6,
            keybind_freecam_speed_exslow: KeyCode::Digit1,
            keybind_freecam_speed_slow: KeyCode::Digit2,
            keybind_freecam_speed_medium: KeyCode::Digit3,
            keybind_freecam_speed_fast: KeyCode::Digit4,
            keybind_freecam_speed_exfast: KeyCode::Digit5,
            keybind_freecam_speed_insane: KeyCode::Digit6,
            keybind_freecam_mode_locked: KeyCode::Digit7,
            keybind_freecam_mode_fllwbody: KeyCode::Digit8,
            keybind_freecam_mode_fllwhead: KeyCode::Digit9,
            free_cam_speed: 1.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenTaleMovmentDemo".into(),
                resolution: (1920.0, 1080.0).into(),
                mode: WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
                ..default()
            }),
                ..default()
        }), PhysicsPlugins::default(), IchunPlugin))
        .insert_resource(Config::default())
        .insert_resource(CameraOffsets{
            fp: Vec3::new(0.0, 0.0, 0.0),
            tp: Vec3::new(0.0, 8.0, 10.0),
            os: Vec3::new(6.0, 5.0, 3.0),
        })
        .insert_resource(PlayerDefaults{
            player_height: 1.0,
            player_radius: 0.5,
        }).insert_resource(CameraControl { pitch: 0.0, sensitivity: 0.007 })
    .insert_resource(FreeCamModes { mode: "Off".to_string() })
        .insert_resource(CameraPOV::FirstPerson)
        .add_systems(Startup, (setup, mouselock))
        .add_systems(FixedUpdate, camera_movement)
        .add_systems(Update, (adjust_camera, freecam_system, freecam_look, toggle_player_input_system))
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_defaults: Res<PlayerDefaults>,
) {
    let grass_tex = asset_server.load("grass.png");
    let textured_material = materials.add(StandardMaterial {
        base_color_texture: Some(grass_tex.into()),
        ..Default::default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        GlobalTransform::default(),
        ));

    commands.spawn((
        Player,
        Kcc::default(),
        KccMovementConfig::default(),
        KccInputConfig::default(),
        Collider::capsule(player_defaults.player_radius, player_defaults.player_height),
        Mesh3d(meshes.add(Mesh::from(Capsule3d::new(player_defaults.player_radius, player_defaults.player_height)))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 1.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        GlobalTransform::default(),
    )).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(1.0, 1.0, 1.0)))),
            MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
            Transform::from_xyz(0.0, 1.0, 0.0),
            GlobalTransform::default(),
        ));

        // Camera positioned at eye height relative to cube
        parent.spawn((
            MainCamera,
            Camera3d { ..default() },
            Transform::from_xyz(0.0, 1.0, 0.0), // adjust if needed
        ));
    });

    let free_camera_ent = commands.spawn((
        FreeCamera,
        FreeCameraController {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.1,
        },
        Camera3d {
            ..default()
        },
        Transform::from_xyz(0.0, 8.0, 10.0),
    )).id();

    commands.entity(free_camera_ent).insert(Camera {
        is_active: false,
        ..default()
    });

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(50.0, 0.1, 50.0),
        Mesh3d(meshes.add(PlaneMeshBuilder::from_size(Vec2::splat(50.0))
            .subdivisions(0)
            .build()
        )),
        MeshMaterial3d(textured_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        ));
    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(1.5, 1.5, 1.5),
        Mesh3d(meshes.add(Cuboid::from_size(Vec3::new(1.5, 1.5, 1.5)))),
        MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 1.5/2.0, 10.0),
        GlobalTransform::default(),
    ));
}

fn mouselock(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = windows.single_mut() {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn freecam_look(
    mut mouse_events: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut FreeCameraController, &mut Transform), With<FreeCamera>>,
    mut free_cam_modes: ResMut<FreeCamModes>,
) {
    let Ok((mut controller, mut transform)) = query.single_mut() else { return };

    if !mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    let mut delta = Vec2::ZERO;
    for ev in mouse_events.read(){
        delta += ev.delta;
    }

    let sensitivity = controller.sensitivity.to_radians();

    controller.yaw -= delta.x * sensitivity;
    controller.pitch -= delta.y * sensitivity;

    controller.pitch = controller.pitch.clamp(-89.9_f32.to_radians(), 89.9_f32.to_radians());

    let yaw_rot = Quat::from_rotation_y(controller.yaw);
    let pitch_rot = Quat::from_rotation_x(controller.pitch);
    let rot = yaw_rot * pitch_rot;

    if free_cam_modes.mode == "Free"{
        transform.rotation = rot;
    } else {
        return;
    }
    transform.rotation = rot;
}

fn freecam_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_query: Query<&mut Camera, (With<FreeCamera>, Without<MainCamera>)>,
    mut main_cam_query: Query<&mut Camera, With<MainCamera>>,
    mut cam_transform: Query<&mut Transform, With<FreeCamera>>,
    mut freecam_controller: Query<&mut FreeCameraController>,
    time: Res<Time>,
    config: Res<Config>,
    free_cam_modes: Res<FreeCamModes>,
) {
    let mut main_cam = main_cam_query.single_mut().unwrap();
    let mut free_cam = cam_query.single_mut().unwrap();
    let mut freecam_transform = match cam_transform.single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };
    let Ok(mut controller) = freecam_controller.single_mut() else { return };
    let mut dir = Vec3::ZERO;
    let speed = config.free_cam_speed * 4.0;

    if keys.just_pressed(KeyCode::F6) {

        let is_main_active = main_cam.is_active;

        main_cam.is_active = !is_main_active;
        free_cam.is_active = is_main_active;
    }

    let forward = freecam_transform.forward();
    let right = freecam_transform.right();
    let up = Vec3::Y;
    if free_cam.is_active {
        if keys.pressed(config.keybind_freecam_move_forward) {
            dir += *forward;
        }
        if keys.pressed(config.keybind_freecam_move_left){
            dir -= *right;
        }
        if keys.pressed(config.keybind_freecam_move_backward){
            dir -= *forward;
        }
        if keys.pressed(config.keybind_freecam_move_right){
            dir += *right;
        }
        if keys.pressed(config.keybind_freecam_move_up){
            dir += up;
        }
        if keys.pressed(config.keybind_freecam_move_down){
            dir -= up;
        }
        if dir.length_squared() > 0.0 {
            let local_dir = right * dir.x + Vec3::Y * dir.y + forward * dir.z;
            let move_dir = local_dir.normalize_or_zero();
            if free_cam_modes.mode == "Free" {
                freecam_transform.translation += dir.normalize_or_zero() * speed * time.delta_secs();
            } else if free_cam_modes.mode == "FollowHead" {
                //TODO
            } else if free_cam_modes.mode == "FollowBody"{
                //TODO
            } else if free_cam_modes.mode == "Locked" {
                return;
            }
        }
    }
}

fn camera_movement(
    plyr_query: Query<&Transform, With<Player>>,
    mut cam_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    offsets: Res<CameraOffsets>,
    view_mode: Res<CameraPOV>,
    pitch: Res<CameraControl>,
) {
    let Ok(player_transform) = plyr_query.single() else { return };
    let Ok(mut camera_transform) = cam_query.single_mut() else { return };

    match *view_mode {
        CameraPOV::FirstPerson => {
            // Get player yaw (horizontal rotation)
            let yaw = Quat::from_rotation_y(player_transform.rotation.to_euler(EulerRot::YXZ).0);
            let pitch_quat = Quat::from_rotation_x(pitch.pitch);

            // Combine yaw (from player) and pitch (from camera)
            camera_transform.rotation = yaw * pitch_quat;

            // Position the camera relative to the player
            camera_transform.translation = player_transform.translation + offsets.fp;
        }
        CameraPOV::ThirdPerson => {
            camera_transform.translation = offsets.tp;
            camera_transform.look_at(Vec3::ZERO, Vec3::Y);
        }
        CameraPOV::OverShoulder => {
            camera_transform.translation = offsets.os;
            camera_transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

fn adjust_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut view_mode: ResMut<CameraPOV>,
    mut freecam_mode: ResMut<FreeCamModes>,
    mut config: ResMut<Config>,
    mut camera: ResMut<CameraControl>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {

    for event in mouse_motion_events.read() {
        // Limit pitch to straight up/down
        camera.pitch = (camera.pitch - event.delta.y * camera.sensitivity).clamp(-1.5, 1.5);
        // Apply yaw rotation to player in a separate system (not shown here)
    }

    if keys.just_pressed(config.keybind_player_camera_switchmode) {
        *view_mode = match *view_mode {
            CameraPOV::FirstPerson => CameraPOV::ThirdPerson,
            CameraPOV::ThirdPerson => CameraPOV::OverShoulder,
            CameraPOV::OverShoulder => CameraPOV::FirstPerson,
        }
    }
    if keys.just_pressed(config.keybind_freecam_toggle) {
        freecam_mode.mode = "Free".to_string();
    }
    if keys.just_pressed(config.keybind_freecam_mode_locked) {
        freecam_mode.mode = "Locked".to_string();
    }
    if keys.just_pressed(config.keybind_freecam_mode_fllwbody) {
        freecam_mode.mode = "FollowBody".to_string();
    }
    if keys.just_pressed(config.keybind_freecam_mode_fllwhead) {
        freecam_mode.mode = "FollowHead".to_string();
    }
    if freecam_mode.mode == "Free" {
        if keys.just_pressed(config.keybind_freecam_speed_exslow) {
            config.free_cam_speed = 0.5;
        }
        if keys.just_pressed(config.keybind_freecam_speed_slow) {
            config.free_cam_speed = 1.0;
        }
        if keys.just_pressed(config.keybind_freecam_speed_medium) {
            config.free_cam_speed = 1.5;
        }
        if keys.just_pressed(config.keybind_freecam_speed_fast) {
            config.free_cam_speed = 2.0;
        }
        if keys.just_pressed(config.keybind_freecam_speed_exfast) {
            config.free_cam_speed = 3.0;
        }
        if keys.just_pressed(config.keybind_freecam_speed_insane) {
            config.free_cam_speed = 5.0;
        }
    }
}

fn toggle_player_input_system(
    free_cam_modes: Res<FreeCamModes>,
    mut commands: Commands,
    query: Query<(Entity, Option<&KccInputConfig>), With<Player>>,
) {
    let allow_input = free_cam_modes.mode != "Free";

    for (entity, input_opt) in query.iter() {
        match (allow_input, input_opt.is_some()) {
            (true, false) => {
                commands.entity(entity).insert(KccInputConfig::default());
            }
            (false, true) => {
                commands.entity(entity).remove::<KccInputConfig>();
            }
            _ => {}
        }
    }
}
