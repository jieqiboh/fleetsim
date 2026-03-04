use bevy::prelude::*;

use crate::model::{
    ActiveScenario, Robot, RobotVisualMaterials, ScenarioConfig, Scenario, Simulation, Task,
};
use crate::simulation::spawn_scenario;

/// Marker for the restart button.
#[derive(Component)]
pub struct RestartButton;

/// Marks a button that switches to a specific scenario.
#[derive(Component)]
pub struct ScenarioButton(pub Scenario);

/// Spawns the overlay UI camera and the button row.
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

    // Horizontal row container in the top-left corner.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(12.0),
                left: px(12.0),
                flex_direction: FlexDirection::Row,
                column_gap: px(8.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            spawn_scenario_button(parent, "Small", Scenario::Small);
            spawn_scenario_button(parent, "Warehouse", Scenario::Warehouse);
            spawn_scenario_button(parent, "Stress Test", Scenario::StressTest);
            spawn_restart_button(parent);
        });
}

fn spawn_scenario_button(parent: &mut ChildSpawnerCommands, label: &str, scenario: Scenario) {
    parent
        .spawn((
            Button,
            ScenarioButton(scenario),
            Node {
                min_height: px(36.0),
                padding: UiRect::axes(px(12.0), px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
            ZIndex(10),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_restart_button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            Button,
            RestartButton,
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
        .with_children(|btn| {
            btn.spawn((
                Text::new("Restart Simulation"),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

/// Despawns all robots and tasks, then respawns from `config`.
#[allow(clippy::too_many_arguments)]
fn reset_simulation(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    sim: &mut Simulation,
    visuals: &RobotVisualMaterials,
    robot_entities: &[Entity],
    task_entities: &[Entity],
    config: &ScenarioConfig,
) {
    for &e in robot_entities.iter().chain(task_entities) {
        commands.entity(e).despawn();
    }
    sim.now = 0.0;
    sim.events.clear();
    spawn_scenario(commands, meshes, materials, visuals, config);
}

/// Switches to a new scenario when a scenario button is pressed.
#[allow(clippy::too_many_arguments)]
pub fn scenario_button_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visuals: Res<RobotVisualMaterials>,
    mut active: ResMut<ActiveScenario>,
    mut sim: ResMut<Simulation>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &ScenarioButton),
        Changed<Interaction>,
    >,
    robot_entities: Query<Entity, With<Robot>>,
    task_entities: Query<Entity, With<Task>>,
) {
    let mut new_scenario = None;

    for (interaction, mut color, scenario_btn) in &mut button_query {
        match *interaction {
            Interaction::Pressed => {
                *color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
                new_scenario = Some(scenario_btn.0);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.35, 0.35, 0.35));
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
        }
    }

    let Some(scenario) = new_scenario else { return };
    active.0 = scenario;

    let robots: Vec<Entity> = robot_entities.iter().collect();
    let tasks: Vec<Entity> = task_entities.iter().collect();
    let config = ScenarioConfig::build(scenario);
    reset_simulation(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut sim,
        &visuals,
        &robots,
        &tasks,
        &config,
    );
}

/// Restarts the current scenario when the restart button is pressed.
#[allow(clippy::too_many_arguments)]
pub fn restart_button_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visuals: Res<RobotVisualMaterials>,
    active: Res<ActiveScenario>,
    mut sim: ResMut<Simulation>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RestartButton>),
    >,
    robot_entities: Query<Entity, With<Robot>>,
    task_entities: Query<Entity, With<Task>>,
) {
    let mut should_restart = false;

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

    if !should_restart { return; }

    let robots: Vec<Entity> = robot_entities.iter().collect();
    let tasks: Vec<Entity> = task_entities.iter().collect();
    let config = ScenarioConfig::build(active.0);
    reset_simulation(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut sim,
        &visuals,
        &robots,
        &tasks,
        &config,
    );
}
