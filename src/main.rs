//! `cyber-divination` 二进制入口。
//!
//! 这里保持极薄，只负责组装模块并启动 TUI。

mod app;
mod entropy;
mod tui;
mod ui;
mod yijing;

use anyhow::Result;

fn main() -> Result<()> {
    let mut terminal = tui::init_terminal()?;
    let app_result = app::run(&mut terminal);
    tui::restore_terminal(&mut terminal)?;
    app_result
}
