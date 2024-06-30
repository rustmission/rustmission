use std::{io, time::Duration};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{Event, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::{FutureExt, StreamExt};
use ratatui::backend::CrosstermBackend as Backend;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

pub struct Tui {
    pub terminal: ratatui::Terminal<Backend<std::io::Stdout>>,
    pub task: JoinHandle<Result<()>>,
    pub cancellation_token: CancellationToken,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
}

impl Tui {
    pub(crate) fn new() -> Result<Self> {
        let terminal = ratatui::Terminal::new(Backend::new(std::io::stdout()))?;
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let task = tokio::spawn(async { Ok(()) });
        Ok(Self {
            terminal,
            task,
            cancellation_token,
            event_rx,
            event_tx,
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let cancellation_token = self.cancellation_token.clone();
        let event_tx = self.event_tx.clone();

        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            loop {
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = cancellation_token.cancelled() => break,
                  event = crossterm_event => Self::handle_crossterm_event::<io::Error>(event, &event_tx)?,
                }
            }
            Ok(())
        });
        Ok(())
    }

    fn handle_crossterm_event<T>(
        event: Option<Result<Event, io::Error>>,
        event_tx: &UnboundedSender<Event>,
    ) -> Result<()> {
        match event {
            Some(Ok(Event::Key(key))) => {
                if key.kind == KeyEventKind::Press {
                    event_tx.send(Event::Key(key)).unwrap();
                }
            }
            Some(Ok(Event::Resize(x, y))) => event_tx.send(Event::Resize(x, y)).unwrap(),
            Some(Err(e)) => Err(e)?,
            _ => (),
        }
        Ok(())
    }

    pub(crate) fn stop(&self) {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                break;
            }
        }
    }

    pub(crate) fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen, cursor::Hide)?;
        self.start()?;
        Ok(())
    }

    pub(crate) fn exit(&mut self) -> Result<()> {
        self.stop();
        if crossterm::terminal::is_raw_mode_enabled()? {
            self.terminal.flush()?;
            crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, cursor::Show)?;
            crossterm::terminal::disable_raw_mode()?;
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}
