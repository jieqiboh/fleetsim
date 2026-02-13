use bevy::prelude::*;
mod camera;
mod model;
mod simulation;
mod ui;

use crate::model::Simulation;

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
