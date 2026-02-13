# FleetSim: A Rust + Bevy Multi-Robot Warehouse Simulation
A 3D multi-robot warehouse simulation platform built in Rust + Bevy, focused on:
- discrete-event simulation
- fleet task allocation
- traffic conflict detection
- interactive tooling/UI for rapid iteration

Designed as a portfolio piece aligned with robotics software, simulation, and game/graphics engineering roles.

## Run
```bash
cargo run
```

## Controls
- `W/A/S/D`: move camera
- `Space` / `Left Shift`: move camera up / down
- Hold `Right Mouse Button` + move mouse: look around
- Click `Restart Simulation` button: reset robots, tasks, and simulation time

## What It Does
- Spawns a ground plane, robots, and task markers
- Allocates the nearest unassigned task to each idle robot (fleet coordination)
- Moves robots through scheduled discrete events (time-ordered event queue)
- Draws robot paths with gizmo lines
- Detects traffic conflicts using robot-robot distance checks
- Highlights collisions by switching robot materials in real time
- Supports full simulation reset through an in-app UI button

## Project Structure
- `src/main.rs`: app wiring and system registration
- `src/model.rs`: shared components/resources/types
- `src/camera.rs`: camera setup and controls
- `src/simulation.rs`: world setup and simulation systems
- `src/ui.rs`: UI setup and restart button behavior
