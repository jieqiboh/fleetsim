use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Simulation::new())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                camera_movement,
                run_simulation,
                allocate_tasks,
                draw_robot_paths,
            ),
        )
        .run();
}

#[derive(Component)]
struct Robot {
    id: usize,
}

#[derive(Component, Default)]
struct RobotAssignment {
    task_id: Option<usize>,
}

#[derive(Component, Default)]
struct RobotPath {
    points: Vec<Vec3>,
}

#[derive(Component)]
struct Task {
    id: usize,
    assigned_to: Option<usize>,
    completed: bool,
}

#[derive(Component)]
struct FlyCamera {
    speed: f32,
    sensitivity: f32,
}

#[derive(Resource)]
struct Simulation {
    now: f64,
    events: BinaryHeap<Event>,
}

impl Simulation {
    fn new() -> Self {
        Self {
            now: 0.0,
            events: BinaryHeap::new(),
        }
    }

    fn schedule(&mut self, event: Event) {
        self.events.push(event);
    }
}

#[derive(Debug)]
struct Event {
    timestamp: f64,
    event_type: EventType,
}

// Reverse ordering so smallest timestamp pops first
impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        other.timestamp.partial_cmp(&self.timestamp).unwrap()
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for Event {}

#[derive(Debug)]
enum EventType {
    MoveRobot {
        robot_id: usize,
        target: Vec3,
        task_id: Option<usize>,
    },
}

fn pseudo_random(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43_758.547;
    x - x.floor()
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

    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
        Transform::from_scale(Vec3::splat(20.0)),
    ));

    // Spawn robots
    for i in 0..5 {
        let start = Vec3::new(i as f32 * 2.0 - 4.0, 0.5, 0.0);
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

    // Spawn simple tasks to be allocated to robots.
    for task_id in 0..12 {
        let x = pseudo_random(task_id as f32 + 10.0) * 14.0 - 7.0;
        let z = pseudo_random(task_id as f32 + 42.0) * 14.0 - 7.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.9, 0.3, 0.2))),
            Transform::from_xyz(x, 0.25, z).with_scale(Vec3::splat(0.3)),
            Task {
                id: task_id,
                assigned_to: None,
                completed: false,
            },
        ));
    }
}

// ---------- SIMULATION SYSTEM ----------

fn run_simulation(
    time: Res<Time>,
    mut sim: ResMut<Simulation>,
    mut robots: Query<(&Robot, &mut Transform, &mut RobotAssignment, &mut RobotPath)>,
    mut tasks: Query<&mut Task>,
) {
    // Advance simulation clock
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
                    if robot.id == robot_id {
                        let from = transform.translation;
                        transform.translation = target;
                        if path.points.last().copied() != Some(from) {
                            path.points.push(from);
                        }
                        path.points.push(target);
                        assignment.task_id = None;
                    }
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

fn allocate_tasks(
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

fn draw_robot_paths(mut gizmos: Gizmos, query: Query<&RobotPath>) {
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

// ---------- CAMERA SYSTEM ----------

fn camera_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &FlyCamera)>,
) {
    let Ok((mut transform, settings)) = query.single_mut() else {
        return;
    };

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

    if mouse_buttons.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            let yaw = Quat::from_rotation_y(-ev.delta.x * settings.sensitivity * 0.01);
            let pitch = Quat::from_rotation_x(-ev.delta.y * settings.sensitivity * 0.01);

            transform.rotation = yaw * transform.rotation;
            transform.rotation = transform.rotation * pitch;
        }
    }
}
