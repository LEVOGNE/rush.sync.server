pub mod input;
pub mod keyboard;

use crossterm::event::{self as crossterm_event, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::Receiver<AppEvent>,
    shutdown_tx: Vec<Sender<()>>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel(100);
        let mut shutdown_tx = Vec::new();

        let (input_shutdown_tx, mut input_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(input_shutdown_tx);

        let input_tx = tx.clone();
        tokio::spawn(async move {
            let mut last_event_time = Instant::now();
            let min_event_interval = Duration::from_millis(16);
            let mut last_resize_time = Instant::now();
            let min_resize_interval = Duration::from_millis(50);

            loop {
                tokio::select! {
                    _ = input_shutdown_rx.recv() => break,
                    _ = async {
                        if crossterm_event::poll(Duration::from_millis(99)).unwrap() {
                            let now = Instant::now();

                            if let Ok(event) = crossterm_event::read() {
                                match event {
                                    CrosstermEvent::Key(key) => {
                                        if now.duration_since(last_event_time) >= min_event_interval {
                                            let _ = input_tx.send(AppEvent::Input(key)).await;
                                            last_event_time = now;
                                        }
                                    }
                                    CrosstermEvent::Resize(width, height) => {
                                        if now.duration_since(last_resize_time) >= min_resize_interval {
                                            let _ = input_tx.send(AppEvent::Resize(width, height)).await;
                                            last_resize_time = now;

                                            log::trace!(
                                                "🔄 Resize event throttled: {}x{} ({}ms since last)",
                                                width, height,
                                                now.duration_since(last_resize_time).as_millis()
                                            );
                                        } else {
                                            log::trace!(
                                                "⏭️ Resize event dropped (too fast): {}x{}",
                                                width, height
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } => {}
                }
            }
        });

        let (tick_shutdown_tx, mut tick_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(tick_shutdown_tx);

        let tick_tx = tx;
        tokio::spawn(async move {
            let mut interval = interval(tick_rate);
            loop {
                tokio::select! {
                    _ = tick_shutdown_rx.recv() => break,
                    _ = interval.tick() => {
                        let _ = tick_tx.send(AppEvent::Tick).await;
                    }
                }
            }
        });

        EventHandler { rx, shutdown_tx }
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    pub async fn shutdown(&mut self) {
        for tx in self.shutdown_tx.iter() {
            let _ = tx.send(()).await;
        }
    }
}
