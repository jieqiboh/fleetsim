use bevy::prelude::*;

// Camera controls and camera entity setup.
mod camera;
// Shared components/resources and helper functions.
mod model;
// World spawning and simulation systems.
mod simulation;
// UI setup and interaction systems.
mod ui;

use crate::model::Simulation;

// Entry point: wire plugins, resources, startup systems, and frame systems.
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Simulation::new())
        .add_systems(
            Startup,
            (
                camera::setup_camera,
                simulation::setup_world,
                ui::setup_restart_ui,
            ),
        )
        .add_systems(
            Update,
            (
                camera::camera_movement,
                simulation::run_simulation,
                simulation::allocate_tasks,
                simulation::draw_robot_paths,
                ui::restart_button_system,
            ),
        )
        .run();
}
