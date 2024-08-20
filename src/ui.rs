use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, Paragraph},
    Frame,
};

use crate::app::App;
use crate::timer::TimerStatus;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.size();

    let vertical_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3)])
        .horizontal_margin(1)
        .split(area);

    let horizontal_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(2, 3)])
        .split(vertical_layout[0]);

    match app.get_timer() {
        Some(timer) if timer.get_status() == TimerStatus::Done => {
            // Done UI:
            f.render_widget(
                Paragraph::new("Timer is done! You made it!")
                    .block(Block::default().borders(Borders::ALL)),
                f.size(),
            )
        }
        Some(timer) => {
            // Running UI:
            let ratio =
                (timer.elapsed_time().as_secs_f64() / timer.get_duration().as_secs_f64()).min(1.0);
            let label = timer.to_string();

            let progress = Gauge::default()
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(timer.get_name()),
                )
                .gauge_style(
                    Style::default()
                        .fg(Color::Magenta)
                        .bg(Color::Black)
                        .add_modifier(Modifier::ITALIC | Modifier::BOLD),
                )
                .use_unicode(true)
                .label(label)
                .ratio(ratio);

            f.render_widget(progress, horizontal_layout[0]);
        }
        None => {
            f.render_widget(
                Paragraph::new("No active timer").block(Block::default().borders(Borders::ALL)),
                vertical_layout[0],
            );
        }
    }
}
