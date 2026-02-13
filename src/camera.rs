use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::model::FlyCamera;

/// Spawns the 3D camera with initial transform and movement settings.
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-5.0, 8.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCamera {
            speed: 10.0,
            sensitivity: 0.2,
        },
    ));
}

/// Updates fly camera movement from keyboard and mouse input.
pub fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &FlyCamera)>,
) {
    // This project expects exactly one fly camera.
    let Ok((mut transform, settings)) = query.single_mut() else {
        return;
    };

    // Build movement direction from currently pressed keys.
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

    // Rotate only while right mouse button is held.
    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            let yaw = Quat::from_rotation_y(-ev.delta.x * settings.sensitivity * 0.01);
            let pitch = Quat::from_rotation_x(-ev.delta.y * settings.sensitivity * 0.01);
            transform.rotation = yaw * transform.rotation;
            transform.rotation = transform.rotation * pitch;
        }
    }
}
