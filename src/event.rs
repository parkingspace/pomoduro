use crossterm::event::Event as CrosstermEvent;
use futures::StreamExt;

#[derive(Clone, Debug)]
pub enum Event {
    Crossterm(CrosstermEvent),
    Error,
    Render,
}

type Stream = std::pin::Pin<Box<dyn futures::Stream<Item = Event>>>;

pub struct Events {
    streams: tokio_stream::StreamMap<&'static str, Stream>,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            streams: tokio_stream::StreamMap::from_iter([
                ("crossterm", crossterm_stream()),
                ("render", render_stream()),
            ]),
        }
    }
}

impl Events {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_name, event)| event)
    }
}

fn crossterm_stream() -> Stream {
    use crossterm::event::EventStream;
    use crossterm::event::KeyEventKind;
    use CrosstermEvent::Key;

    Box::pin(EventStream::new().fuse().filter_map(|event| async move {
        match event {
            // Ignore key release / repeat events
            Ok(Key(key)) if key.kind == KeyEventKind::Release => None,
            Ok(event) => Some(Event::Crossterm(event)),
            Err(_) => Some(Event::Error),
        }
    }))
}

fn render_stream() -> Stream {
    use tokio_stream::wrappers::IntervalStream;

    const FRAME_RATE: f64 = 15.0;
    let render_delay = std::time::Duration::from_secs_f64(1.0 / FRAME_RATE);
    let render_interval = tokio::time::interval(render_delay);
    Box::pin(IntervalStream::new(render_interval).map(|_| Event::Render))
}
