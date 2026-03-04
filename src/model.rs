use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// The available simulation presets.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Scenario {
    Small,
    Warehouse,
    StressTest,
}

/// Resolved spawn data for a scenario: robot and task world positions.
pub struct ScenarioConfig {
    pub robot_positions: Vec<Vec3>,
    pub task_positions: Vec<Vec3>,
}

impl ScenarioConfig {
    pub fn build(scenario: Scenario) -> Self {
        match scenario {
            Scenario::Small => Self {
                robot_positions: (0..3).map(robot_start_position).collect(),
                task_positions: (0..8).map(task_position).collect(),
            },
            Scenario::Warehouse => {
                // 5 robots evenly spaced in a row at the bottom edge.
                let robot_positions = (0..5)
                    .map(|i| Vec3::new(i as f32 * 3.0 - 6.0, 0.5, -6.0))
                    .collect();
                // 25 tasks in a 5-column × 5-row grid centred on the arena.
                let task_positions = (0..25)
                    .map(|i| {
                        let col = (i % 5) as f32;
                        let row = (i / 5) as f32;
                        Vec3::new(col * 3.0 - 6.0, 0.25, row * 2.5 - 5.0)
                    })
                    .collect();
                Self { robot_positions, task_positions }
            }
            Scenario::StressTest => {
                // 10 robots in two rows of 5, well within the plane bounds (-10..10).
                let robot_positions = (0..10)
                    .map(|i| {
                        let col = (i % 5) as f32;
                        let row = (i / 5) as f32;
                        Vec3::new(col * 4.0 - 8.0, 0.5, row * 2.0 + 7.0)
                    })
                    .collect();
                Self {
                    robot_positions,
                    task_positions: (0..30).map(task_position).collect(),
                }
            }
        }
    }
}

/// Active scenario resource — determines which preset is currently loaded.
#[derive(Resource)]
pub struct ActiveScenario(pub Scenario);

#[derive(Component)]
/// A robot entity participating in the simulation.
pub struct Robot {
    /// Stable robot identifier used for task assignment and event routing.
    pub id: usize,
}

#[derive(Component, Default)]
/// Tracks which task (if any) is currently assigned to a robot.
pub struct RobotAssignment {
    /// The assigned task id, or `None` when the robot is idle.
    pub task_id: Option<usize>,
}

#[derive(Component, Default)]
/// Stores the sequence of positions visited by a robot.
pub struct RobotPath {
    /// Ordered waypoints used to draw a path line in the scene.
    pub points: Vec<Vec3>,
}

#[derive(Component, Default)]
/// Stores whether a robot is currently in a collision state.
pub struct CollisionState {
    /// True when the robot is within collision distance of another robot.
    pub is_colliding: bool,
}

#[derive(Component)]
/// A unit of work that can be allocated to a robot.
pub struct Task {
    /// Stable task identifier.
    pub id: usize,
    /// Robot id currently assigned to this task, if any.
    pub assigned_to: Option<usize>,
    /// Whether this task has already been completed.
    pub completed: bool,
}

#[derive(Component)]
/// Runtime movement settings for the free-fly camera.
pub struct FlyCamera {
    /// Camera translation speed in world units per second.
    pub speed: f32,
    /// Mouse-look sensitivity multiplier.
    pub sensitivity: f32,
}

#[derive(Resource, Clone)]
/// Shared robot materials for normal and collision-highlighted states.
pub struct RobotVisualMaterials {
    /// Material shown when a robot is not colliding.
    pub normal: Handle<StandardMaterial>,
    /// Material shown when a robot is colliding.
    pub collision: Handle<StandardMaterial>,
}

#[derive(Resource)]
/// Global simulation state and pending event queue.
pub struct Simulation {
    /// Current simulation clock time in seconds.
    pub now: f64,
    /// Priority queue of future events, ordered by earliest timestamp.
    pub events: BinaryHeap<Event>,
}

impl Simulation {
    /// Creates a simulation with time at `0.0` and an empty event queue.
    pub fn new() -> Self {
        Self {
            now: 0.0,
            events: BinaryHeap::new(),
        }
    }

    /// Pushes a new event into the priority queue.
    pub fn schedule(&mut self, event: Event) {
        self.events.push(event);
    }
}

#[derive(Debug)]
/// A scheduled action that should execute at a specific simulation time.
pub struct Event {
    /// Simulation timestamp at which this event becomes executable.
    pub timestamp: f64,
    /// Concrete payload describing what action to perform.
    pub event_type: EventType,
}

impl Ord for Event {
    /// Reverses sort order so the smallest timestamp is popped first.
    fn cmp(&self, other: &Self) -> Ordering {
        other.timestamp.partial_cmp(&self.timestamp).unwrap_or(Ordering::Equal)
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
/// Concrete event payloads executed by the simulation loop.
pub enum EventType {
    /// Moves one robot to a target position and optionally marks a task complete.
    MoveRobot {
        /// Identifier of the robot to move.
        robot_id: usize,
        /// Destination position in world space.
        target: Vec3,
        /// Task id to complete after arrival, if the move came from allocation.
        task_id: Option<usize>,
    },
}

/// Lightweight deterministic pseudo-random helper used for task placement.
fn pseudo_random(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43_758.547;
    x - x.floor()
}

/// Returns the deterministic start position for a robot id.
fn robot_start_position(robot_id: usize) -> Vec3 {
    Vec3::new(robot_id as f32 * 2.0 - 4.0, 0.5, 0.0)
}

/// Returns the deterministic world position for a task id.
fn task_position(task_id: usize) -> Vec3 {
    Vec3::new(
        pseudo_random(task_id as f32 + 10.0) * 14.0 - 7.0,
        0.25,
        pseudo_random(task_id as f32 + 42.0) * 14.0 - 7.0,
    )
}
