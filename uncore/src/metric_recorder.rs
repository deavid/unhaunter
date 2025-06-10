use bevy::diagnostic::{DiagnosticMeasurement, DiagnosticPath};
use bevy::utils::Instant;
use bevy::{diagnostic::DiagnosticsStore, prelude::*};
use std::sync::{LazyLock, Mutex, mpsc};

const CHANNEL_CAPACITY: usize = 2048;

static DIAGNOSTIC_CHANNEL: LazyLock<StaticChannel> = LazyLock::new(StaticChannel::default);

#[derive(Debug, Clone)]
pub struct Data {
    pub path: DiagnosticPath,
    pub time: Instant,
    pub value: f64,
}

struct StaticChannel {
    tx: mpsc::SyncSender<Data>,
    rx: Mutex<mpsc::Receiver<Data>>,
}

impl Default for StaticChannel {
    fn default() -> Self {
        let (tx, rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
        Self {
            tx,
            rx: Mutex::new(rx),
        }
    }
}

fn receive_data(mut diag_store: ResMut<DiagnosticsStore>) {
    let rx_guard = DIAGNOSTIC_CHANNEL
        .rx
        .try_lock()
        .expect("uncore::metrics::receive_data was unable to lock for reading messages");

    for data in rx_guard.try_iter() {
        if let Some(diag) = diag_store.get_mut(&data.path) {
            diag.add_measurement(DiagnosticMeasurement {
                time: data.time,
                value: data.value,
            });
        }
    }
}

pub fn send_metric(path: &DiagnosticPath, value: f64) {
    let data = Data {
        path: path.clone(),
        time: Instant::now(),
        value,
    };
    if let Err(e) = DIAGNOSTIC_CHANNEL.tx.try_send(data) {
        error!("Unable to send metric {path:?}: {e:?}");
    }
}

pub struct TimeMeasure {
    path: DiagnosticPath,
    start: Instant,
}

impl TimeMeasure {
    pub fn start(path: DiagnosticPath) -> TimeMeasure {
        TimeMeasure {
            path,
            start: Instant::now(),
        }
    }

    pub fn end_ms(self) {
        self.path.tx(self.start.elapsed().as_secs_f64() * 1000.0);
    }
}

pub trait SendMetric {
    fn tx(&self, value: f64);
    fn time_measure(self) -> TimeMeasure;
}

impl SendMetric for DiagnosticPath {
    fn tx(&self, value: f64) {
        send_metric(self, value);
    }
    fn time_measure(self) -> TimeMeasure {
        TimeMeasure::start(self)
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(PostUpdate, receive_data);
}
