use chrono::{DateTime, Utc};
use std::sync::{Arc, LazyLock, Mutex};
use tracing::Level;
use tracing_subscriber::{Layer, layer::Context};

pub static LOG_COLLECTOR: LazyLock<LogCollector> = LazyLock::new(|| LogCollector::new(1000));

#[derive(Debug, Clone)]
pub struct LogCollector {
    logs: Arc<Mutex<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogCollector {
    pub fn new(max_entries: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::with_capacity(max_entries))),
            max_entries,
        }
    }

    pub fn len(&self) -> usize {
        self.logs.lock().unwrap().len()
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        self.logs.lock().unwrap().clone()
    }

    pub fn clear(&self) {
        self.logs.lock().unwrap().clear();
    }

    fn add_log(&self, entry: LogEntry) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(entry);

        while logs.len() > self.max_entries {
            logs.remove(0);
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: Level,
    pub target: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
}

pub struct CollectorLayer {
    collector: LogCollector,
}

impl CollectorLayer {
    pub fn new(collector: LogCollector) -> Self {
        Self { collector }
    }
}

impl<S> Layer<S> for CollectorLayer
where
    S: tracing::Subscriber + for<'lookup> tracing_subscriber::registry::LookupSpan<'lookup>,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let entry = LogEntry {
            timestamp: Utc::now(),
            level: *metadata.level(),
            target: metadata.target().to_string(),
            message: visitor.message,
            file: metadata.file().map(|s| s.to_string()),
            line: metadata.line(),
        };

        self.collector.add_log(entry);
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);

            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        }
    }
}
