use std::env::var;

use zai_rs::usage::CodingPlanUsageRequest;
use zai_rs::{client::ZaiClient, usage::CodingPlanUsageSummary};

use color_eyre::{Report, Result};
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Gauge},
};

fn main() -> Result<()> {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let usage = rt.block_on(request_limits())?;

    let terminal = ratatui::init();
    let result = run(terminal, &usage);
    ratatui::restore();
    result
}

async fn request_limits() -> Result<CodingPlanUsageSummary> {
    let api_key = var("ZAI_API_KEY")
        .or_else(|_| var("ZHIPU_API_KEY"))
        .map_err(|_| Report::msg("neither ZAI_API_KEY nor ZHIPU_API_KEY is set"))?;

    let client = ZaiClient::builder(api_key).build()?;

    let resp = CodingPlanUsageRequest::new().send_via(&client).await?;

    Ok(resp.summary())
}

fn run(mut terminal: DefaultTerminal, usage: &CodingPlanUsageSummary) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, usage))?;
        if matches!(event::read()?, Event::Key(_)) {
            break Ok(());
        }
    }
}

fn render(f: &mut Frame, usage: &CodingPlanUsageSummary) {
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(f.area());

    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Min(15),
            Constraint::Min(0),
        ])
        .split(horizontal_chunks[1]);

    let gauge_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(usage.limits.iter().map(|_| Constraint::Length(3)))
        .split(vertical_chunks[1]);

    for (limit, chunk) in usage.limits.iter().zip(gauge_chunks.iter()) {
        let resets = limit
            .next_reset_at
            .map(|time| format!(" (resets {})", time.format("%Y-%m-%d %H:%M UTC")))
            .unwrap_or_default();
        let title = format!(" {} - {}%{} ", limit.type_, limit.percentage, resets);

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(title))
            .ratio((limit.percentage / 100.0).clamp(0.0, 1.0))
            .label(format!("{}/{}", limit.used, limit.quota));
        f.render_widget(gauge, *chunk);
    }
}
