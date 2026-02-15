use bevy::prelude::*;

use crate::config::SimulationConfig;
use crate::plugins::tui_state::SimControl;

/// 現在の Tick 番号を保持するリソース
#[derive(Resource, Debug, Default)]
pub struct CurrentTick {
    pub value: u64,
}

/// Tick 管理プラグイン
pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentTick>()
            .add_systems(
                Update,
                tick_advance_system
                    .in_set(crate::plugins::simulation::SimulationSet)
                    .run_if(crate::plugins::tui_state::simulation_running),
            );
    }
}

/// Tick をインクリメントし、上限に達したら AppExit を送信する
///
/// SimControl が存在する場合（TUI モード）はポーズ制御に従う
pub fn tick_advance_system(
    mut tick: ResMut<CurrentTick>,
    config: Res<SimulationConfig>,
    mut exit: EventWriter<AppExit>,
    sim_control: Option<ResMut<SimControl>>,
) {
    tick.value += 1;

    if tick.value >= config.max_ticks {
        // TUI モードでは強制終了せずに「停止」状態にする
        if config.tui_mode {
            if let Some(mut control) = sim_control {
                control.stopped = true;
                control.paused = true;
            }
        } else {
            exit.send(AppExit::Success);
        }
    }
}
