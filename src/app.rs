//! 应用状态机与事件循环。

use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    entropy::{EntropySource, SystemEntropy},
    tui::AppTerminal,
    ui, yijing,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppPhase {
    Welcome,
    Casting,
    Assembling,
    ReverseConfirm,
    Result,
}

/// UI 底部的短时仪式反馈。
pub struct RitualPulse {
    message: String,
    until: Instant,
}

impl RitualPulse {
    pub fn message(&self) -> &str {
        &self.message
    }

    fn expired(&self) -> bool {
        Instant::now() >= self.until
    }
}

/// 应用运行时状态。
pub struct App {
    phase: AppPhase,
    phase_started_at: Instant,
    raw_digits: Vec<u8>,
    line_sums: Vec<u8>,
    journal: Vec<String>,
    pulse: Option<RitualPulse>,
    should_quit: bool,
    entropy: Box<dyn EntropySource>,
    tick_count: u64,
    last_cast_at: Option<Instant>,
    last_entropy: Option<(String, u8)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new(Box::new(SystemEntropy))
    }
}

impl App {
    pub fn new(entropy: Box<dyn EntropySource>) -> Self {
        Self {
            phase: AppPhase::Welcome,
            phase_started_at: Instant::now(),
            raw_digits: Vec::with_capacity(18),
            line_sums: Vec::with_capacity(6),
            journal: vec!["等待落子。".to_string()],
            pulse: None,
            should_quit: false,
            entropy,
            tick_count: 0,
            last_cast_at: None,
            last_entropy: None,
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn phase(&self) -> AppPhase {
        self.phase
    }

    pub fn phase_elapsed(&self) -> Duration {
        self.phase_started_at.elapsed()
    }

    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    pub fn last_cast_elapsed(&self) -> Option<Duration> {
        self.last_cast_at.map(|t| t.elapsed())
    }

    pub fn last_entropy(&self) -> Option<(&str, u8)> {
        self.last_entropy.as_ref().map(|(fp, d)| (fp.as_str(), *d))
    }

    pub fn casts_completed(&self) -> usize {
        self.raw_digits.len()
    }

    pub fn completed_lines(&self) -> usize {
        self.line_sums.len()
    }

    pub fn pulse(&self) -> Option<&RitualPulse> {
        self.pulse.as_ref()
    }

    pub fn journal_entries(&self) -> &[String] {
        &self.journal
    }

    pub fn line_sums(&self) -> &[u8] {
        &self.line_sums
    }

    pub fn instruction(&self) -> String {
        match self.phase {
            AppPhase::Welcome => "心中默问。Enter 落子。".to_string(),
            AppPhase::Casting => format!("{:02} / 18", self.raw_digits.len() + 1),
            AppPhase::Assembling => "Enter 收束。".to_string(),
            AppPhase::ReverseConfirm => "Enter 显卦。".to_string(),
            AppPhase::Result => "再问一次？Enter。".to_string(),
        }
    }

    pub fn focus_caption(&self) -> String {
        match self.phase {
            AppPhase::Welcome => String::new(),
            AppPhase::Casting => {
                let line_no = self.raw_digits.len() / 3 + 1;
                format!("第 {} 爻", line_no)
            }
            AppPhase::Assembling => "六爻已备。".to_string(),
            AppPhase::ReverseConfirm => String::new(),
            AppPhase::Result => String::new(),
        }
    }

    pub fn current_result(&self) -> Option<yijing::HexagramResult> {
        (self.line_sums.len() == 6).then(|| yijing::analyze_hexagram(&self.line_sums))
    }

    pub fn history_snapshot(&self) -> Vec<String> {
        match self.phase {
            AppPhase::Welcome | AppPhase::Casting => {
                if self.raw_digits.is_empty() {
                    vec!["尚未落子。".to_string()]
                } else {
                    let mut rows = Vec::new();
                    for (idx, chunk) in self.raw_digits.chunks(3).enumerate() {
                        let joined = chunk
                            .iter()
                            .map(u8::to_string)
                            .collect::<Vec<_>>()
                            .join(" ");
                        if chunk.len() == 3 {
                            rows.push(format!(
                                "{}: [{}] -> {} / {}",
                                yijing::LINE_POSITIONS[idx],
                                joined,
                                self.line_sums[idx],
                                yijing::line_label(self.line_sums[idx])
                            ));
                        } else {
                            rows.push(format!("{}: [{}]", yijing::LINE_POSITIONS[idx], joined));
                        }
                    }
                    rows
                }
            }
            AppPhase::Assembling | AppPhase::ReverseConfirm | AppPhase::Result => self
                .line_sums
                .iter()
                .copied()
                .enumerate()
                .map(|(idx, value)| {
                    format!(
                        "{}: {} / {} / {}",
                        yijing::LINE_POSITIONS[idx],
                        value,
                        yijing::line_symbol(value),
                        yijing::line_label(value)
                    )
                })
                .collect(),
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) -> Result<()> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => self.handle_enter()?,
            _ => {}
        }
        Ok(())
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        if self.pulse.as_ref().is_some_and(RitualPulse::expired) {
            self.pulse = None;
        }
    }

    fn handle_enter(&mut self) -> Result<()> {
        match self.phase {
            AppPhase::Welcome => {
                self.transition_to(AppPhase::Casting);
                self.cast_digit()?;
            }
            AppPhase::Casting => self.cast_digit()?,
            AppPhase::Assembling => {
                self.transition_to(AppPhase::ReverseConfirm);
                self.push_journal("收束完成。");
                self.set_pulse("收束完成。");
            }
            AppPhase::ReverseConfirm => {
                self.transition_to(AppPhase::Result);
                let result = yijing::analyze_hexagram(&self.line_sums);
                self.push_journal(format!("本卦显形：{}", result.primary.name));
                let pulse = if let Some(relating) = result.relating {
                    format!(
                        "已显现 · {} -> {} ({:02} -> {:02})",
                        result.primary.name, relating.name, result.primary.index, relating.index
                    )
                } else {
                    format!(
                        "已显现 · {} ({:02})",
                        result.primary.name, result.primary.index
                    )
                };
                self.set_pulse(pulse);
            }
            AppPhase::Result => {
                let mut next = App::new(std::mem::replace(
                    &mut self.entropy,
                    Box::new(SystemEntropy),
                ));
                next.transition_to(AppPhase::Casting);
                next.cast_digit()?;
                *self = next;
            }
        }
        Ok(())
    }

    fn cast_digit(&mut self) -> Result<()> {
        let throw_no = self.raw_digits.len() + 1;
        let sample = self.entropy.next_digit(throw_no)?;
        self.raw_digits.push(sample.digit);
        self.last_cast_at = Some(Instant::now());
        self.push_journal(format!("#{:02} → {}", throw_no, sample.digit));
        let short_fp: String = sample.fingerprint.chars().take(10).collect();
        self.set_pulse(format!("{} · {}", short_fp, sample.digit));
        self.last_entropy = Some((sample.fingerprint, sample.digit));

        if self.raw_digits.len() % 3 == 0 {
            let line_idx = self.line_sums.len();
            let trio = &self.raw_digits[self.raw_digits.len() - 3..];
            let sum: u8 = trio.iter().copied().sum();
            self.line_sums.push(sum);
            self.push_journal(format!(
                "{} [{} {} {}] = {}",
                yijing::LINE_POSITIONS[line_idx],
                trio[0], trio[1], trio[2], sum,
            ));

            if self.line_sums.len() == 6 {
                self.transition_to(AppPhase::Assembling);
                self.push_journal("六爻已备。");
                self.set_pulse("六爻已备。");
            }
        }

        Ok(())
    }

    fn push_journal(&mut self, entry: impl Into<String>) {
        self.journal.push(entry.into());
        if self.journal.len() > 48 {
            let drain_to = self.journal.len() - 48;
            self.journal.drain(0..drain_to);
        }
    }

    fn set_pulse(&mut self, message: impl Into<String>) {
        self.pulse = Some(RitualPulse {
            message: message.into(),
            until: Instant::now() + Duration::from_millis(1400),
        });
    }

    fn transition_to(&mut self, phase: AppPhase) {
        self.phase = phase;
        self.phase_started_at = Instant::now();
    }
}

/// 主事件循环。
pub fn run(terminal: &mut AppTerminal) -> Result<()> {
    let mut app = App::default();

    while !app.should_quit() {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if event::poll(Duration::from_millis(60))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code)?;
                }
            }
        }

        app.tick();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use crossterm::event::KeyCode;

    use super::{App, AppPhase};
    use crate::entropy::{EntropySample, EntropySource};

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
                fingerprint: format!("test-{throw_no:02}"),
            })
        }
    }

    #[test]
    fn full_ritual_reaches_result_phase() {
        let mut app = App::new(Box::new(ScriptedEntropy::new(vec![
            2, 2, 2, 2, 2, 3, 2, 3, 3, 3, 3, 3, 2, 2, 3, 2, 3, 2,
        ])));

        app.handle_key(KeyCode::Enter)
            .expect("welcome -> first cast");
        for _ in 1..18 {
            app.handle_key(KeyCode::Enter).expect("casting");
        }

        assert_eq!(app.phase(), AppPhase::Assembling);
        assert_eq!(app.line_sums(), &[6, 7, 8, 9, 7, 7]);

        app.handle_key(KeyCode::Enter).expect("assembling");
        assert_eq!(app.phase(), AppPhase::ReverseConfirm);

        app.handle_key(KeyCode::Enter).expect("reverse confirm");
        assert_eq!(app.phase(), AppPhase::Result);

        let result = app.current_result().expect("hexagram result");
        assert_eq!(
            result.changing_lines,
            vec!["初六".to_string(), "九四".to_string()]
        );
        assert_eq!(result.primary.name, "天水讼");
        assert_eq!(result.relating.expect("relating").name, "风泽中孚");
    }
}
