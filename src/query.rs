use crate::config::{self, Args};
use hickory_resolver::Resolver;
use hickory_resolver::config::{ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use rand::seq::SliceRandom;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type TokioResolver = Resolver<TokioRuntimeProvider>;

pub struct Server {
    pub label: String,
    pub ip: String,
    pub color: &'static str,
    pub resolver: Arc<TokioResolver>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Ok,
    Nxdomain,
    Error,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Ok => write!(f, "ok"),
            Status::Nxdomain => write!(f, "nxdomain"),
            Status::Error => write!(f, "error"),
        }
    }
}

pub struct QueryResult {
    pub duration: Duration,
    pub status: Status,
}

const EXTRA_COLORS: &[&str] = &["\x1b[35m", "\x1b[34m", "\x1b[33m"];

fn build_resolver(ip: IpAddr, timeout_ms: u64) -> Arc<TokioResolver> {
    let ns = NameServerConfig::new(ip, false, vec![ConnectionConfig::udp()]);
    let cfg = ResolverConfig::from_parts(None, vec![], vec![ns]);
    let mut opts = ResolverOpts::default();
    opts.timeout = Duration::from_millis(timeout_ms);
    opts.attempts = 1;
    opts.cache_size = 0;
    let resolver = Resolver::builder_with_config(cfg, TokioRuntimeProvider::default())
        .with_options(opts)
        .build()
        .expect("failed to build resolver");
    Arc::new(resolver)
}

pub fn build_servers(cfg: &Args) -> Vec<Server> {
    let mut servers = vec![
        Server {
            label: cfg.custom_dns_label.clone(),
            ip: cfg.custom_dns.to_string(),
            color: "\x1b[36m",
            resolver: build_resolver(cfg.custom_dns, cfg.timeout),
        },
        Server {
            label: cfg.public_dns_label.clone(),
            ip: cfg.public_dns.to_string(),
            color: "\x1b[32m",
            resolver: build_resolver(cfg.public_dns, cfg.timeout),
        },
    ];
    for (i, extra) in config::parse_extra_dns(&cfg.extra_dns)
        .into_iter()
        .enumerate()
    {
        servers.push(Server {
            label: extra.label,
            ip: extra.ip.to_string(),
            color: EXTRA_COLORS[i % EXTRA_COLORS.len()],
            resolver: build_resolver(extra.ip, cfg.timeout),
        });
    }
    servers
}

pub async fn resolve(resolver: &TokioResolver, domain: &str, timeout_ms: u64) -> QueryResult {
    let start = Instant::now();
    let result = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        resolver.lookup_ip(domain),
    )
    .await;
    let duration = start.elapsed();
    let status = match result {
        Ok(Ok(_)) => Status::Ok,
        Ok(Err(e)) => {
            if e.is_no_records_found() {
                Status::Nxdomain
            } else {
                Status::Error
            }
        }
        Err(_) => Status::Error,
    };
    QueryResult { duration, status }
}

const DOMAINS: &[&str] = &[
    "google.com",
    "youtube.com",
    "facebook.com",
    "twitter.com",
    "instagram.com",
    "reddit.com",
    "github.com",
    "stackoverflow.com",
    "amazon.com",
    "netflix.com",
    "wikipedia.org",
    "cloudflare.com",
    "apple.com",
    "microsoft.com",
    "linkedin.com",
    "twitch.tv",
    "discord.com",
    "spotify.com",
    "tiktok.com",
    "whatsapp.com",
    "zoom.us",
    "dropbox.com",
    "slack.com",
    "heise.de",
    "spiegel.de",
    "bbc.com",
    "nytimes.com",
    "reuters.com",
    "theguardian.com",
    "medium.com",
];

pub fn shuffled_domains() -> Vec<String> {
    let mut domains: Vec<String> = DOMAINS.iter().map(|s| s.to_string()).collect();
    domains.shuffle(&mut rand::thread_rng());
    domains
}

pub async fn warmup(cfg: &Args, servers: &[Server]) -> Result<(), String> {
    let total = DOMAINS.len() * cfg.warmup_rounds as usize;
    eprint!("  Warming up server caches ({total} queries per server)...");

    let mut error_counts = vec![0usize; servers.len()];
    let total_queries = total;

    for _ in 0..cfg.warmup_rounds {
        for domain in DOMAINS {
            let mut set = tokio::task::JoinSet::new();
            for (i, server) in servers.iter().enumerate() {
                let resolver = server.resolver.clone();
                let domain = domain.to_string();
                let timeout = cfg.timeout;
                set.spawn(async move {
                    let r = resolve(&resolver, &domain, timeout).await;
                    (i, r)
                });
            }
            while let Some(Ok((i, result))) = set.join_next().await {
                if result.status == Status::Error {
                    error_counts[i] += 1;
                }
            }
        }
    }

    for (i, &errs) in error_counts.iter().enumerate() {
        if errs == total_queries {
            return Err(format!(
                "Warmup failed — {} ({}) is unreachable. Check the server IP.",
                servers[i].label, servers[i].ip
            ));
        }
    }

    eprintln!(" \x1b[32mdone\x1b[0m\n");
    Ok(())
}
