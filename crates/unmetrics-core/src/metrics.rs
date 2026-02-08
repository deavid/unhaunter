use bevy::diagnostic::{DiagnosticMeasurement, DiagnosticPath, DiagnosticsStore};
use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_platform::time::Instant;
use std::sync::{mpsc, LazyLock, Mutex};

const CHANNEL_CAPACITY: usize = 32768;

pub static DIAGNOSTIC_CHANNEL: LazyLock<StaticChannel> = LazyLock::new(StaticChannel::default);

#[derive(Debug, Clone)]
pub struct Data {
    pub path: DiagnosticPath,
    pub time: Instant,
    pub value: f64,
}

pub struct StaticChannel {
    pub tx: mpsc::SyncSender<Data>,
    pub rx: Mutex<mpsc::Receiver<Data>>,
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

pub fn receive_data(
    mut diag_store: ResMut<DiagnosticsStore>,
    mut remembered_paths: Local<HashSet<DiagnosticPath>>,
) {
    let rx_guard = DIAGNOSTIC_CHANNEL
        .rx
        .try_lock()
        .expect("unmetrics-core::receive_data was unable to lock for reading messages");

    let mut frame_data: HashMap<DiagnosticPath, f64> = HashMap::default();
    let now = Instant::now();

    for data in rx_guard.try_iter() {
        *frame_data.entry(data.path.clone()).or_insert(0.0) += data.value;
        remembered_paths.insert(data.path);
    }

    for path in remembered_paths.iter() {
        if let Some(diag) = diag_store.get_mut(path) {
            let value = frame_data.get(path).cloned().unwrap_or(0.0);
            diag.add_measurement(DiagnosticMeasurement {
                time: now,
                value,
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
