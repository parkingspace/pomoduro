use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{Block, Gauge, Paragraph},
    Frame,
};

use crate::timer::Timer;

pub fn render(f: &mut Frame, timer: &Timer) {
    let area = f.size();
    let ratio =
        1.0 - (timer.get_remaining_time().as_secs_f64() / timer.get_duration().as_secs_f64());

    // Display remaining time using gauge widget
    f.render_widget(
        Gauge::default()
            .block(Block::bordered().title("Remaining"))
            .gauge_style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC),
            )
            .label(timer.to_string())
            .ratio(ratio),
        area,
    );

    // NOTE: This is just for debugging
    // Display ratio
    f.render_widget(
        Paragraph::new(ratio.to_string()).block(Block::bordered()),
        area,
    )
}
