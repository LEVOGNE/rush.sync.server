pub mod keyboard;
pub mod state;

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent, KeyEvent, MouseEventKind};
use std::sync::OnceLock;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
pub enum AppEvent {
    Input(KeyEvent),
    MouseScrollUp,
    MouseScrollDown,
    Tick,
    Resize(u16, u16),
    /// Background progress message from async commands (start all, stop all, etc.)
    Progress(String),
}

// ===== Global Progress Channel =====
// Commands can send progress messages to the TUI without blocking.

static PROGRESS_TX: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

/// Initialize the global progress channel. Returns the receiver.
/// Called once by ScreenManager at startup.
pub fn init_progress_channel() -> mpsc::UnboundedReceiver<String> {
    let (tx, rx) = mpsc::unbounded_channel();
    let _ = PROGRESS_TX.set(tx);
    rx
}

/// Send a progress message to the TUI. Safe to call from any thread/task.
/// Returns false if the channel is not initialized or closed.
pub fn send_progress(message: String) -> bool {
    if let Some(tx) = PROGRESS_TX.get() {
        tx.send(message).is_ok()
    } else {
        false
    }
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    shutdown_tx: Vec<Sender<()>>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut shutdown_tx = Vec::new();

        // Input event handler
        let (input_shutdown_tx, input_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(input_shutdown_tx);
        Self::spawn_input_handler(tx.clone(), input_shutdown_rx);

        // Tick handler
        let (tick_shutdown_tx, tick_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(tick_shutdown_tx);
        Self::spawn_tick_handler(tx, tick_rate, tick_shutdown_rx);

        EventHandler { rx, shutdown_tx }
    }

    fn spawn_input_handler(tx: mpsc::Sender<AppEvent>, mut shutdown_rx: mpsc::Receiver<()>) {
        tokio::spawn(async move {
            let (mut last_key_time, mut last_resize_time) = (Instant::now(), Instant::now());
            let (key_interval, resize_interval) =
                (Duration::from_millis(16), Duration::from_millis(50));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => break,
                    _ = async {
                        if crossterm_event::poll(Duration::from_millis(99)).unwrap_or(false) {
                            if let Ok(event) = crossterm_event::read() {
                                let now = Instant::now();
                                match event {
                                    CrosstermEvent::Key(key) if now.duration_since(last_key_time) >= key_interval => {
                                        let _ = tx.send(AppEvent::Input(key)).await;
                                        last_key_time = now;
                                    }
                                    CrosstermEvent::Mouse(mouse) => {
                                        match mouse.kind {
                                            MouseEventKind::ScrollUp => {
                                                let _ = tx.send(AppEvent::MouseScrollUp).await;
                                            }
                                            MouseEventKind::ScrollDown => {
                                                let _ = tx.send(AppEvent::MouseScrollDown).await;
                                            }
                                            _ => {}
                                        }
                                    }
                                    CrosstermEvent::Resize(w, h) if now.duration_since(last_resize_time) >= resize_interval => {
                                        let _ = tx.send(AppEvent::Resize(w, h)).await;
                                        last_resize_time = now;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } => {}
                }
            }
        });
    }

    fn spawn_tick_handler(
        tx: mpsc::Sender<AppEvent>,
        tick_rate: Duration,
        mut shutdown_rx: mpsc::Receiver<()>,
    ) {
        tokio::spawn(async move {
            let mut interval = interval(tick_rate);
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => break,
                    _ = interval.tick() => { let _ = tx.send(AppEvent::Tick).await; }
                }
            }
        });
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    pub async fn shutdown(&mut self) {
        for tx in &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
    }
}
