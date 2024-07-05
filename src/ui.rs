use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::timer::{Timer, TimerDisplay};

fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render(f: &mut Frame, timer: &Timer) {
    if timer.is_done() {
        f.render_widget(
            Paragraph::new(timer.get_status().to_string())
                .block(Block::default().borders(Borders::ALL)),
            f.size(),
        )
    } else {
        let area = f.size();
        let ratio = timer.elapsed_time().as_secs_f64().floor() / timer.get_duration().as_secs_f64();

        // TODO: show correct time
        let label = match timer.get_display() {
            TimerDisplay::Remaining => format!("{}", timer),
            TimerDisplay::Elapsed => format!("{}", timer.elapsed_time().as_secs_f64()),
            TimerDisplay::Percentage => format!(
                "{}",
                timer.elapsed_time().as_secs_f64().floor() / timer.get_duration().as_secs_f64()
            ),
        };

        // Display remaining time using gauge widget
        f.render_widget(
            Gauge::default()
                .block(Block::default().title("Remaining").borders(Borders::ALL))
                .gauge_style(
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Black)
                        .add_modifier(Modifier::ITALIC),
                )
                .label(Span::from(label))
                .ratio(ratio),
            centered_rect(area, 60, 50),
        );

        // NOTE: This is just for debugging
        // Display ratio
        f.render_widget(
            Paragraph::new(timer.get_status().to_string()).block(Block::bordered()),
            area,
        )
    }
}
