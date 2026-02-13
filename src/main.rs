use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, camera_movement)
        .run();
}

#[derive(Component)]
struct Robot;

#[derive(Component)]
struct FlyCamera {
    speed: f32,
    sensitivity: f32,
}

// ---------- SETUP ----------

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 8.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCamera {
            speed: 10.0,
            sensitivity: 0.2,
        },
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
        Transform::from_scale(Vec3::splat(20.0)),
    ));

    // Spawn some robot cubes
    for i in 0..5 {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.3))),
            Transform::from_xyz(i as f32 * 2.0 - 4.0, 0.5, 0.0),
            Robot,
        ));
    }
}

// ---------- CAMERA CONTROLS ----------

fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &FlyCamera)>,
) {
    let Ok((mut transform, settings)) = query.single_mut() else {
        return;
    };

    // Keyboard movement
    let mut direction = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        direction += *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction -= *transform.forward();
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= *transform.right();
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += *transform.right();
    }
    if keyboard.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keyboard.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    if direction.length_squared() > 0.0 {
        transform.translation += direction.normalize() * settings.speed * time.delta_secs();
    }

    // Mouse look (hold right mouse button)
    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            let yaw = Quat::from_rotation_y(-ev.delta.x * settings.sensitivity * 0.01);
            let pitch = Quat::from_rotation_x(-ev.delta.y * settings.sensitivity * 0.01);

            transform.rotation = yaw * transform.rotation;
            transform.rotation = transform.rotation * pitch;
        }
    }
}
