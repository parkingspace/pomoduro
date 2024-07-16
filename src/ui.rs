use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Frame,
};

use crate::timer::{Timer, TimerStatus};

// TODO: UI FOR POMODORO SESSIONS!
pub fn render(f: &mut Frame, timer: &Timer) {
    if timer.get_status() == TimerStatus::Done {
        // Done UI:
        f.render_widget(
            Paragraph::new("Timer is done! You made it!")
                .block(Block::default().borders(Borders::ALL)),
            f.size(),
        )
    } else {
        // Running UI:
        let area = f.size();
        // TODO: check if this computation is safe
        let ratio = timer.elapsed_time().as_secs_f64().floor() / timer.get_duration().as_secs_f64();

        // let (title, label) = match timer.get_display() {
        //     TimerDisplay::Remaining => ("Remaining", timer.to_string()),
        //     TimerDisplay::Elapsed => (
        //         "Elapsed",
        //         timer.format_duration(timer.elapsed_time()).to_string(),
        //     ),
        //     TimerDisplay::Percentage => (
        //         "Progress",
        //         format!(
        //             "{:.2}%",
        //             timer.elapsed_time().as_secs_f32() / timer.get_duration().as_secs_f32() * 100.0
        //         ),
        //     ),
        // };

        let label = timer.to_string();

        let progress = Gauge::default()
            .block(Block::bordered().title("My Timer"))
            .gauge_style(
                Style::default()
                    .fg(Color::Magenta)
                    .bg(Color::Black)
                    .add_modifier(Modifier::ITALIC | Modifier::BOLD),
            )
            .use_unicode(true)
            .label(label)
            .ratio(ratio);

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3)])
            .horizontal_margin(1)
            .split(area);

        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(2, 3)])
            .split(vertical_layout[0]);

        f.render_widget(progress, horizontal_layout[0]);
    }
}
