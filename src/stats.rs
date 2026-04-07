use crate::query::{QueryResult, Status};
use std::collections::{HashMap, VecDeque};

pub struct Entry {
    pub ms: f64,
    pub ok: bool,
    pub blocked: bool,
    pub cache_hit: bool,
}

pub struct Computed {
    pub ok: usize,
    pub cache_hits: usize,
    pub blocked: usize,
    pub errors: usize,
    pub min: f64,
    pub avg: f64,
    pub p95: f64,
    pub p99: f64,
    pub max: f64,
}

pub struct DomainAvg {
    pub sum: f64,
    pub count: usize,
}

pub struct Store {
    pub windows: Vec<VecDeque<Entry>>,
    pub domain_avgs: HashMap<(String, usize), DomainAvg>,
    window_size: usize,
}

impl Store {
    pub fn new(servers: &[crate::query::Server], window_size: usize) -> Self {
        Self {
            windows: (0..servers.len()).map(|_| VecDeque::new()).collect(),
            domain_avgs: HashMap::new(),
            window_size,
        }
    }

    pub fn record(&mut self, server_idx: usize, domain: &str, result: &QueryResult, cache_hit_ms: f64) {
        let ms = result.duration.as_secs_f64() * 1000.0;
        let ok = result.status == Status::Ok;
        let blocked = result.status == Status::Nxdomain;
        let cache_hit = ok && ms < cache_hit_ms;

        let window = &mut self.windows[server_idx];
        if window.len() >= self.window_size {
            window.pop_front();
        }
        window.push_back(Entry { ms, ok, blocked, cache_hit });

        if ok {
            let key = (domain.to_string(), server_idx);
            let avg = self.domain_avgs.entry(key).or_insert(DomainAvg { sum: 0.0, count: 0 });
            avg.sum += ms;
            avg.count += 1;
        }
    }

    pub fn compute(&self, server_idx: usize) -> Option<Computed> {
        let window = &self.windows[server_idx];
        let mut ok_ms = Vec::new();
        let mut blocked = 0;
        let mut errors = 0;
        let mut cache_hits = 0;

        for e in window {
            if e.ok {
                ok_ms.push(e.ms);
                if e.cache_hit { cache_hits += 1; }
            } else if e.blocked {
                blocked += 1;
            } else {
                errors += 1;
            }
        }

        if ok_ms.is_empty() { return None; }

        ok_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = ok_ms.len();
        let avg = ok_ms.iter().sum::<f64>() / len as f64;

        Some(Computed {
            ok: len,
            cache_hits,
            blocked,
            errors,
            min: ok_ms[0],
            avg,
            p95: ok_ms[(len as f64 * 0.95).ceil() as usize - 1].min(ok_ms[len - 1]),
            p99: ok_ms[(len as f64 * 0.99).ceil() as usize - 1].min(ok_ms[len - 1]),
            max: ok_ms[len - 1],
        })
    }
}
