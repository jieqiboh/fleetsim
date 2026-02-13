use bevy::prelude::*;

use crate::model::{
    RestartButton, Robot, RobotAssignment, RobotPath, Simulation, Task, robot_start_position,
    task_position,
};

/// Spawns the overlay UI camera and a restart button.
pub fn setup_restart_ui(mut commands: Commands) {
    // Render UI after the 3D camera pass.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: bevy::camera::ClearColorConfig::None,
            ..default()
        },
    ));

    commands
        .spawn((
            // Absolute-positioned container in the top-left corner.
            Node {
                position_type: PositionType::Absolute,
                top: px(12.0),
                left: px(12.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    RestartButton,
                    // Basic button sizing and label alignment.
                    Node {
                        min_width: px(170.0),
                        min_height: px(36.0),
                        padding: UiRect::axes(px(12.0), px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.18, 0.45, 0.85)),
                    ZIndex(10),
                ))
                .with_children(|button| {
                    button.spawn((
                        // Visible button label.
                        Text::new("Restart Simulation"),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Handles restart button interaction and resets simulation state when pressed.
pub fn restart_button_system(
    mut sim: ResMut<Simulation>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    mut robots: Query<
        (&Robot, &mut Transform, &mut RobotAssignment, &mut RobotPath),
        (With<Robot>, Without<Task>),
    >,
    mut tasks: Query<(&mut Task, &mut Transform), (With<Task>, Without<Robot>)>,
) {
    let mut should_restart = false;

    // Update button colors by interaction state and detect click.
    for (interaction, mut color) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.10, 0.30, 0.60));
                should_restart = true;
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.24, 0.52, 0.95));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.18, 0.45, 0.85));
            }
        }
    }

    if !should_restart {
        return;
    }

    // Reset simulation timeline and clear pending events.
    sim.now = 0.0;
    sim.events.clear();

    // Reset robots to initial position and clear assignments/path history.
    for (robot, mut transform, mut assignment, mut path) in &mut robots {
        let start = robot_start_position(robot.id);
        transform.translation = start;
        assignment.task_id = None;
        path.points.clear();
        path.points.push(start);
    }

    // Reset task state and deterministic positions.
    for (mut task, mut transform) in &mut tasks {
        task.assigned_to = None;
        task.completed = false;
        transform.translation = task_position(task.id);
    }
}
