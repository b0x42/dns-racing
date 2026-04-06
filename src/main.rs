mod config;
mod csv;
mod display;
mod query;
mod stats;

use std::io::IsTerminal;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cfg = config::parse();
    let servers = query::build_servers(&cfg);

    let csv_path = csv::make_path();
    let (csv_tx, csv_rx) = mpsc::unbounded_channel();
    let csv_handle = tokio::spawn(csv::writer_task(csv_path.clone(), csv_rx));

    display::banner(&cfg, &servers, &csv_path);

    if let Err(e) = query::warmup(&cfg, &servers).await {
        eprintln!("\x1b[31mError: {e}\x1b[0m");
        std::process::exit(1);
    }

    let mut stats_store = stats::Store::new(&servers, cfg.window);
    let mut domain_idx: usize = 0;
    let domains = query::shuffled_domains();
    let start = Instant::now();
    let mut last_stats = start;
    let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / cfg.rps as f64));
    let mut table_lines: usize = 0;

    let shutdown = shutdown_signal();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let domain = &domains[domain_idx % domains.len()];
                domain_idx += 1;

                let mut set = JoinSet::new();
                for (i, server) in servers.iter().enumerate() {
                    let resolver = server.resolver.clone();
                    let domain = domain.clone();
                    let timeout = cfg.timeout;
                    set.spawn(async move {
                        let result = query::resolve(&resolver, &domain, timeout).await;
                        (i, domain, result)
                    });
                }

                while let Some(Ok((idx, domain, result))) = set.join_next().await {
                    stats_store.record(idx, &domain, &result, cfg.cache_hit_ms);
                    let _ = csv_tx.send(csv::Row {
                        server_ip: servers[idx].ip.clone(),
                        domain,
                        latency_ms: result.duration.as_secs_f64() * 1000.0,
                        status: result.status,
                    });
                }

                let now = Instant::now();
                if now.duration_since(last_stats) >= Duration::from_millis(cfg.stats_every) {
                    table_lines = display::live_stats(&stats_store, &servers, start.elapsed(), table_lines);
                    last_stats = now;
                }
            }
            _ = &mut shutdown => {
                break;
            }
        }
    }

    drop(csv_tx);
    display::live_stats(&stats_store, &servers, start.elapsed(), table_lines);
    display::domain_breakdown(&stats_store, &servers);
    display::verdict(&stats_store, &servers);
    let _ = csv_handle.await;
    println!("\n\x1b[32mResults saved → {csv_path}\x1b[0m\n");
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = signal::ctrl_c();

    #[cfg(unix)]
    {
        let mut term = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let esc = esc_key();
        tokio::select! {
            _ = ctrl_c => {}
            _ = term.recv() => {}
            _ = esc => {}
        }
    }

    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
    }
}

#[cfg(unix)]
async fn esc_key() {
    use std::io::Read;
    if !std::io::stdin().is_terminal() {
        return std::future::pending().await;
    }
    tokio::task::spawn_blocking(|| {
        use std::os::fd::AsRawFd;
        let fd = std::io::stdin().as_raw_fd();
        let orig = unsafe {
            let mut t = std::mem::zeroed();
            libc::tcgetattr(fd, &mut t);
            let orig = t;
            t.c_lflag &= !(libc::ICANON | libc::ECHO);
            t.c_cc[libc::VMIN] = 1;
            t.c_cc[libc::VTIME] = 0;
            libc::tcsetattr(fd, libc::TCSANOW, &t);
            orig
        };
        let mut buf = [0u8; 1];
        loop {
            if std::io::stdin().read(&mut buf).is_ok() && buf[0] == 0x1b {
                break;
            }
        }
        unsafe { libc::tcsetattr(fd, libc::TCSANOW, &orig) };
    })
    .await
    .ok();
}
