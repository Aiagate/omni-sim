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
                    let val = args[i + 1].as_str();
                    if val == "-1" {
                        config.max_ticks = u64::MAX;
                    } else {
                        config.max_ticks = val.parse().unwrap_or(config.max_ticks);
                    }
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
            "--tps" => {
                if i + 1 < args.len() {
                    config.max_tps = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let tui_mode = config.tui_mode;
    let max_tps = config.max_tps;

    // インターバルの計算
    // 1. 指定がある場合はそれを使用 (1000ms / TPS)
    // 2. TUI モードで指定がない場合は 60 TPS (16ms)
    // 3. CLI モードで指定がない場合は 0ms (フルスピード)
    let interval = match max_tps {
        Some(tps) if tps > 0 => Duration::from_millis(1000 / tps),
        Some(_) => Duration::from_millis(0), // TPS 0 は制限なし扱い
        None => {
            if tui_mode {
                Duration::from_millis(16)
            } else {
                Duration::from_millis(0)
            }
        }
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
    
    // Web Integration (Always active for now, or could be flagged)
    app.add_plugins(plugins::web_integration::WebIntegrationPlugin);

    // 出力プラグイン: TUI or CLI
    if tui_mode {
        app.add_plugins(TuiOutputPlugin);
    } else {
        app.add_plugins(CliOutputPlugin);
    }

    app.run();
}
