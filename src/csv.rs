use crate::query::Status;
use chrono::Utc;
use std::io::Write;
use tokio::sync::mpsc;

pub struct Row {
    pub server_ip: String,
    pub domain: String,
    pub latency_ms: f64,
    pub status: Status,
}

pub fn make_path() -> String {
    let ts = Utc::now().format("%Y-%m-%dT%H-%M-%S");
    format!("dns_racing_{ts}.csv")
}

pub async fn writer_task(path: String, mut rx: mpsc::UnboundedReceiver<Row>) {
    let file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error: Could not create CSV file {path}: {e}");
            return;
        }
    };
    let mut file = std::io::BufWriter::new(file);
    let _ = writeln!(file, "timestamp,server,domain,latency_ms,status");

    while let Some(row) = rx.recv().await {
        let ts = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        if writeln!(
            file,
            "{},{},{},{:.2},{}",
            ts, row.server_ip, row.domain, row.latency_ms, row.status
        )
        .is_err()
        {
            eprintln!("Warning: CSV write failed, stopping CSV logging");
            break;
        }
    }

    let _ = file.flush();
}
