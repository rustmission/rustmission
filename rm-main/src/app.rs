use rm_config::Config;
use rm_shared::action::event_to_action;
use rm_shared::action::Action;
use rm_shared::action::Mode;
use std::sync::Arc;

use crate::{
    transmission::{self, TorrentAction},
    tui::Tui,
    ui::{components::Component, MainWindow},
};

use anyhow::{Error, Result};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};
use transmission_rpc::{types::SessionGet, TransClient};

#[derive(Clone)]
pub struct Ctx {
    pub client: Arc<Mutex<TransClient>>,
    pub config: Arc<Config>,
    pub session_info: Arc<SessionGet>,
    action_tx: UnboundedSender<Action>,
    trans_tx: UnboundedSender<TorrentAction>,
}

impl Ctx {
    async fn new(
        client: Arc<Mutex<TransClient>>,
        config: Config,
        action_tx: UnboundedSender<Action>,
        trans_tx: UnboundedSender<TorrentAction>,
    ) -> Result<Self> {
        let response = client.lock().await.session_get().await;
        match response {
            Ok(res) => {
                let session_info = Arc::new(res.arguments);
                return Ok(Self {
                    client,
                    config: Arc::new(config),
                    action_tx,
                    trans_tx,
                    session_info,
                });
            }
            Err(e) => {
                let config_path = config.directories.main_path;
                return Err(Error::msg(format!(
                    "{e}\nIs the connection info in {:?} correct?",
                    config_path
                )));
            }
        }
    }

    pub(crate) fn send_action(&self, action: Action) {
        self.action_tx.send(action).unwrap();
    }

    pub(crate) fn send_torrent_action(&self, action: TorrentAction) {
        self.trans_tx.send(action).unwrap();
    }
}

pub struct App {
    should_quit: bool,
    ctx: Ctx,
    action_rx: UnboundedReceiver<Action>,
    main_window: MainWindow,
    mode: Mode,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let client = Arc::new(Mutex::new(transmission::utils::client_from_config(&config)));

        let (trans_tx, trans_rx) = mpsc::unbounded_channel();
        let ctx = Ctx::new(client, config, action_tx, trans_tx).await?;

        tokio::spawn(transmission::action_handler(ctx.clone(), trans_rx));
        Ok(Self {
            should_quit: false,
            main_window: MainWindow::new(ctx.clone()),
            action_rx,
            ctx,
            mode: Mode::Normal,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tui = Tui::new()?;

        tui.enter()?;

        self.render(&mut tui)?;

        self.main_loop(&mut tui).await?;

        tui.exit()?;
        Ok(())
    }

    async fn main_loop(&mut self, tui: &mut Tui) -> Result<()> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(250));
        loop {
            let tui_event = tui.next();
            let action = self.action_rx.recv();
            let tick_action = interval.tick();

            tokio::select! {
                _ = tick_action => {
                    self.ctx.action_tx.send(Action::Tick).unwrap();
                },

                event = tui_event => {
                    if let Some(action) = event_to_action(self.mode, event.unwrap(), &self.ctx.config.keybindings.keymap) {
                        if let Some(action) = self.update(action).await {
                            self.ctx.action_tx.send(action).unwrap();
                        }
                    };
                },

                action = action => {
                    if let Some(action) = action {
                        if action.is_render() {
                            self.render(tui)?;
                        } else if action.is_quit() {
                            self.should_quit = true;
                        } else if let Some(action) = self.update(action).await {
                            self.ctx.action_tx.send(action).unwrap();
                        }
                    }
                }
            }

            if self.should_quit {
                break Ok(());
            }
        }
    }

    fn render(&mut self, tui: &mut Tui) -> Result<()> {
        tui.terminal.draw(|f| {
            self.main_window.render(f, f.size());
        })?;
        Ok(())
    }

    #[must_use]
    async fn update(&mut self, action: Action) -> Option<Action> {
        use Action as A;
        match &action {
            A::Render => Some(A::Render),

            A::HardQuit => {
                self.should_quit = true;
                None
            }

            A::SwitchToInputMode => {
                self.mode = Mode::Input;
                Some(A::Render)
            }

            A::SwitchToNormalMode => {
                self.mode = Mode::Normal;
                Some(A::Render)
            }

            _ => self.main_window.handle_actions(action),
        }
    }
}
