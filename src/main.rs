mod components;
mod systems;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use std::time::Duration;
use systems::{report, setup, transport_execution};

fn main() {
    App::new()
        .add_plugins(
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            ))),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, (transport_execution, report))
        .run();
}
