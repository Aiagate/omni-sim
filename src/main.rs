mod config;
mod tick;
mod components;
mod systems;
mod plugins;
mod world_snapshot;

use bevy::prelude::*;
use bevy::app::ScheduleRunnerPlugin;
use std::time::Duration;

use config::SimulationConfig;
use tick::TickPlugin;
use plugins::simulation::SimulationPlugin;
use plugins::world_init::WorldInitPlugin;
use plugins::cli_output::CliOutputPlugin;
use plugins::tui_output::TuiOutputPlugin;

fn main() {
    // コマンドライン引数のパース
    let args: Vec<String> = std::env::args().collect();
    let mut config = SimulationConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--max-ticks" => {
                if i + 1 < args.len() {
                    config.max_ticks = args[i + 1].parse().unwrap_or(config.max_ticks);
                    i += 1;
                }
            }
            "--seed" => {
                if i + 1 < args.len() {
                    config.seed = args[i + 1].parse().unwrap_or(config.seed);
                    i += 1;
                }
            }
            "--tui" => {
                config.tui_mode = true;
            }
            _ => {}
        }
        i += 1;
    }

    let tui_mode = config.tui_mode;

    // TUI モードではインターバルを 16ms (≈60fps)、CLI モードでは 0ms（最高速）
    let interval = if tui_mode {
        Duration::from_millis(16)
    } else {
        Duration::from_millis(0)
    };

    let mut app = App::new();

    // ヘッドレスモード: MinimalPlugins + ScheduleRunner
    app.add_plugins(MinimalPlugins.set(
        ScheduleRunnerPlugin::run_loop(interval),
    ));

    // TUI モードではログプラグインを無効化（stdout 競合回避）
    if !tui_mode {
        app.add_plugins(bevy::log::LogPlugin {
            level: bevy::log::Level::INFO,
            ..default()
        });
    }

    // シミュレーション設定
    app.insert_resource(config);

    // Tick 管理
    app.add_plugins(TickPlugin);

    // 初期世界生成
    app.add_plugins(WorldInitPlugin);

    // シミュレーションロジック
    app.add_plugins(SimulationPlugin);

    // 出力プラグイン: TUI or CLI
    if tui_mode {
        app.add_plugins(TuiOutputPlugin);
    } else {
        app.add_plugins(CliOutputPlugin);
    }

    app.run();
}
