use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Simulation::new())
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, run_simulation))
        .run();
}

#[derive(Component)]
struct Robot {
    id: usize,
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
    MoveRobot { robot_id: usize, target: Vec3 },
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
    mut sim: ResMut<Simulation>,
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
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::default())),
            MeshMaterial3d(materials.add(Color::srgb(0.2, 0.7, 0.3))),
            Transform::from_xyz(i as f32 * 2.0 - 4.0, 0.5, 0.0),
            Robot { id: i },
        ));

        // Schedule first move event
        sim.schedule(Event {
            timestamp: 2.0 + i as f64,
            event_type: EventType::MoveRobot {
                robot_id: i,
                target: Vec3::new(0.0, 0.5, 5.0),
            },
        });
    }
}

// ---------- SIMULATION SYSTEM ----------

fn run_simulation(
    time: Res<Time>,
    mut sim: ResMut<Simulation>,
    mut query: Query<(&Robot, &mut Transform)>,
) {
    // Advance simulation clock
    sim.now += time.delta_secs_f64();

    while let Some(event) = sim.events.peek() {
        if event.timestamp > sim.now {
            break;
        }

        let event = sim.events.pop().unwrap();

        match event.event_type {
            EventType::MoveRobot { robot_id, target } => {
                for (robot, mut transform) in &mut query {
                    if robot.id == robot_id {
                        transform.translation = target;
                    }
                }

                // Schedule next move (looping)
                let now = sim.now;
                let seed = now as f32 + robot_id as f32;
                sim.schedule(Event {
                    timestamp: now + 3.0,
                    event_type: EventType::MoveRobot {
                        robot_id,
                        target: Vec3::new(
                            pseudo_random(seed) * 8.0 - 4.0,
                            0.5,
                            pseudo_random(seed + 1.2345) * 8.0 - 4.0,
                        ),
                    },
                });
            }
        }
    }
}

// ---------- CAMERA SYSTEM (UNCHANGED) ----------

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
