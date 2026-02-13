use bevy::prelude::*;

use crate::components::common::{SimName, BelongsToNation};
use crate::components::economy::{Resources, ResourceType, DepletableResources};
use crate::components::environment::{PlanetaryEnvironment, RenewableResources, EnvironmentalHealth};
use crate::components::population::Population;
use crate::components::military::MilitaryStrength;
use crate::components::diplomacy::{DiplomaticRelation, AtWar};
use crate::components::technology::{TechnologyState, TechField};
use crate::components::events::{EventKind, EventEffect, EventLog};
use crate::components::simulation_event::SimulationEvent;
use crate::tick::CurrentTick;
use crate::config::SimulationConfig;

/// CLI 出力プラグイン
pub struct CliOutputPlugin;

impl Plugin for CliOutputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, cli_banner_system);
        app.add_systems(
            FixedUpdate,
            (
                cli_event_display_system.after(crate::systems::events::event_system),
                cli_summary_system.after(crate::tick::tick_advance_system),
            ),
        );
    }
}

// ============================================================
// 表示ヘルパー関数（絵文字・名前・フォーマットは全てここで管理）
// ============================================================

use crate::plugins::tui_state;

// ヘルパー関数を tui_state から再エクスポートまたは直接利用
fn event_kind_emoji(kind: &EventKind) -> &'static str { tui_state::event_kind_emoji(kind) }
fn event_kind_name(kind: &EventKind) -> &'static str { tui_state::event_kind_name(kind) }
fn resource_type_name(rt: &ResourceType) -> &'static str { tui_state::resource_type_name(rt) }
fn tech_field_name(field: &TechField) -> &'static str { tui_state::tech_field_name(field) }
fn format_event_effect(effect: &EventEffect) -> String { tui_state::format_event_effect(effect) }

// ============================================================
// Startup: バナー表示
// ============================================================

fn cli_banner_system() {
    println!();
    println!("  ╔═══════════════════════════════════════╗");
    println!("  ║      Deep Juno Simulation             ║");
    println!("  ║   Tick-Based Civilization Simulator    ║");
    println!("  ╚═══════════════════════════════════════╝");
    println!();
}

// ============================================================
// FixedUpdate: Tick ごとのイベント表示
// ============================================================

/// SimulationEvent を受信してリアルタイムの CLI ログを出力するシステム
///
/// 表示頻度は表示側で制御する（ドメインは毎 Tick イベントを送出）
fn cli_event_display_system(
    mut sim_events: EventReader<SimulationEvent>,
    config: Res<SimulationConfig>,
) {
    for event in sim_events.read() {
        // 重要度によるフィルタリング
        if event.importance() < config.min_importance {
            continue;
        }

        match event {
            // === 初期化 ===
            SimulationEvent::WorldInitialized { max_ticks, seed } => {
                info!("=== Deep Juno シミュレーション開始 ===");
                info!("  最大 Tick 数: {}", max_ticks);
                info!("  乱数シード: {}", seed);
                info!("================================");
            }

            // === 経済（10 Tick 毎） ===
            SimulationEvent::ResourceReport { tick: t, planet, food, minerals, energy, goods, labor_bonus, tech_level } => {
                if t % 10 == 0 {
                    let tech_info = tech_level
                        .map(|lv| format!(" Tech:Lv{}", lv))
                        .unwrap_or_default();
                    info!(
                        "[Tick {:>4}] {} - 資源: 食料={:.1}, 鉱物={:.1}, \
                        エネルギー={:.1}, 工業品={:.1} (労働力x{:.2}{})",
                        t, planet, food, minerals, energy, goods, labor_bonus, tech_info
                    );
                    // 鉱物埋蔵量などの詳細を経済システムから送るように修正したいが、
                    // 現状は SimulationEvent を変えずに済ませるため、概要システムで表示する。
                    // ただし、ここで食料やエネルギーの「不足」があれば警告は出せる。
                    if *food < 10.0 { info!("  [!] {} の食料蓄積が危機的です", planet); }
                    if *energy < 10.0 { info!("  [!] {} のエネルギー供給が不安定です", planet); }
                }
            }

            // === 人口（10 Tick 毎） ===
            SimulationEvent::PopulationReport { tick: t, planet, count, change, growth_rate, starvation_ticks, habitability, capacity } => {
                if t % 10 == 0 {
                    let status = if *starvation_ticks > 0 {
                        format!(" [!] 飢餓:{}Tick", starvation_ticks)
                    } else {
                        String::new()
                    };
                    info!(
                        "[Tick {:>4}] {} - 人口: {:.0} ({:+.0}, 成長率: {:+.4}, 居住性: {:.2}, 上限: {:.0}){}",
                        t, planet, count, change, growth_rate, habitability, capacity, status
                    );
                }
            }


            SimulationEvent::TradeSummary { tick: t, from, to, total_amount, resources, .. } => {
                if t % 10 == 0 {
                    let items: Vec<String> = resources.iter()
                        .map(|(rt, amt)| format!("{}:{:.1}", resource_type_name(rt), amt))
                        .collect();
                    info!(
                        "[Tick {:>4}] 貿易: {} -> {} | 計 {:.1} ({})",
                        t, from, to, total_amount, items.join(", ")
                    );
                }
            }

            // === 軍事（10 Tick 毎） ===
            SimulationEvent::MilitaryReport { tick: t, planet, ships, power } => {
                if t % 10 == 0 {
                    info!(
                        "[Tick {:>4}] 軍事: {} - 艦船: {} (戦力: {:.0})",
                        t, planet, ships, power
                    );
                }
            }

            // === 外交（10 Tick 毎） ===
            SimulationEvent::DiplomacyReport { tick: t, relation, score, trend } => {
                if t % 10 == 0 {
                    info!(
                        "[Tick {:>4}] 外交: {} -> 関係スコア: {:.1} (トレンド: {:+.2})",
                        t, relation, score, trend
                    );
                }
            }

            // === 戦争（常時表示） ===
            SimulationEvent::WarDeclared { tick: t, aggressor, defender, score } => {
                info!(
                    "[Tick {:>4}] WAR 戦争勃発！ {} が {} に宣戦布告！(外交スコア: {:.1})",
                    t, aggressor, defender, score
                );
            }
            SimulationEvent::Ceasefire { tick: t, nation, duration } => {
                info!(
                    "[Tick {:>4}] PEACE 停戦: {} -- {} Tick の戦争が終結",
                    t, nation, duration
                );
            }
            SimulationEvent::Surrender { tick: t, nation } => {
                info!(
                    "[Tick {:>4}] SURRENDER {}: 艦船全滅 -- 降伏",
                    t, nation
                );
            }
            SimulationEvent::CombatReport { tick: t, attacker, target, ships_destroyed, casualties, remaining_ships } => {
                if t % 5 == 0 {
                    info!(
                        "[Tick {:>4}] COMBAT {} -> {} 攻撃: 艦船-{} 撃破, 人口-{:.0} 犠牲, {} 残艦:{}",
                        t, attacker, target, ships_destroyed, casualties, target, remaining_ships
                    );
                }
            }

            // === 研究（レベルアップは常時、レポートは 25 Tick 毎） ===
            SimulationEvent::TechLevelUp { tick: t, planet, total_level, next_field, rate } => {
                info!(
                    "[Tick {:>4}] TECH {} -- 技術レベルアップ！ (総合Lv: {}, 次の研究: {}, 研究レート: {:.1}/Tick)",
                    t, planet, total_level, tech_field_name(next_field), rate
                );
            }
            SimulationEvent::ResearchReport { tick: t, planet, levels, rate, progress, cost } => {
                if t % 25 == 0 {
                    info!(
                        "[Tick {:>4}] TECH {} -- 技術: 農Lv{} 鉱Lv{} ｴﾈLv{} 工Lv{} 軍Lv{} 航Lv{} 環Lv{} 核Lv{} (レート: {:.1}/Tick, 進捗: {:.0}/{:.0})",
                        t, planet,
                        levels[0], levels[1], levels[2], levels[3], levels[4], levels[5], levels[6], levels[7],
                        rate, progress, cost
                    );
                }
            }

            // === ランダムイベント（常時表示） ===
            SimulationEvent::RandomEvent { tick: t, planet, kind, intensity, effect } => {
                info!(
                    "[Tick {:>4}] {} {} -- {} (強度: {:.1}x): {}",
                    t, event_kind_emoji(kind), planet, event_kind_name(kind), intensity, format_event_effect(effect)
                );
            }

            // === テラフォーミング ===
            SimulationEvent::TerraformingProgress { tick: t, planet, phase, progress } => {
                if t % 10 == 0 {
                    info!(
                        "[Tick {:>4}] TERRAFORMING {} -- {}: {:.1}%",
                        t, planet, phase, progress * 100.0
                    );
                }
            }
            SimulationEvent::TerraformingPhaseComplete { tick: t, planet, phase } => {
                info!(
                    "[Tick {:>4}] TERRAFORMING {} -- {} フェーズ完了！居住性が向上しました。",
                    t, planet, phase
                );
            }
        }
    }
}

// ============================================================
// FixedUpdate: シミュレーション終了時のサマリー
// ============================================================

/// シミュレーション終了時にサマリーテーブルを出力する
fn cli_summary_system(
    tick: Res<CurrentTick>,
    config: Res<SimulationConfig>,
    planets: Query<(
        &SimName,
        &Population,
        &Resources,
        Option<&MilitaryStrength>,
        Option<&BelongsToNation>,
        &PlanetaryEnvironment,
        &DepletableResources,
        &RenewableResources,
        &EnvironmentalHealth,
    )>,
    nations: Query<&TechnologyState>,
    relations: Query<(&SimName, &DiplomaticRelation)>,
    wars: Query<(&SimName, &AtWar)>,
    event_log: Res<EventLog>,
) {
    if tick.value >= config.max_ticks {
        // === 惑星サマリー ===
        println!();
        println!("+============================================================================================================================+");
        println!("|                                       Deep Juno - シミュレーション結果                                                     |");
        println!("+================+============+===========+=========+=========+========+========+=========+=========+===========+============+===========+");
        println!("| 惑星           | 人口/上限  | 成長率    | 居住性  | 土壌/汚染 | 食料   | 鉱物   | ｴﾈﾙｷﾞｰ  | 工業品  | 鉱物埋蔵量 | 戦力       | 技術 Lv   |");
        println!("+================+============+===========+=========+=========+========+========+=========+=========+===========+============+===========+");

        for (name, pop, res, mil, belongs_to, env, deplet, renew, health) in &planets {
            let starvation = if pop.starvation_ticks > 0 {
                format!(" [!]{}", pop.starvation_ticks)
            } else {
                String::new()
            };
            let (_combat_flag, mil_str) = if let Some(m) = mil {
                let combat = m.in_combat;
                let marker = if combat { "*" } else { " " };
                (combat, format!("{}{:>3}/{:.0}", marker, m.ships, m.power))
            } else {
                (false, "  -/-".to_string())
            };
            
            // 技術ステートを国家から取得
            let tech = belongs_to.and_then(|b| nations.get(b.0).ok());
            let tech_str = if let Some(t) = tech {
                format!("{:>2}", t.total_level())
            } else {
                "-".to_string()
            };
            
            let env_info = format!("{:.1}/{:.1}", renew.soil_fertility, health.air_pollution + health.water_pollution);
            let mineral_info = format!("{:>4.0}%", (deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0)) * 100.0);

            println!(
                "| {:14} | {:>4.0}/{:>4.0}M | {:>+8.4}% | {:>7.2} | {:>7} | {:>6.0} | {:>6.0} | {:>7.0} | {:>7.0} | {:>10} | {:10} | {:>2} |{}",
                name.0, pop.count / 1_000_000.0, env.population_capacity / 1_000_000.0, pop.effective_growth_rate * 100.0,
                env.habitability, env_info, res.food, res.minerals, res.energy, res.manufactured_goods,
                mineral_info, mil_str, tech_str, starvation
            );
        }

        // === 外交サマリー ===
        println!();
        println!("+============================================+");
        println!("|         外交関係                           |");
        println!("+======================+=========+===========+");
        println!("| 関係                 | スコア  | 状態      |");
        println!("+======================+=========+===========+");

        for (name, rel) in &relations {
            let status = if rel.score > 50.0 { "同盟" }
                else if rel.score > 0.0 { "友好" }
                else if rel.score > -30.0 { "緊張" }
                else if rel.score > -50.0 { "敵対" }
                else { "戦争中" };
            println!("| {:20} | {:>+6.1} | {:10} |", name.0, rel.score, status);
        }
        println!("+======================+=========+===========+");

        // === 進行中の戦争 ===
        let active_wars: Vec<_> = wars.iter().collect();
        if !active_wars.is_empty() {
            println!();
            println!("+=====================================================+");
            println!("|         進行中の戦争                                |");
            println!("+=================+===========+==========+=============+");
            println!("| 国家            | 開始 Tick | 撃破艦船 | 民間犠牲    |");
            println!("+=================+===========+==========+=============+");
            for (name, war) in &active_wars {
                println!("| {:15} | {:>9} | {:>8} | {:>11.0} |",
                    name.0, war.started_at, war.total_ships_lost, war.total_casualties);
            }
            println!("+=================+===========+==========+=============+");
        }

        // === イベント履歴（最新10件） ===
        if !event_log.events.is_empty() {
            println!();
            println!("+================================================================+");
            println!("|         イベント履歴（最新10件）                                |");
            println!("+======+=================+========================================+");
            println!("| Tick | 惑星            | イベント                               |");
            println!("+======+=================+========================================+");

            let start = if event_log.events.len() > 10 { event_log.events.len() - 10 } else { 0 };
            for record in &event_log.events[start..] {
                println!("| {:>4} | {:15} | {} {} -- {}",
                    record.tick, record.planet_name,
                    event_kind_emoji(&record.kind), event_kind_name(&record.kind),
                    format_event_effect(&record.effect)
                );
            }
            println!("+======+=================+========================================+");
            println!("  合計 {} 件のイベントが発生", event_log.events.len());
        }

        println!();
    }
}
