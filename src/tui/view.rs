use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::AppState;

pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(frame.area());

    let titles = state
        .session_manager
        .ordered_ids()
        .into_iter()
        .map(|id| Line::from(format!("{:?}", id)))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Sessions"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(tabs, chunks[0]);
    frame.render_widget(
        Paragraph::new("Terminal view attaches here")
            .block(Block::default().borders(Borders::ALL)),
        chunks[1],
    );
}