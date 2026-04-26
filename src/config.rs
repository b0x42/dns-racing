use clap::Parser;
use std::net::IpAddr;

#[derive(Parser)]
#[command(name = "dns-racing", version, about = "Race your DNS server against public resolvers")]
pub struct Args {
    #[arg(long, env = "CUSTOM_DNS", default_value = "192.168.0.5")]
    pub custom_dns: IpAddr,

    #[arg(long, env = "CUSTOM_DNS_LABEL", default_value = "My DNS")]
    pub custom_dns_label: String,

    #[arg(long = "public-dns", env = "CLOUDFLARE", default_value = "1.1.1.1")]
    pub public_dns: IpAddr,

    #[arg(long, env = "PUBLIC_DNS_LABEL", default_value = "Cloudflare")]
    pub public_dns_label: String,

    #[arg(long, env = "EXTRA_DNS", default_value = "")]
    pub extra_dns: String,

    #[arg(long, env = "RPS", default_value = "25")]
    pub rps: u32,

    #[arg(long, env = "STATS_EVERY", default_value = "1000")]
    pub stats_every: u64,

    #[arg(long, env = "TIMEOUT", default_value = "5000")]
    pub timeout: u64,

    #[arg(long, env = "WINDOW", default_value = "500")]
    pub window: usize,

    #[arg(long, env = "WARMUP_ROUNDS", default_value = "2")]
    pub warmup_rounds: u32,

    #[arg(long, env = "CACHE_HIT_MS", default_value = "1.0")]
    pub cache_hit_ms: f64,
}

pub struct ExtraDns {
    pub ip: IpAddr,
    pub label: String,
}

pub fn parse() -> Args {
    let args = Args::parse();

    if args.rps == 0 {
        eprintln!("Error: RPS must be a positive number.");
        std::process::exit(1);
    }
    if args.custom_dns == args.public_dns {
        eprintln!("Error: CUSTOM_DNS and public DNS are the same IP. Set them to different servers.");
        std::process::exit(1);
    }
    args
}

pub fn parse_extra_dns(s: &str) -> Vec<ExtraDns> {
    s.split(',')
        .filter(|e| !e.trim().is_empty())
        .map(|entry| {
            let entry = entry.trim();
            let (ip_str, label) = match entry.split_once(':') {
                Some((ip, lbl)) => (ip, lbl.to_string()),
                None => (entry, entry.to_string()),
            };
            let ip: IpAddr = ip_str.parse().unwrap_or_else(|_| {
                eprintln!("Error: Invalid IP address: {ip_str}");
                std::process::exit(1);
            });
            ExtraDns { ip, label }
        })
        .collect()
}
