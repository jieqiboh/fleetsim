use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub const NUM_ROBOTS: usize = 5;
pub const NUM_TASKS: usize = 12;

#[derive(Component)]
pub struct Robot {
    pub id: usize,
}

#[derive(Component, Default)]
pub struct RobotAssignment {
    pub task_id: Option<usize>,
}

#[derive(Component, Default)]
pub struct RobotPath {
    pub points: Vec<Vec3>,
}

#[derive(Component)]
pub struct Task {
    pub id: usize,
    pub assigned_to: Option<usize>,
    pub completed: bool,
}

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct FlyCamera {
    pub speed: f32,
    pub sensitivity: f32,
}

#[derive(Resource)]
pub struct Simulation {
    pub now: f64,
    pub events: BinaryHeap<Event>,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            now: 0.0,
            events: BinaryHeap::new(),
        }
    }

    pub fn schedule(&mut self, event: Event) {
        self.events.push(event);
    }
}

#[derive(Debug)]
pub struct Event {
    pub timestamp: f64,
    pub event_type: EventType,
}

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
pub enum EventType {
    MoveRobot {
        robot_id: usize,
        target: Vec3,
        task_id: Option<usize>,
    },
}

pub fn pseudo_random(seed: f32) -> f32 {
    let x = (seed * 12.9898).sin() * 43_758.547;
    x - x.floor()
}

pub fn robot_start_position(robot_id: usize) -> Vec3 {
    Vec3::new(robot_id as f32 * 2.0 - 4.0, 0.5, 0.0)
}

pub fn task_position(task_id: usize) -> Vec3 {
    Vec3::new(
        pseudo_random(task_id as f32 + 10.0) * 14.0 - 7.0,
        0.25,
        pseudo_random(task_id as f32 + 42.0) * 14.0 - 7.0,
    )
}
