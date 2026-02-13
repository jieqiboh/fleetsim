use bevy::prelude::*;

use crate::model::{
    Event, EventType, NUM_ROBOTS, NUM_TASKS, Robot, RobotAssignment, RobotPath, Simulation, Task,
    robot_start_position, task_position,
};

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
        Transform::from_scale(Vec3::splat(20.0)),
    ));

    for i in 0..NUM_ROBOTS {
        let start = robot_start_position(i);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.3))),
            Transform::from_translation(start),
            Robot { id: i },
            RobotAssignment::default(),
            RobotPath {
                points: vec![start],
            },
        ));
    }

    for task_id in 0..NUM_TASKS {
        let pos = task_position(task_id);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
            Transform::from_translation(pos).with_scale(Vec3::splat(0.3)),
            Task {
                id: task_id,
                assigned_to: None,
                completed: false,
            },
        ));
    }
}

pub fn run_simulation(
    time: Res<Time>,
    mut sim: ResMut<Simulation>,
    mut robots: Query<(&Robot, &mut Transform, &mut RobotAssignment, &mut RobotPath)>,
    mut tasks: Query<&mut Task>,
) {
    sim.now += time.delta_secs_f64();

    while let Some(event) = sim.events.peek() {
        if event.timestamp > sim.now {
            break;
        }

        let event = sim.events.pop().unwrap();
        match event.event_type {
            EventType::MoveRobot {
                robot_id,
                target,
                task_id,
            } => {
                for (robot, mut transform, mut assignment, mut path) in &mut robots {
                    if robot.id != robot_id {
                        continue;
                    }

                    let from = transform.translation;
                    transform.translation = target;
                    if path.points.last().copied() != Some(from) {
                        path.points.push(from);
                    }
                    path.points.push(target);
                    assignment.task_id = None;
                }

                if let Some(task_id) = task_id {
                    for mut task in &mut tasks {
                        if task.id == task_id {
                            task.completed = true;
                            task.assigned_to = None;
                            break;
                        }
                    }
                }
            }
        }
    }
}

pub fn allocate_tasks(
    mut sim: ResMut<Simulation>,
    mut robots: Query<(Entity, &Robot, &Transform, &mut RobotAssignment)>,
    mut tasks: Query<(Entity, &mut Task, &Transform)>,
) {
    let now = sim.now;

    for (_, robot, transform, mut assignment) in &mut robots {
        if assignment.task_id.is_some() {
            continue;
        }

        let mut best_task_entity = None;
        let mut best_task_id = 0usize;
        let mut best_task_pos = Vec3::ZERO;
        let mut best_dist_sq = f32::MAX;

        for (task_entity, task, task_transform) in &mut tasks {
            if task.completed || task.assigned_to.is_some() {
                continue;
            }

            let task_pos = task_transform.translation;
            let dist_sq = transform.translation.distance_squared(task_pos);
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_task_entity = Some(task_entity);
                best_task_id = task.id;
                best_task_pos = task_pos;
            }
        }

        let Some(task_entity) = best_task_entity else {
            continue;
        };

        assignment.task_id = Some(best_task_id);
        if let Ok((_, mut task, _)) = tasks.get_mut(task_entity) {
            task.assigned_to = Some(robot.id);
        }

        let travel_time = (best_dist_sq.sqrt() / 4.0).max(0.5) as f64;
        sim.schedule(Event {
            timestamp: now + travel_time,
            event_type: EventType::MoveRobot {
                robot_id: robot.id,
                target: Vec3::new(best_task_pos.x, 0.5, best_task_pos.z),
                task_id: Some(best_task_id),
            },
        });
    }
}

pub fn draw_robot_paths(mut gizmos: Gizmos, query: Query<&RobotPath>) {
    for path in &query {
        if path.points.len() < 2 {
            continue;
        }

        for segment in path.points.windows(2) {
            let a = segment[0] + Vec3::Y * 0.05;
            let b = segment[1] + Vec3::Y * 0.05;
            gizmos.line(a, b, Color::srgb(0.1, 0.7, 1.0));
        }
    }
}
