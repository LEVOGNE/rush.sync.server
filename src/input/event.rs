use crossterm::event::{self as crossterm_event, Event as CrosstermEvent, KeyEvent};
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{interval, Duration};

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

        // Shutdown-Kanal für Input-Task
        let (input_shutdown_tx, mut input_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(input_shutdown_tx);

        // Input-Handler Task
        let input_tx = tx.clone();
        tokio::spawn(async move {
            let mut last_event_time = tokio::time::Instant::now();
            let min_event_interval = Duration::from_millis(50);

            loop {
                tokio::select! {
                    _ = input_shutdown_rx.recv() => break,
                    _ = async {
                        if crossterm_event::poll(Duration::from_millis(100)).unwrap() {
                            let now = tokio::time::Instant::now();
                            if now.duration_since(last_event_time) >= min_event_interval {
                                if let Ok(event) = crossterm_event::read() {
                                    match event {
                                        CrosstermEvent::Key(key) => {
                                            let _ = input_tx.send(AppEvent::Input(key)).await;
                                        }
                                        CrosstermEvent::Resize(width, height) => {
                                            let _ = input_tx.send(AppEvent::Resize(width, height)).await;
                                        }
                                        _ => {}
                                    }
                                    last_event_time = now;
                                }
                            }
                        }
                    } => {}
                }
            }
        });

        // Tick-Handler Task
        let (tick_shutdown_tx, mut tick_shutdown_rx) = mpsc::channel(1);
        shutdown_tx.push(tick_shutdown_tx);

        let tick_tx = tx;
        tokio::spawn(async move {
            let mut interval = interval(tick_rate); // Verwende den übergebenen tick_rate Parameter
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
