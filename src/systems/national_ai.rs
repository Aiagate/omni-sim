use bevy::prelude::*;
use crate::components::common::SimName;
use crate::components::diplomacy::AtWar;
use crate::components::military::{MilitaryStrength, Fleet};
use crate::tick::CurrentTick;

/// 国家戦略システム
///
/// 戦争中の国家が艦隊を編成し、敵星系へ派遣するロジック
pub fn national_strategy_system(
    mut commands: Commands,
    _tick: Res<CurrentTick>,
    war_nations: Query<(Entity, &AtWar)>,
    mut planets: Query<(
        Entity,
        &crate::components::common::BelongsToNation,
        &crate::components::common::BelongsToStarSystem,
        &mut MilitaryStrength
    ), With<crate::components::common::Planet>>,
    fleets: Query<(
        Entity,
        &crate::components::common::BelongsToNation,
        &crate::components::common::BelongsToStarSystem
    ), With<Fleet>>,
) {
    // 各戦争国家について
    for (nation_entity, war) in &war_nations {
        // 敵の星系を取得 (AtWar に enemy_planet があるので、そこから星系を特定したいが、
        // AtWar には Entity しかない。Enemy Planet がどの System かを知る必要がある。
        // しかし、system-based warfare に移行したので、AtWar に enemy_system を持たせるべきだったかもしれない。
        // 現状は enemy_planet があるので、それを query で引いて system を特定する。)
        
        let target_system = if let Ok((_, _, sys, _)) = planets.get(war.enemy_planet) {
            sys.0
        } else {
            continue; // 敵惑星が見つからない（消滅？）
        };

        // すでにその星系に自軍艦隊がいるかチェック
        let has_fleet_there = fleets.iter().any(|(_, f_nation, f_sys)| {
            f_nation.0 == nation_entity && f_sys.0 == target_system
        });

        if has_fleet_there {
            continue; // 既に展開済み
        }

        // 艦隊を編成できる惑星を探す (自国の惑星)
        // 最も艦船が多い惑星を選ぶ
        let mut best_planet = None;
        let mut max_ships = 0;

        for (p_entity, p_nation, p_sys, mil) in &planets {
            if p_nation.0 == nation_entity && mil.ships > 20 { // 最低20隻は残す
                if mil.ships > max_ships {
                    max_ships = mil.ships;
                    best_planet = Some((p_entity, p_sys.0, mil.ships));
                }
            }
        }

        if let Some((p_entity, _p_sys, ships)) = best_planet {
            // 派遣する艦船数 (半分を派遣)
            let send_ships = ships / 2;
            
            // 惑星から艦船を減らす
            if let Ok((_, _, _, mut mil)) = planets.get_mut(p_entity) {
                mil.ships -= send_ships;
                mil.recalculate();
            }

            // 艦隊を生成 (ターゲット星系に配置)
            // Note: 本来は移動時間が必要だが、今回は簡略化のため即時配置、
            // あるいは p_sys にスポーンして移動させるべきだが、
            // Simplified Plan では「敵星系で戦闘」なので、敵星系に Fleet を置く。
            // (ワープ航法で直接出現したという設定)
            
            commands.spawn((
                SimName(format!("Fleet of Nation {:?}", nation_entity)),
                Fleet,
                MilitaryStrength::new(send_ships),
                crate::components::common::BelongsToNation(nation_entity),
                crate::components::common::BelongsToStarSystem(target_system), // 敵星系に配置
                crate::components::space::Position::new(0.0, 0.0, 0.0), // ダミー座標
            ));
            
            // ログなどは combat_system で出るのでここでは省略
        }
    }
}
