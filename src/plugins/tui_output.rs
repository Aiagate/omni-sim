use std::io::{self, Stdout};
use std::time::Duration;

use bevy::prelude::*;
use crossterm::{
    event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Cell, Chart, Dataset, GraphType, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};

use crate::components::diplomacy::{AtWar, DiplomaticRelation};
use crate::components::economy::Resources;
use crate::components::military::MilitaryStrength;
use crate::components::population::Population;
use crate::components::simulation_event::SimulationEvent;
use crate::components::technology::TechnologyState;
use crate::config::SimulationConfig;
use crate::tick::CurrentTick;

use super::tui_state::*;
use super::simulation::SimulationSet;

// ============================================================
// Terminal リソース
// ============================================================

/// Ratatui Terminal を Bevy Resource として保持する
#[derive(Resource)]
pub struct TerminalResource {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

// ============================================================
// TUI プラグイン
// ============================================================

/// Ratatui ベースの TUI 出力プラグイン
pub struct TuiOutputPlugin;

impl Plugin for TuiOutputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TuiAppState>()
            .init_resource::<SimControl>()
            .add_systems(Startup, tui_setup_system)
            .add_systems(
                FixedUpdate,
                (
                    tui_input_system
                        .before(SimulationSet),
                    tui_event_collector_system
                        .after(crate::systems::events::event_system),
                    tui_snapshot_system
                        .after(tui_event_collector_system),
                    tui_tps_system
                        .after(tui_snapshot_system),
                    tui_render_system
                        .after(tui_tps_system)
                        .after(crate::tick::tick_advance_system),
                    tui_step_reset_system
                        .after(tui_render_system)
                        .after(SimulationSet),
                    tui_quit_system
                        .after(tui_step_reset_system),
                ),
            );
    }
}

// ============================================================
// Startup: Terminal 初期化
// ============================================================

fn tui_setup_system(mut commands: Commands) {
    enable_raw_mode().expect("failed to enable raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).expect("failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend).expect("failed to create terminal");
    commands.insert_resource(TerminalResource { terminal });
}

// ============================================================
// FixedUpdate: キーボード入力処理
// ============================================================

fn tui_input_system(mut sim_control: ResMut<SimControl>, mut state: ResMut<TuiAppState>) {
    // ノンブロッキングでキーイベントをポーリング
    while let Ok(true) = event::poll(Duration::from_millis(0)) {
        if let Ok(CrosstermEvent::Key(key)) = event::read() {
            match key.code {
                // 終了
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    sim_control.quit_requested = true;
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    sim_control.quit_requested = true;
                }

                // 一時停止 / 再開 (Toggle)
                KeyCode::Char(' ') | KeyCode::Char('s') | KeyCode::Char('S') => {
                    if sim_control.paused || sim_control.stopped {
                        sim_control.paused = false;
                        sim_control.stopped = false;
                    } else {
                        sim_control.paused = true;
                        sim_control.stopped = true;
                    }
                }

                // ステップ実行（ポーズ中のみ）
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    if (sim_control.paused || sim_control.stopped) && state.current_tick < state.max_ticks {
                        sim_control.step_once = true;
                    }
                }

                // 速度変更
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    sim_control.speed = sim_control.speed.faster();
                }
                KeyCode::Char('-') => {
                    sim_control.speed = sim_control.speed.slower();
                }

                // ビュー切り替え: 詳細表示
                KeyCode::Enter | KeyCode::Char('i') | KeyCode::Char('I') => {
                    if state.view == TuiView::Main && !state.planets.is_empty() {
                        state.view = TuiView::PlanetDetail;
                    }
                }
                // ビュー切り替え: 戻る
                KeyCode::Esc | KeyCode::Backspace => {
                    if state.view == TuiView::PlanetDetail {
                        state.view = TuiView::Main;
                    }
                }

                // 惑星選択
                KeyCode::Up => {
                    if state.selected_planet > 0 {
                        state.selected_planet -= 1;
                    }
                }
                KeyCode::Down => {
                    let max = if state.planets.is_empty() {
                        0
                    } else {
                        state.planets.len() - 1
                    };
                    if state.selected_planet < max {
                        state.selected_planet += 1;
                    }
                }

                // ログスクロール
                KeyCode::Char('j') | KeyCode::Char('J') => {
                    if state.log_scroll_offset + 1 < state.event_log.len() {
                        state.log_scroll_offset += 1;
                    }
                }
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    if state.log_scroll_offset > 0 {
                        state.log_scroll_offset -= 1;
                    }
                }

                _ => {}
            }
        }
    }
}

// ============================================================
// FixedUpdate: SimulationEvent → TuiAppState への変換
// ============================================================

fn tui_event_collector_system(
    mut sim_events: EventReader<SimulationEvent>,
    mut state: ResMut<TuiAppState>,
    config: Res<SimulationConfig>,
) {
    for event in sim_events.read() {
        // 重要度によるフィルタリング
        if event.importance() < config.min_importance {
            continue;
        }

        match event {
            SimulationEvent::WorldInitialized { max_ticks, seed } => {
                state.max_ticks = *max_ticks;
                state.seed = *seed;
                state.push_log(
                    0,
                    format!("シミュレーション開始 (Seed: {}, Max: {} Tick)", seed, max_ticks),
                    EventSeverity::Info,
                );
            }

            SimulationEvent::ResourceReport { .. } => {
                // スナップショットで処理
            }

            SimulationEvent::PopulationReport { tick: t, planet, starvation_ticks, .. } => {
                if *starvation_ticks > 0 {
                    state.push_log(
                        *t,
                        format!("⚠ {} 飢餓進行中 ({} Tick)", planet, starvation_ticks),
                        EventSeverity::Warning,
                    );
                }
            }


            SimulationEvent::TradeSummary { tick: t, from, to, total_amount, resources, .. } => {
                // Medium レベルだがログの流量を抑えるため、一定間隔または大量輸送時のみ表示
                if *t % 10 == 0 || *total_amount > 50.0 {
                    let items: Vec<String> = resources.iter()
                        .map(|(rt, amt)| format!("{}:{:.0}", resource_type_short_name(rt), amt))
                        .collect();
                    
                    state.push_log(
                        *t,
                        format!(
                            "📦 {} → {} | 計 {:.1} ({})",
                            from, to, total_amount, items.join(", ")
                        ),
                        EventSeverity::Info,
                    );
                }
            }

            SimulationEvent::MilitaryReport { .. } => {
                // スナップショットで処理
            }

            SimulationEvent::DiplomacyReport { tick: t, relation, score, trend } => {
                if t % 10 == 0 {
                    state.push_log(
                        *t,
                        format!("🤝 {} スコア: {:.1} ({:+.2})", relation, score, trend),
                        EventSeverity::Info,
                    );
                }
            }

            SimulationEvent::WarDeclared { tick: t, aggressor, defender, score } => {
                state.push_log(
                    *t,
                    format!("⚔ 戦争勃発！ {} → {} (外交: {:.1})", aggressor, defender, score),
                    EventSeverity::Critical,
                );
            }

            SimulationEvent::Ceasefire { tick: t, nation, duration } => {
                state.push_log(
                    *t,
                    format!("🕊 停戦: {} ({} Tick の戦争が終結)", nation, duration),
                    EventSeverity::Warning,
                );
            }

            SimulationEvent::Surrender { tick: t, nation } => {
                state.push_log(
                    *t,
                    format!("🏳 降伏: {} 艦船全滅", nation),
                    EventSeverity::Critical,
                );
            }

            SimulationEvent::ResearchReport { tick: t, planet, levels, rate, progress: _, cost: _ } => {
                if t % 5 == 0 {
                    state.push_log(
                        *t,
                        format!(
                            "TECH {}: 農{} 鉱{} ｴﾈ{} 工{} 軍{} 航{} 環{} 核{} ({:.1}/T)",
                            planet, levels[0], levels[1], levels[2], levels[3], levels[4], levels[5], levels[6], levels[7], rate
                        ),
                        EventSeverity::Info,
                    );
                }
            }

            SimulationEvent::TechLevelUp { tick: t, planet, total_level, next_field, .. } => {
                state.push_log(
                    *t,
                    format!(
                        "💡 {} 技術Lv UP! (総合Lv{}, 次: {})",
                        planet, total_level, tech_field_name(next_field)
                    ),
                    EventSeverity::Info,
                );
            }

            SimulationEvent::CombatReport { tick: t, attacker, target, ships_destroyed, casualties, remaining_ships } => {
                if t % 5 == 0 {
                    state.push_log(
                        *t,
                        format!(
                            "💥 {} → {} 攻撃: -{} 艦, -{:.0} 人, 残 {} 艦",
                            attacker, target, ships_destroyed, casualties, remaining_ships
                        ),
                        EventSeverity::Warning,
                    );
                }
            }

            SimulationEvent::RandomEvent { tick: t, planet, kind, intensity, effect } => {
                let severity = match kind {
                    crate::components::events::EventKind::Plague
                    | crate::components::events::EventKind::Rebellion
                    | crate::components::events::EventKind::EnergyCrisis => EventSeverity::Critical,
                    _ => EventSeverity::Warning,
                };
                state.push_log(
                    *t,
                    format!(
                        "{} {} {} ({:.1}x): {}",
                        event_kind_emoji(kind),
                        planet,
                        event_kind_name(kind),
                        intensity,
                        format_event_effect(effect)
                    ),
                    severity,
                );
            }

            SimulationEvent::TerraformingProgress { tick: t, planet, phase, progress } => {
                if t % 10 == 0 {
                    state.push_log(
                        *t,
                        format!("🏗 Terraforming {}: {} ({:.1}%)", planet, phase, progress * 100.0),
                        EventSeverity::Info,
                    );
                }
            }
            SimulationEvent::TerraformingPhaseComplete { tick: t, planet, phase } => {
                state.push_log(
                    *t,
                    format!("✨ {} {} Completion!", planet, phase),
                    EventSeverity::Info,
                );
            }
        }
    }
}

// ============================================================
// FixedUpdate: ECS → スナップショット同期
// ============================================================

fn tui_snapshot_system(
    tick: Res<CurrentTick>,
    config: Res<SimulationConfig>,
    sim_control: Res<SimControl>,
    mut state: ResMut<TuiAppState>,
    planets: Query<(
        &crate::components::common::SimName,
        &Population,
        &Resources,
        Option<&MilitaryStrength>,
        Option<&crate::components::common::BelongsToNation>,
        &crate::components::environment::PlanetaryEnvironment,
        &crate::components::economy::DepletableResources,
        &crate::components::environment::RenewableResources,
        &crate::components::environment::EnvironmentalHealth,
    )>,
    nations: Query<&TechnologyState>,
    relations: Query<(&crate::components::common::SimName, &DiplomaticRelation)>,
    wars: Query<(&crate::components::common::SimName, &AtWar)>,
) {
    state.current_tick = tick.value;
    state.max_ticks = config.max_ticks;

    // 惑星スナップショットを更新
    state.planets.clear();
    for (name, pop, res, mil, belongs_to, env, deplet, renew, health) in &planets {
        let tech = belongs_to.and_then(|b| nations.get(b.0).ok());
        state.planets.push(PlanetSnapshot {
            name: name.0.clone(),
            population: pop.count,
            growth_rate: pop.effective_growth_rate,
            food: res.food,
            minerals: res.minerals,
            energy: res.energy,
            manufactured_goods: res.manufactured_goods,
            ships: mil.map_or(0, |m: &MilitaryStrength| m.ships),
            power: mil.map_or(0.0, |m| m.power),
            in_combat: mil.map_or(false, |m| m.in_combat),
            tech_total: tech.map_or(0, |t: &TechnologyState| t.total_level()),
            tech_levels: tech.map_or([0; 8], |t: &TechnologyState| {
                [
                    t.agriculture_level,
                    t.mining_level,
                    t.energy_level,
                    t.manufacturing_level,
                    t.military_level,
                    t.navigation_level,
                    t.environmental_level,
                    t.nuclear_fusion_level,
                ]
            }),
            starvation_ticks: pop.starvation_ticks,
            habitability: env.habitability,
            population_capacity: env.population_capacity,
            mineral_reserves_pct: (deplet.mineral_reserves / deplet.initial_mineral_reserves.max(1.0)),
            pollution: health.air_pollution + health.water_pollution,
            soil_fertility: renew.soil_fertility,
        });
    }

    // 外交スナップショットを更新
    state.diplomacy.clear();
    for (name, rel) in &relations {
        let status = if rel.score > 50.0 {
            "同盟"
        } else if rel.score > 0.0 {
            "友好"
        } else if rel.score > -30.0 {
            "緊張"
        } else if rel.score > -50.0 {
            "敵対"
        } else {
            "戦争中"
        };
        state.diplomacy.push(DiplomacyEntry {
            relation_name: name.0.clone(),
            score: rel.score,
            status: status.to_string(),
        });
    }

    // 戦争スナップショットを更新
    state.wars.clear();
    for (name, war) in &wars {
        state.wars.push(WarEntry {
            nation: name.0.clone(),
            started_at: war.started_at,
            ships_lost: war.total_ships_lost,
            casualties: war.total_casualties,
        });
    }

    // 歴史データを更新 (進行中のみ)
    if !sim_control.paused && !sim_control.stopped || sim_control.step_once {
        let total_pop: f64 = state.planets.iter().map(|p| p.population).sum();
        let total_goods: f64 = state.planets.iter().map(|p| p.manufactured_goods).sum();

        state.population_history.push(total_pop as u64);
        state.resource_history.push(total_goods as u64);

        if state.population_history.len() > 100 {
            state.population_history.remove(0);
        }
        if state.resource_history.len() > 100 {
            state.resource_history.remove(0);
        }
    }
}

// ============================================================
// FixedUpdate: TPS 計算
// ============================================================

fn tui_tps_system(
    time: Res<Time>,
    mut state: ResMut<TuiAppState>,
    mut last_tick: Local<u64>,
    mut last_time: Local<f64>,
) {
    let now = time.elapsed_secs_f64();
    let delta_time = now - *last_time;

    // 0.5秒ごとに更新
    if delta_time >= 0.5 {
        let delta_tick = state.current_tick.saturating_sub(*last_tick);
        state.tps = delta_tick as f64 / delta_time;
        
        *last_tick = state.current_tick;
        *last_time = now;
    }
}

// ============================================================
// FixedUpdate: ダッシュボード描画
// ============================================================

fn tui_render_system(
    mut term_res: ResMut<TerminalResource>,
    state: Res<TuiAppState>,
    sim_control: Res<SimControl>,
) {
    let _ = term_res.terminal.draw(|frame| {
        render_dashboard(frame, &state, &sim_control);
    });
}

/// ダッシュボード全体の描画
fn render_dashboard(frame: &mut Frame, state: &TuiAppState, control: &SimControl) {
    let area = frame.area();

    // メインレイアウト: ヘッダー(3) + ボディ(残) + フッター(3)
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // ヘッダー
            Constraint::Min(10),   // ボディ
            Constraint::Length(3), // フッター
        ])
        .split(area);

    render_header(frame, main_layout[0], state, control);
    
    match state.view {
        TuiView::Main => render_body(frame, main_layout[1], state),
        TuiView::PlanetDetail => render_planet_detail(frame, main_layout[1], state),
    }

    render_footer(frame, main_layout[2], state, control);
}

/// ヘッダー: タイトル、Tick 進捗、速度、状態
fn render_header(frame: &mut Frame, area: Rect, state: &TuiAppState, control: &SimControl) {
    let status = if control.quit_requested {
        "QUIT"
    } else if control.stopped {
        if state.current_tick >= state.max_ticks { "FINISHED" } else { "STOPPED" }
    } else if control.paused {
        "PAUSED"
    } else {
        "RUNNING"
    };
    let status_color = match status {
        "RUNNING" => Color::Green,
        "PAUSED" => Color::Yellow,
        "FINISHED" => Color::Cyan,
        _ => Color::Red,
    };

    let progress_pct = if state.max_ticks > 0 {
        (state.current_tick as f64 / state.max_ticks as f64 * 100.0).min(100.0)
    } else {
        0.0
    };

    let header_text = format!(
        " Deep Juno   Tick: {}/{} ({:.0}%)  |  TPS: {:.1}  |  Speed: {}  |  Seed: {}",
        state.current_tick,
        state.max_ticks,
        progress_pct,
        state.tps,
        control.speed.label(),
        state.seed,
    );

    let header = Paragraph::new(Line::from(vec![
        Span::styled(header_text, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(
            format!(" {} ", status),
            Style::default()
                .fg(Color::Black)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Deep Juno Simulation ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    );

    frame.render_widget(header, area);
}

/// ボディ: 左（惑星テーブル + 外交）、右（イベントログ）
fn render_body(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // 左パネル
            Constraint::Percentage(40), // 右パネル（ログ）
        ])
        .split(area);

    // 左パネル: 惑星テーブル + 外交 + 統計
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // 惑星テーブル
            Constraint::Percentage(30), // 外交・戦争
            Constraint::Length(7),      // 統計グラフ
        ])
        .split(body_layout[0]);

    render_planet_table(frame, left_layout[0], state);
    render_diplomacy_panel(frame, left_layout[1], state);
    render_statistics_panel(frame, left_layout[2], state);

    // 右パネル: イベントログ
    render_event_log(frame, body_layout[1], state);
}

/// 惑星サマリーテーブル
fn render_planet_table(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    let header_cells = [
        "惑星", "人口/上限", "成長率", "居住性", "土壌/汚染", "鉱物残", "食料", "鉱物", "ｴﾈﾙｷﾞｰ", "工業品", "艦船", "技術",
    ]
    .iter()
    .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = state
        .planets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let combat_marker = if p.in_combat { "*" } else { " " };
            let tech_str = format!(
                "{} ({}/{}/{}/{}/{}/{}/{})",
                p.tech_total,
                p.tech_levels[0],
                p.tech_levels[1],
                p.tech_levels[2],
                p.tech_levels[3],
                p.tech_levels[4],
                p.tech_levels[5],
                p.tech_levels[6]
            );

            let name_style = if i == state.selected_planet {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let pop_color = if p.starvation_ticks > 0 {
                Color::Red
            } else {
                Color::White
            };

            Row::new(vec![
                Cell::from(format!("{}{}", if i == state.selected_planet { "> " } else { "  " }, p.name))
                    .style(name_style),
                Cell::from(format!("{}/{}", format_short_population(p.population), format_short_population(p.population_capacity)))
                    .style(Style::default().fg(pop_color)),
                Cell::from(format!("{:+.2}%", p.growth_rate * 100.0))
                    .style(Style::default().fg(if p.growth_rate >= 0.0 {
                        Color::Green
                    } else {
                        Color::Red
                    })),
                Cell::from(format!("{:.2}", p.habitability)),
                Cell::from(format!("{:.1}/{:.2}", p.soil_fertility, p.pollution)),
                Cell::from(format!("{:.0}%", p.mineral_reserves_pct * 100.0)),
                Cell::from(format!("{:.0}", p.food)),
                Cell::from(format!("{:.0}", p.minerals)),
                Cell::from(format!("{:.0}", p.energy)),
                Cell::from(format!("{:.0}", p.manufactured_goods)),
                Cell::from(format!("{}{}/{:.0}", combat_marker, p.ships, p.power)),
                Cell::from(tech_str),
            ])
            .height(1)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(12),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Planets (Press Enter for detail) ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(table, area);
}

/// 外交・戦争パネル
fn render_diplomacy_panel(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    let diplo_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    // 外交関係
    let diplo_items: Vec<ListItem> = state
        .diplomacy
        .iter()
        .map(|d| {
            let color = if d.score > 50.0 {
                Color::Green
            } else if d.score > 0.0 {
                Color::Cyan
            } else if d.score > -30.0 {
                Color::Yellow
            } else {
                Color::Red
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{}: ", d.relation_name),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:+.1} ", d.score),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("({})", d.status),
                    Style::default().fg(color),
                ),
            ]))
        })
        .collect();

    let diplo_list = List::new(diplo_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Diplomacy ")
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(diplo_list, diplo_layout[0]);

    // 進行中の戦争
    let war_items: Vec<ListItem> = if state.wars.is_empty() {
        vec![ListItem::new(Span::styled(
            "  Peace",
            Style::default().fg(Color::Green),
        ))]
    } else {
        state
            .wars
            .iter()
            .map(|w| {
                ListItem::new(Line::from(vec![
                    Span::styled("* ", Style::default().fg(Color::Red)),
                    Span::styled(
                        format!("{} ", w.nation),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled(
                        format!("(T{}, -{} ships, -{:.0} pop)", w.started_at, w.ships_lost, w.casualties),
                        Style::default().fg(Color::Red),
                    ),
                ]))
            })
            .collect()
    };

    let war_list = List::new(war_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Wars ")
            .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(war_list, diplo_layout[1]);
}

/// 統計パネル（折れ線グラフ表示）
fn render_statistics_panel(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    let stats_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(area);

    // 人口推移
    render_line_chart(
        frame,
        stats_layout[0],
        &state.population_history,
        state.current_tick,
        " Total Population ",
        Color::Yellow,
    );

    // 工業品推移
    render_line_chart(
        frame,
        stats_layout[1],
        &state.resource_history,
        state.current_tick,
        " Total Manufactured Goods ",
        Color::Blue,
    );
}

/// 汎用折れ線グラフ描画ヘルパー
fn render_line_chart(
    frame: &mut Frame,
    area: Rect,
    history: &[u64],
    current_tick: u64,
    title: &str,
    color: Color,
) {
    if history.is_empty() {
        return;
    }

    let data: Vec<(f64, f64)> = history
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = current_tick as f64 - (history.len() as f64 - 1.0 - i as f64);
            (x, v as f64)
        })
        .collect();

    let min_x = data.first().map(|d| d.0).unwrap_or(0.0);
    let max_x = data.last().map(|d| d.0).unwrap_or(0.0);
    let min_y = data.iter().map(|d| d.1).fold(f64::INFINITY, f64::min);
    let max_y = data.iter().map(|d| d.1).fold(f64::NEG_INFINITY, f64::max);

    // Y軸の範囲に少しバッファを持たせる
    let y_range = max_y - min_y;
    let y_padding = if y_range == 0.0 { 1.0 } else { y_range * 0.1 };
    let y_bounds = [
        (min_y - y_padding).max(0.0),
        max_y + y_padding,
    ];

    let dataset = Dataset::default()
        .name(title)
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(color))
        .data(&data);

    let x_axis = Axis::default()
        .title("Tick")
        .style(Style::default().fg(Color::Gray))
        .bounds([min_x, max_x])
        .labels(vec![
            Span::raw(format!("{}", min_x as u64)),
            Span::raw(format!("{}", max_x as u64)),
        ]);

    let y_axis = Axis::default()
        .title("Value")
        .style(Style::default().fg(Color::Gray))
        .bounds(y_bounds)
        .labels(vec![
            Span::raw(format!("{:.0}", y_bounds[0])),
            Span::raw(format!("{:.0}", y_bounds[1])),
        ]);

    let chart = Chart::new(vec![dataset])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD))
                ]))
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    frame.render_widget(chart, area);
}

/// イベントログパネル（スクロール可能）
fn render_event_log(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    let visible_height = area.height.saturating_sub(2) as usize; // ボーダー分を引く

    // スクロール位置の計算（最新が下）
    let total = state.event_log.len();
    let max_scroll = total.saturating_sub(visible_height);
    let scroll_pos = if state.log_scroll_offset == 0 {
        max_scroll // デフォルトは最新
    } else {
        max_scroll.saturating_sub(state.log_scroll_offset)
    };

    let items: Vec<ListItem> = state
        .event_log
        .iter()
        .skip(scroll_pos)
        .take(visible_height)
        .map(|entry| {
            let color = match entry.severity {
                EventSeverity::Info => Color::White,
                EventSeverity::Warning => Color::Yellow,
                EventSeverity::Critical => Color::Red,
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("[{:>4}] ", entry.tick),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(&entry.message, Style::default().fg(color)),
            ]))
        })
        .collect();

    let scroll_indicator = if total > visible_height {
        format!(" Log ({}/{}) ", scroll_pos + visible_height.min(total.saturating_sub(scroll_pos)), total)
    } else {
        format!(" Log ({}) ", total)
    };

    let log_list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(scroll_indicator)
            .title_style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
    );
    frame.render_widget(log_list, area);
}

/// 惑星詳細ビュー
fn render_planet_detail(frame: &mut Frame, area: Rect, state: &TuiAppState) {
    if state.planets.is_empty() || state.selected_planet >= state.planets.len() {
        return;
    }
    let p = &state.planets[state.selected_planet];

    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" Detailed Info: {} ", p.name))
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    
    let inner_area = detail_block.inner(area);
    frame.render_widget(detail_block, area);

    let main_detail_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(inner_area);

    // 左側: 基礎統計
    let left_detail = vec![
        Line::from(vec![Span::styled("Population: ", Style::default().fg(Color::Gray)), Span::raw(format_population(p.population))]),
        Line::from(vec![Span::styled("Growth:     ", Style::default().fg(Color::Gray)), Span::styled(format!("{:+.2}%", p.growth_rate * 100.0), Style::default().fg(if p.growth_rate >= 0.0 { Color::Green } else { Color::Red }))]),
        Line::from(""),
        Line::from(Span::styled("--- Resources ---", Style::default().fg(Color::Yellow))),
        Line::from(vec![Span::styled("Food:       ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}", p.food))]),
        Line::from(vec![Span::styled("Minerals:   ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}", p.minerals))]),
        Line::from(vec![Span::styled("Energy:     ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}", p.energy))]),
        Line::from(vec![Span::styled("Goods:      ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}", p.manufactured_goods))]),
        Line::from(""),
        Line::from(Span::styled("--- Military ---", Style::default().fg(Color::Red))),
        Line::from(vec![Span::styled("Ships:      ", Style::default().fg(Color::Gray)), Span::raw(p.ships.to_string())]),
        Line::from(vec![Span::styled("Power:      ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}", p.power))]),
        Line::from(vec![Span::styled("Status:     ", Style::default().fg(Color::Gray)), Span::styled(if p.in_combat { "IN COMBAT" } else { "Idle" }, Style::default().fg(if p.in_combat { Color::Red } else { Color::Green }))]),
        Line::from(""),
        Line::from(Span::styled("--- Environment & Sustainability ---", Style::default().fg(Color::Green))),
        Line::from(vec![Span::styled("Habitability: ", Style::default().fg(Color::Gray)), Span::raw(format!("{:.2}", p.habitability))]),
        Line::from(vec![Span::styled("Capacity:     ", Style::default().fg(Color::Gray)), Span::raw(format_population(p.population_capacity))]),
        Line::from(vec![Span::styled("Soil Fertility:", Style::default().fg(Color::Gray)), Span::raw(format!("{:.2}", p.soil_fertility))]),
        Line::from(vec![Span::styled("Pollution:    ", Style::default().fg(Color::Gray)), Span::styled(format!("{:.3}", p.pollution), Style::default().fg(if p.pollution > 0.5 { Color::Red } else if p.pollution > 0.1 { Color::Yellow } else { Color::Green }))]),
        Line::from(vec![Span::styled("Minerals (Res):", Style::default().fg(Color::Gray)), Span::raw(format!("{:.1}%", p.mineral_reserves_pct * 100.0))]),
    ];
    frame.render_widget(Paragraph::new(left_detail), main_detail_layout[0]);

    // 右側: 技術詳細
    let tech_fields = ["Agriculture", "Mining", "Energy", "Manufacturing", "Military", "Navigation", "Environmental"];
    let mut tech_detail = vec![
        Line::from(Span::styled("--- Technology Levels ---", Style::default().fg(Color::Blue))),
    ];
    for (i, field) in tech_fields.iter().enumerate() {
        let level = p.tech_levels[i];
        let bonus = (level as f64) * 0.10 * 100.0;
        tech_detail.push(Line::from(vec![
            Span::styled(format!("{:<14}: ", field), Style::default().fg(Color::Gray)),
            Span::styled(format!("Lv{}", level), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw(format!(" (+{:.0}%)", bonus)),
        ]));
    }
    
    tech_detail.push(Line::from(""));
    tech_detail.push(Line::from(vec![
        Span::styled("Total Tech Level: ", Style::default().fg(Color::Cyan)),
        Span::styled(p.tech_total.to_string(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ]));

    frame.render_widget(Paragraph::new(tech_detail), main_detail_layout[1]);
}

/// フッター: キーバインドヘルプ
fn render_footer(frame: &mut Frame, area: Rect, state: &TuiAppState, control: &SimControl) {
    let pause_label = if control.paused || control.stopped { "Resume" } else { "Pause" };
    let mut keys = vec![
        Span::styled(" [Space/S] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(pause_label),
        Span::styled("  [N] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Step"),
        Span::styled("  [+/-] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Speed"),
    ];

    match state.view {
        TuiView::Main => {
            keys.extend_from_slice(&[
                Span::styled("  [Up/Down] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("Select"),
                Span::styled("  [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Detail"),
                Span::styled("  [J/K] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("Scroll"),
            ]);
        }
        TuiView::PlanetDetail => {
            keys.extend_from_slice(&[
                Span::styled("  [Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Back"),
            ]);
        }
    }

    keys.extend_from_slice(&[
        Span::styled("  [Q] ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Quit"),
    ]);

    let footer = Paragraph::new(Line::from(keys))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(footer, area);
}

// ============================================================
// 終了処理
// ============================================================

fn tui_quit_system(
    sim_control: Res<SimControl>,
    mut exit: EventWriter<AppExit>,
    term_res: Option<ResMut<TerminalResource>>,
) {
    if sim_control.quit_requested {
        // Terminal をリストアしてから終了
        if let Some(mut res) = term_res {
            let _ = disable_raw_mode();
            let _ = execute!(res.terminal.backend_mut(), LeaveAlternateScreen);
            let _ = res.terminal.show_cursor();
        }
        exit.send(AppExit::Success);
    }
}

/// ステップ実行フラグをリセットするシステム
fn tui_step_reset_system(mut sim_control: ResMut<SimControl>) {
    if sim_control.step_once {
        sim_control.step_once = false;
    }
}

// ============================================================
// ヘルパー
// ============================================================

/// 人口表示のフォーマット (例: 10.0M, 2.5M, 150.3K)
fn format_population(pop: f64) -> String {
    if pop >= 1_000_000.0 {
        format!("{:.1}M", pop / 1_000_000.0)
    } else if pop >= 1_000.0 {
        format!("{:.1}K", pop / 1_000.0)
    } else {
        format!("{:.0}", pop)
    }
}

/// 短縮版人口表示
fn format_short_population(pop: f64) -> String {
    if pop >= 1_000_000.0 {
        format!("{:.1}M", pop / 1_000_000.0)
    } else if pop >= 1_000.0 {
        format!("{:.0}K", pop / 1_000.0)
    } else {
        format!("{:.0}", pop)
    }
}

fn resource_type_short_name(rt: &crate::components::economy::ResourceType) -> &'static str {
    match rt {
        crate::components::economy::ResourceType::Food => "食",
        crate::components::economy::ResourceType::Minerals => "鉱",
        crate::components::economy::ResourceType::Energy => "エ",
        crate::components::economy::ResourceType::ManufacturedGoods => "工",
    }
}

fn tech_field_name(field: &crate::components::technology::TechField) -> &'static str {
    match field {
        crate::components::technology::TechField::Agriculture => "Agriculture",
        crate::components::technology::TechField::Mining => "Mining",
        crate::components::technology::TechField::EnergyTech => "Energy",
        crate::components::technology::TechField::Manufacturing => "Manufacturing",
        crate::components::technology::TechField::MilitaryTech => "Military",
        crate::components::technology::TechField::SpaceNavigation => "Navigation",
        crate::components::technology::TechField::EnvironmentalTech => "Environmental",
        crate::components::technology::TechField::NuclearFusion => "Nuclear Fusion",
    }
}

fn event_kind_emoji(kind: &crate::components::events::EventKind) -> &'static str {
    match kind {
        crate::components::events::EventKind::Plague => "🦠",
        crate::components::events::EventKind::BountifulHarvest => "🍎",
        crate::components::events::EventKind::MineralDiscovery => "💎",
        crate::components::events::EventKind::BabyBoom => "👶",
        crate::components::events::EventKind::EnergyCrisis => "⚡",
        crate::components::events::EventKind::TechBreakthrough => "🚀",
        crate::components::events::EventKind::Rebellion => "🔥",
        crate::components::events::EventKind::TradeBoom => "💰",
        crate::components::events::EventKind::Earthquake => "🌋",
        crate::components::events::EventKind::RadiationStorm => "☢️",
        crate::components::events::EventKind::MeteoriteImpact => "☄️",
        crate::components::events::EventKind::EnvironmentalDisaster => "☣️",
    }
}

fn event_kind_name(kind: &crate::components::events::EventKind) -> &'static str {
    match kind {
        crate::components::events::EventKind::Plague => "Epidemic",
        crate::components::events::EventKind::BountifulHarvest => "Bountiful Harvest",
        crate::components::events::EventKind::MineralDiscovery => "Mineral Discovery",
        crate::components::events::EventKind::BabyBoom => "Baby Boom",
        crate::components::events::EventKind::EnergyCrisis => "Energy Crisis",
        crate::components::events::EventKind::TechBreakthrough => "Tech Breakthrough",
        crate::components::events::EventKind::Rebellion => "Rebellion",
        crate::components::events::EventKind::TradeBoom => "Trade Boom",
        crate::components::events::EventKind::Earthquake => "Earthquake",
        crate::components::events::EventKind::RadiationStorm => "Radiation Storm",
        crate::components::events::EventKind::MeteoriteImpact => "Meteorite Impact",
        crate::components::events::EventKind::EnvironmentalDisaster => "Environmental Disaster",
    }
}

fn format_event_effect(effect: &crate::components::events::EventEffect) -> String {
    let mut parts = Vec::new();
    if effect.population_change != 0.0 { parts.push(format!("Pop: {:+.0}", effect.population_change)); }
    if effect.food_change != 0.0 { parts.push(format!("Food: {:+.0}", effect.food_change)); }
    if effect.minerals_change != 0.0 { parts.push(format!("Min: {:+.0}", effect.minerals_change)); }
    if effect.energy_change != 0.0 { parts.push(format!("En: {:+.0}", effect.energy_change)); }
    if effect.goods_change != 0.0 { parts.push(format!("Goods: {:+.0}", effect.goods_change)); }
    if effect.research_change != 0.0 { parts.push(format!("Res: {:+.0}", effect.research_change)); }
    parts.join(", ")
}
