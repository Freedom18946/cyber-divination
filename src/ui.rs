//! `ratatui` 渲染层 — 三色·去框·呼吸设计。

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Wrap},
};

use crate::{
    app::{App, AppPhase},
    yijing,
};

// ── 三色原则 ─────────────────────────────────────────────

const TEXT: Color = Color::Rgb(232, 235, 238); // 主色：近白
const WARM: Color = Color::Rgb(220, 198, 132); // 强调色：暖光
const DIM: Color = Color::Rgb(90, 96, 102); // 辅助色：深灰

// ── 主入口 ───────────────────────────────────────────────

/// 绘制当前帧。
pub fn render(frame: &mut Frame, app: &App) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // 顶部留白
            Constraint::Length(2),  // header
            Constraint::Length(1),  // 分隔线
            Constraint::Min(8),    // body
            Constraint::Length(1), // footer
        ])
        .split(frame.area());

    render_header(frame, root[1], app);
    render_separator(frame, root[2]);

    if app.phase() == AppPhase::Welcome {
        render_welcome(frame, root[3], app);
    } else {
        let body = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(root[3]);

        match app.phase() {
            AppPhase::Result => render_result_scene(frame, body[0], app),
            _ => render_ritual_scene(frame, body[0], app),
        }
        render_trace(frame, body[1], app);
    }

    render_footer(frame, root[4], app);
}

// ── Header ───────────────────────────────────────────────

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let mut spans = vec![Span::styled(
        " 数字起卦",
        Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
    )];

    if app.phase() != AppPhase::Welcome {
        spans.push(Span::styled(
            format!("  {:02}/18 · {}/6爻", app.casts_completed(), app.completed_lines()),
            Style::default().fg(DIM),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), rows[0]);
    frame.render_widget(Paragraph::new(stage_dots(app)), rows[1]);
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(line, Style::default().fg(DIM)))),
        area,
    );
}

// ── 阶段轨道（圆点链） ──────────────────────────────────

fn stage_dots(app: &App) -> Line<'static> {
    const STAGES: [(&str, AppPhase); 5] = [
        ("静场", AppPhase::Welcome),
        ("起数", AppPhase::Casting),
        ("收束", AppPhase::Assembling),
        ("显卦", AppPhase::ReverseConfirm),
        ("结果", AppPhase::Result),
    ];

    let current = phase_order(app.phase());
    let mut spans = vec![Span::styled(" ", Style::default())];

    for (idx, (label, phase)) in STAGES.iter().enumerate() {
        let order = phase_order(*phase);
        let (dot, color) = if order < current {
            ("◉", DIM)
        } else if order == current {
            ("●", WARM)
        } else {
            ("○", DIM)
        };

        let modifier = if order == current {
            Modifier::BOLD
        } else {
            Modifier::empty()
        };

        spans.push(Span::styled(
            format!("{} {}", dot, label),
            Style::default().fg(color).add_modifier(modifier),
        ));
        if idx + 1 != STAGES.len() {
            spans.push(Span::styled(" ─ ", Style::default().fg(DIM)));
        }
    }

    Line::from(spans)
}

// ── Welcome（全宽居中·呼吸） ─────────────────────────────

fn render_welcome(frame: &mut Frame, area: Rect, app: &App) {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(1),
            Constraint::Percentage(40),
        ])
        .split(area);

    // 呼吸闪烁：非对称周期，1.6s 亮 / 0.8s 暗
    let phase = app.tick_count() % 40;
    let breathing_on = phase < 26;
    let style = if breathing_on {
        Style::default().fg(WARM)
    } else {
        Style::default().fg(DIM)
    };

    frame.render_widget(
        Paragraph::new(Line::from(Span::styled("心中默问。Enter 落子。", style)))
            .alignment(Alignment::Center),
        v[1],
    );
}

// ── Ritual（取数·收束·显卦确认） ─────────────────────────

fn render_ritual_scene(frame: &mut Frame, area: Rect, app: &App) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10)])
        .split(area);

    // 焦点区
    let mut focus = Vec::new();
    let instruction = app.instruction();
    if !instruction.is_empty() {
        focus.push(Line::from(Span::styled(
            instruction,
            Style::default().fg(TEXT).add_modifier(Modifier::BOLD),
        )));
    }
    let caption = app.focus_caption();
    if !caption.is_empty() {
        focus.push(Line::from(Span::styled(
            caption,
            Style::default().fg(DIM),
        )));
    }

    frame.render_widget(
        Paragraph::new(focus)
            .wrap(Wrap { trim: true })
            .block(Block::default().padding(Padding::new(2, 0, 1, 0))),
        sections[0],
    );

    // 下半：卦势 + 节奏
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(sections[1]);

    frame.render_widget(
        Paragraph::new(formation_lines(app))
            .wrap(Wrap { trim: true })
            .block(Block::default().padding(Padding::new(2, 1, 0, 0))),
        body[0],
    );
    frame.render_widget(
        Paragraph::new(tempo_lines(app))
            .wrap(Wrap { trim: true })
            .block(Block::default().padding(Padding::new(1, 1, 0, 0))),
        body[1],
    );
}

// ── Result ────────────────────────────────────────────────

fn render_result_scene(frame: &mut Frame, area: Rect, app: &App) {
    let Some(result) = app.current_result() else {
        return;
    };
    let stage = reveal_stage(app);

    // Stage 0: 纯黑 + 呼吸光点
    if stage == 0 {
        let v = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(42),
                Constraint::Length(1),
                Constraint::Percentage(42),
            ])
            .split(area);

        let dot_on = (app.tick_count() / 10).is_multiple_of(2);
        let style = if dot_on {
            Style::default().fg(WARM)
        } else {
            Style::default().fg(DIM)
        };

        frame.render_widget(
            Paragraph::new(Line::from(Span::styled("·", style)))
                .alignment(Alignment::Center),
            v[1],
        );
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(10)])
        .split(area);

    // 英雄区：卦名大字
    let heading = if let Some(relating) = result.relating {
        format!(
            "{}  →  {}",
            hero_name(result.primary.name),
            hero_name(relating.name)
        )
    } else {
        hero_name(result.primary.name)
    };

    let hero = vec![
        Line::from(""),
        Line::from(Span::styled(
            heading,
            Style::default().fg(WARM).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    frame.render_widget(
        Paragraph::new(hero).alignment(Alignment::Center),
        sections[0],
    );

    // 下半：本卦 + 之卦
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(sections[1]);

    frame.render_widget(
        Paragraph::new(primary_hexagram_lines(app, stage))
            .alignment(Alignment::Center)
            .block(Block::default().padding(Padding::new(2, 1, 1, 0))),
        body[0],
    );
    frame.render_widget(
        Paragraph::new(relating_and_notes_lines(app, stage))
            .wrap(Wrap { trim: true })
            .block(Block::default().padding(Padding::new(1, 1, 1, 0))),
        body[1],
    );
}

// ── Trace（右侧·仅左线） ────────────────────────────────

fn render_trace(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(DIM))
        .padding(Padding::new(1, 0, 0, 0));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(8)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(trace_summary_lines(app)).wrap(Wrap { trim: true }),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(trace_log_lines(app)).wrap(Wrap { trim: false }),
        sections[1],
    );
}

// ── Footer（单行回响） ──────────────────────────────────

fn render_footer(frame: &mut Frame, area: Rect, app: &App) {
    let content = if let Some(pulse) = app.pulse() {
        Span::styled(pulse.message(), Style::default().fg(WARM))
    } else {
        Span::styled(
            match app.phase() {
                AppPhase::Welcome => "Enter 开始  ·  q 退出",
                AppPhase::Result => "Enter 再问  ·  q 退出",
                _ => "Enter 继续  ·  q 退出",
            },
            Style::default().fg(DIM),
        )
    };

    frame.render_widget(
        Paragraph::new(Line::from(content))
            .block(Block::default().padding(Padding::new(1, 0, 0, 0))),
        area,
    );
}

// ── 内容生成 ─────────────────────────────────────────────

fn formation_lines(app: &App) -> Vec<Line<'static>> {
    match app.phase() {
        AppPhase::Welcome => Vec::new(),
        AppPhase::Casting => {
            let flash = app
                .last_cast_elapsed()
                .is_some_and(|d| d.as_millis() < 250);
            let mut lines = Vec::new();

            for (idx, value) in app.line_sums().iter().copied().enumerate().rev() {
                let is_last = idx == app.completed_lines().saturating_sub(1);
                let glyph = wide_glyph(value);
                let style = if is_last && flash {
                    Style::default().fg(WARM).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(TEXT)
                };

                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:<4}  ", line_slot(idx)),
                        Style::default().fg(DIM),
                    ),
                    Span::styled(glyph, style),
                    Span::styled(
                        format!("  {}", line_meta(value)),
                        Style::default().fg(DIM),
                    ),
                ]));
            }

            let pending = app.completed_lines();
            if pending < 6 {
                let trio_slot = app.casts_completed() % 3;
                let indicator = match trio_slot {
                    0 => "○ ○ ○",
                    1 => "● ○ ○",
                    2 => "● ● ○",
                    _ => "○ ○ ○",
                };
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }
                let ind_style = if flash {
                    Style::default().fg(WARM).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(WARM)
                };
                lines.push(Line::from(Span::styled(
                    format!("{}  {}", line_slot(pending), indicator),
                    ind_style,
                )));
            }

            lines
        }
        AppPhase::Assembling | AppPhase::ReverseConfirm => app
            .line_sums()
            .iter()
            .copied()
            .enumerate()
            .rev()
            .map(|(idx, value)| {
                Line::from(vec![
                    Span::styled(
                        format!("{:<4}  ", line_slot(idx)),
                        Style::default().fg(DIM),
                    ),
                    Span::styled(wide_glyph(value), Style::default().fg(TEXT)),
                    Span::styled(
                        format!("  {}", line_meta(value)),
                        Style::default().fg(DIM),
                    ),
                ])
            })
            .collect(),
        AppPhase::Result => Vec::new(),
    }
}

fn tempo_lines(app: &App) -> Vec<Line<'static>> {
    match app.phase() {
        AppPhase::Welcome => Vec::new(),
        AppPhase::Casting => {
            let next_cast = app.casts_completed() + 1;
            let current_line = app.casts_completed() / 3 + 1;
            vec![
                Line::from(Span::styled(
                    format!("数位  {:02}", next_cast),
                    Style::default().fg(TEXT),
                )),
                Line::from(Span::styled(
                    format!("第 {} 爻", current_line),
                    Style::default().fg(DIM),
                )),
            ]
        }
        AppPhase::Assembling => vec![Line::from(Span::styled(
            "收束",
            Style::default().fg(TEXT),
        ))],
        AppPhase::ReverseConfirm => vec![Line::from(Span::styled(
            "显卦前夜",
            Style::default().fg(TEXT),
        ))],
        AppPhase::Result => Vec::new(),
    }
}

fn trace_summary_lines(app: &App) -> Vec<Line<'static>> {
    let flash = app
        .last_cast_elapsed()
        .is_some_and(|d| d.as_millis() < 250);
    let mut lines = vec![
        Line::from(Span::styled(
            format!("取数  {:02}/18", app.casts_completed()),
            Style::default().fg(TEXT),
        )),
        Line::from(Span::styled(
            format!("成爻  {:02}/06", app.completed_lines()),
            Style::default().fg(TEXT),
        )),
    ];

    if let Some((fingerprint, _)) = app.last_entropy() {
        let style = if flash {
            Style::default().fg(WARM)
        } else {
            Style::default().fg(DIM)
        };
        lines.push(Line::from(Span::styled(
            format!("熵印  {}", fingerprint),
            style,
        )));
    }

    lines
}

fn trace_log_lines(app: &App) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    for row in app.history_snapshot().into_iter().take(6) {
        lines.push(Line::from(Span::styled(row, Style::default().fg(DIM))));
    }

    if !lines.is_empty() {
        lines.push(Line::from(""));
    }

    let limit = if app.phase() == AppPhase::Result {
        7
    } else {
        10
    };

    let flash = app
        .last_cast_elapsed()
        .is_some_and(|d| d.as_millis() < 250);

    for (i, entry) in app.journal_entries().iter().rev().take(limit).enumerate() {
        let style = if i == 0 && flash {
            Style::default().fg(WARM)
        } else {
            Style::default().fg(TEXT)
        };
        lines.push(Line::from(vec![
            Span::styled("· ", Style::default().fg(DIM)),
            Span::styled(entry.clone(), style),
        ]));
    }

    lines
}

fn primary_hexagram_lines(app: &App, stage: u8) -> Vec<Line<'static>> {
    let Some(result) = app.current_result() else {
        return Vec::new();
    };

    let subtitle = if result.changing_lines.is_empty() {
        format!("#{:02}  无变爻", result.primary.index)
    } else {
        format!(
            "#{:02}  {} 个变爻",
            result.primary.index,
            result.changing_lines.len()
        )
    };

    if stage < 2 {
        return vec![Line::from(Span::styled(
            subtitle,
            Style::default().fg(DIM),
        ))];
    }

    let lines_visible = lines_to_reveal(app);
    let mut output = vec![
        Line::from(Span::styled(subtitle, Style::default().fg(DIM))),
        Line::from(""),
    ];

    for (drawn_idx, (idx, value)) in app
        .line_sums()
        .iter()
        .copied()
        .enumerate()
        .rev()
        .enumerate()
    {
        if drawn_idx >= lines_visible {
            output.push(Line::from(""));
            continue;
        }

        let is_changing = matches!(value, 6 | 9);
        let glyph = hero_glyph(value);

        // 变爻脉冲：大部分时间亮，短暂暗
        let style = if is_changing {
            let pulse = !(app.tick_count() / 8).is_multiple_of(3);
            if pulse {
                Style::default().fg(WARM).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(DIM)
            }
        } else {
            Style::default().fg(TEXT)
        };

        let change_mark = if is_changing { "  ◄" } else { "" };
        output.push(Line::from(vec![
            Span::styled(format!("{} ", line_slot(idx)), Style::default().fg(DIM)),
            Span::styled(glyph, style),
            Span::styled(change_mark, Style::default().fg(WARM)),
        ]));
    }

    output
}

fn relating_and_notes_lines(app: &App, stage: u8) -> Vec<Line<'static>> {
    let Some(result) = app.current_result() else {
        return Vec::new();
    };

    if stage < 3 {
        return vec![Line::from(Span::styled(
            "变化层尚未展开。",
            Style::default().fg(DIM),
        ))];
    }

    let mut lines = Vec::new();

    if let Some(relating) = result.relating {
        lines.push(Line::from(Span::styled(
            format!("之卦  {}", relating.name),
            Style::default().fg(WARM).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            format!("#{:02}", relating.index),
            Style::default().fg(DIM),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "之卦  无",
            Style::default().fg(WARM).add_modifier(Modifier::BOLD),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        if result.changing_lines.is_empty() {
            "变爻  无".to_string()
        } else {
            format!("变爻  {}", result.changing_lines.join(" · "))
        },
        Style::default().fg(TEXT),
    )));

    if stage >= 4 {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("变后六爻", Style::default().fg(WARM))));
        for (idx, value) in result
            .transformed_lines
            .iter()
            .copied()
            .enumerate()
            .rev()
        {
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", line_slot(idx)), Style::default().fg(DIM)),
                Span::styled(wide_glyph(value), Style::default().fg(TEXT)),
            ]));
        }
    }

    lines
}

// ── 辅助 ─────────────────────────────────────────────────

/// 渐进揭示时序（拉长节奏）。
fn reveal_stage(app: &App) -> u8 {
    let ms = app.phase_elapsed().as_millis();
    match ms {
        0..=399 => 0,     // 纯黑 + 呼吸光点
        400..=899 => 1,   // 卦名浮现
        900..=1499 => 2,  // 六爻逐行显现
        1500..=2199 => 3, // 之卦与变爻淡入
        _ => 4,           // 完整展示
    }
}

/// Stage 2: 六爻自上而下逐行显现，每行间隔 100ms。
fn lines_to_reveal(app: &App) -> usize {
    let ms = app.phase_elapsed().as_millis();
    if ms < 900 {
        return 0;
    }
    (((ms - 900) / 100) as usize + 1).min(6)
}

/// 卦名字间距拉开，制造大字视觉。
fn hero_name(s: &str) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("  ")
}

/// 宽版爻线（仪式场景用）。
fn wide_glyph(value: u8) -> &'static str {
    if yijing::is_yang(value) {
        "━━━━━━━━━━━"
    } else {
        "━━━━   ━━━━"
    }
}

/// 英雄版爻线（结果页用，更宽）。
fn hero_glyph(value: u8) -> &'static str {
    if yijing::is_yang(value) {
        "━━━━━━━━━━━━━━━━━"
    } else {
        "━━━━━━━   ━━━━━━━"
    }
}

fn phase_order(phase: AppPhase) -> usize {
    match phase {
        AppPhase::Welcome => 0,
        AppPhase::Casting => 1,
        AppPhase::Assembling => 2,
        AppPhase::ReverseConfirm => 3,
        AppPhase::Result => 4,
    }
}

fn line_slot(idx: usize) -> &'static str {
    match idx {
        0 => "初爻",
        1 => "二爻",
        2 => "三爻",
        3 => "四爻",
        4 => "五爻",
        5 => "上爻",
        _ => "?",
    }
}

fn line_meta(value: u8) -> &'static str {
    match value {
        6 => "老阴 *",
        7 => "少阳",
        8 => "少阴",
        9 => "老阳 *",
        _ => "?",
    }
}

#[cfg(test)]
mod tests {
    use std::{env, fs, path::Path, thread, time::Duration};

    use anyhow::Result;
    use crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    use super::render;
    use crate::{
        app::App,
        entropy::{EntropySample, EntropySource},
    };

    struct ScriptedEntropy {
        digits: Vec<u8>,
        cursor: usize,
    }

    impl ScriptedEntropy {
        fn new(digits: Vec<u8>) -> Self {
            Self { digits, cursor: 0 }
        }
    }

    impl EntropySource for ScriptedEntropy {
        fn next_digit(&mut self, throw_no: usize) -> Result<EntropySample> {
            let digit = self.digits[self.cursor];
            self.cursor += 1;
            Ok(EntropySample {
                digit,
                fingerprint: format!("snap-{throw_no:02}"),
            })
        }
    }

    #[test]
    fn welcome_screen_snapshot() {
        let app = App::new(Box::new(ScriptedEntropy::new(vec![2; 18])));
        let snapshot = render_snapshot(&app, 84, 24);
        assert_snapshot("tests/snapshots/welcome_screen.txt", &snapshot);
    }

    #[test]
    fn welcome_screen_small_snapshot() {
        let app = App::new(Box::new(ScriptedEntropy::new(vec![2; 18])));
        let snapshot = render_snapshot(&app, 60, 18);
        assert_snapshot("tests/snapshots/welcome_screen_small.txt", &snapshot);
    }

    #[test]
    fn result_screen_snapshot_shows_relating_hexagram() {
        let mut app = App::new(Box::new(ScriptedEntropy::new(vec![
            2, 2, 2, 2, 2, 3, 2, 3, 3, 3, 3, 3, 2, 2, 3, 2, 3, 2,
        ])));

        app.handle_key(KeyCode::Enter.into()).expect("welcome");
        for _ in 1..18 {
            app.handle_key(KeyCode::Enter.into()).expect("casting");
        }
        app.handle_key(KeyCode::Enter.into()).expect("assembling");
        app.handle_key(KeyCode::Enter.into()).expect("reverse");
        thread::sleep(Duration::from_millis(2300));

        let snapshot = render_snapshot(&app, 112, 30);
        assert_snapshot("tests/snapshots/result_screen.txt", &snapshot);
    }

    fn render_snapshot(app: &App, width: u16, height: u16) -> String {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).expect("test terminal");
        terminal.draw(|frame| render(frame, app)).expect("draw");
        buffer_to_string(terminal.backend().buffer())
    }

    fn buffer_to_string(buffer: &Buffer) -> String {
        let mut rows = Vec::new();
        for y in 0..buffer.area.height {
            let mut row = String::new();
            for x in 0..buffer.area.width {
                let symbol = buffer[(x, y)].symbol();
                if !symbol.is_empty() {
                    row.push_str(symbol);
                }
            }
            rows.push(row.trim_end().to_string());
        }
        rows.join("\n")
    }

    fn assert_snapshot(path: &str, actual: &str) {
        let snapshot_path = Path::new(path);
        if env::var("UPDATE_SNAPSHOTS").ok().as_deref() == Some("1") {
            fs::write(snapshot_path, actual).expect("write snapshot");
            return;
        }

        let expected = fs::read_to_string(snapshot_path).expect("read snapshot");
        assert_eq!(actual, expected.trim_end());
    }
}
