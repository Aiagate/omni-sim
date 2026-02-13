use bevy::prelude::*;
use crate::components::simulation_event::SimulationEvent;
use crate::components::nation_index::NationPlanetIndex;
use crate::systems;

/// シミュレーションシステム用セット
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct SimulationSet;

/// シミュレーション全体のプラグイン
/// 全ドメインのシステムを FixedUpdate に登録する
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        // SimulationEvent を Bevy Event として登録
        app.add_event::<SimulationEvent>();

        // EventLog リソースを初期化
        app.init_resource::<crate::components::events::EventLog>()
            .init_resource::<NationPlanetIndex>();

        app.add_systems(
            FixedUpdate,
            (
                // 1. 資源産出・人口成長 (独立して実行可能)
                (
                    systems::economy::resource_production_system,
                    systems::population::population_growth_system,
                ),
                // 2. 環境、貿易、軍事 (1 の結果に依存する場合があるため、1 の後に実行)
                (
                    systems::economy::environment_dynamics_system,
                    systems::trade::trade_system,
                    systems::military::military_system,
                ).after(systems::economy::resource_production_system),
                // 3. 外交、戦争、研究、テラフォーミング (2 または 1 の後に実行)
                (
                    systems::diplomacy::diplomacy_system.after(systems::military::military_system),
                    systems::war::war_trigger_system.after(systems::diplomacy::diplomacy_system),
                    systems::war::combat_resolution_system.after(systems::war::war_trigger_system),
                    systems::research::research_system.after(systems::economy::resource_production_system),
                    systems::terraforming::terraforming_system.after(systems::research::research_system),
                ),
                // 4. イベント処理・インデックス更新 (最後に実行)
                (
                    systems::events::event_system,
                    systems::index_update::update_nation_planet_index_system,
                ).after(systems::terraforming::terraforming_system),
            )
                .in_set(SimulationSet)
                .run_if(crate::plugins::tui_state::simulation_running),
        );
    }
}
