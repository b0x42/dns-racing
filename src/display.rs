use crate::config::Args;
use crate::query::Server;
use crate::stats::Store;
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const YELLOW: &str = "\x1b[33m";

fn fmt_ms(v: f64) -> String { format!("{v:.1}ms") }

fn is_tty() -> bool { io::stdout().is_terminal() }

pub fn banner(cfg: &Args, servers: &[Server], csv_path: &str) {
    println!("\n{BOLD}DNS Racing{RESET}");
    for s in servers {
        println!("  {}{}{:<12}{RESET} {}", s.color, BOLD, s.label, s.ip);
    }
    let total_rps = cfg.rps as usize * servers.len();
    println!("  {:<12} {} req/s per server  →  {} total", "Rate", cfg.rps, total_rps);
    println!("  {:<12} last {} results per server", "Window", cfg.window);
    println!("  {:<12} {csv_path}", "Output");
    println!("  Press {BOLD}Ctrl+C{RESET} to stop\n");
}

pub fn live_stats(store: &Store, servers: &[Server], elapsed: Duration, prev_lines: usize) -> usize {
    let mut out = io::stdout().lock();
    let cols = [12, 6, 7, 5, 9, 9, 9, 9, 9];
    let hr = |l: &str, m: &str, r: &str| -> String {
        let inner: Vec<String> = cols.iter().map(|w| "─".repeat(w + 2)).collect();
        format!("{l}{}{r}", inner.join(m))
    };
    let cell = |v: &str, w: usize| -> String { format!(" {:>w$} ", v, w = w) };

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(format!("{BOLD}Stats after {}s{RESET}", elapsed.as_secs()));
    lines.push(hr("┌", "┬", "┐"));
    lines.push(format!(
        "│{}│{}│{}│{}│{}│{}│{}│{}│{}│",
        cell("Server", cols[0]), cell("OK", cols[1]),
        cell("Blocked", cols[2]), cell("Err", cols[3]), cell("Min", cols[4]),
        cell("Avg", cols[5]), cell("p95", cols[6]), cell("p99", cols[7]), cell("Max", cols[8])
    ));
    lines.push(hr("├", "┼", "┤"));

    for (i, s) in servers.iter().enumerate() {
        if let Some(st) = store.compute(i) {
            lines.push(format!(
                "│ {}{BOLD}{:<w$}{RESET} │{}│{}│{}│{}│{}│{}│{}│{}│",
                s.color, s.label,
                cell(&st.ok.to_string(), cols[1]),
                cell(&st.blocked.to_string(), cols[2]),
                cell(&st.errors.to_string(), cols[3]),
                cell(&fmt_ms(st.min), cols[4]),
                cell(&fmt_ms(st.avg), cols[5]),
                cell(&fmt_ms(st.p95), cols[6]),
                cell(&fmt_ms(st.p99), cols[7]),
                cell(&fmt_ms(st.max), cols[8]),
                w = cols[0],
            ));
        } else {
            lines.push(format!(
                "│ {}{BOLD}{:<w$}{RESET} │ (no results yet)", s.color, s.label, w = cols[0]
            ));
        }
    }
    lines.push(hr("└", "┴", "┘"));
    lines.push(format!("  {YELLOW}{BOLD}Stop the race with ESC or Ctrl+C{RESET}"));

    if is_tty() && prev_lines > 0 {
        write!(out, "\x1b[{prev_lines}A").ok();
    }
    for line in &lines {
        if is_tty() {
            write!(out, "\r\x1b[2K{line}\n").ok();
        } else {
            writeln!(out, "{line}").ok();
        }
    }
    out.flush().ok();
    lines.len()
}

pub fn domain_breakdown(store: &Store, servers: &[Server]) {
    let domains: Vec<String> = store.domain_avgs.keys().map(|(d, _)| d.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    if domains.is_empty() { return; }

    let mut rows: Vec<(String, Vec<f64>, f64)> = Vec::new();
    for domain in &domains {
        let mut avgs = Vec::new();
        let mut all_present = true;
        for i in 0..servers.len() {
            if let Some(da) = store.domain_avgs.get(&(domain.clone(), i)) {
                avgs.push(da.sum / da.count as f64);
            } else {
                all_present = false;
                break;
            }
        }
        if !all_present { continue; }
        let diff = avgs.iter().skip(1).cloned().reduce(f64::max).unwrap_or(0.0) - avgs[0];
        rows.push((domain.clone(), avgs, diff));
    }
    rows.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    if rows.is_empty() { return; }

    let d_col = 18;
    let diff_col = 10;
    let col_w: Vec<usize> = servers.iter().map(|s| s.label.len().max(9)).collect();

    let hr = |l: &str, m: &str, r: &str| -> String {
        let mut s = format!("{l}{}", "─".repeat(d_col + 2));
        for w in &col_w { s += &format!("{m}{}", "─".repeat(w + 2)); }
        s += &format!("{m}{}{r}", "─".repeat(diff_col + 2));
        s
    };

    let title: Vec<String> = servers.iter().map(|s| format!("{}{}{RESET}", s.color, s.label)).collect();
    println!("\n{BOLD}Per-domain breakdown{RESET} ({})", title.join(" vs "));
    println!("{}", hr("┌", "┬", "┐"));
    print!("│ {:<d_col$} │", "Domain");
    for (i, s) in servers.iter().enumerate() { print!(" {:>w$} │", s.label, w = col_w[i]); }
    println!(" {:>diff_col$} │", "Diff");
    println!("{}", hr("├", "┼", "┤"));

    for (domain, avgs, diff) in &rows {
        let diff_str = format!("{}{:.1}ms", if *diff >= 0.0 { "+" } else { "" }, diff);
        let diff_color = if *diff > 0.5 { servers[0].color } else { YELLOW };
        print!("│ {:<d_col$} │", domain);
        for (i, avg) in avgs.iter().enumerate() { print!(" {:>w$} │", fmt_ms(*avg), w = col_w[i]); }
        println!(" {diff_color}{:>diff_col$}{RESET} │", diff_str);
    }
    println!("{}", hr("└", "┴", "┘"));
}

pub fn verdict(store: &Store, servers: &[Server]) {
    let ranks = ["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th"];
    let mut ranked: Vec<(usize, f64, f64, f64)> = Vec::new();
    for (i, _) in servers.iter().enumerate() {
        if let Some(st) = store.compute(i) {
            ranked.push((i, st.avg, st.p95, st.min));
        }
    }
    if ranked.is_empty() { return; }
    ranked.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let cols = [4, 12, 9, 9, 9, 10];
    let hr = |l: &str, m: &str, r: &str| -> String {
        let inner: Vec<String> = cols.iter().map(|w| "─".repeat(w + 2)).collect();
        format!("{l}{}{r}", inner.join(m))
    };
    let cell = |v: &str, w: usize| -> String { format!(" {:>w$} ", v, w = w) };

    println!("\n{BOLD}Race Results{RESET}");
    println!("{}", hr("┌", "┬", "┐"));
    println!("│{}│{}│{}│{}│{}│{}│",
        cell("Rank", cols[0]), cell("Server", cols[1]),
        cell("Avg", cols[2]), cell("p95", cols[3]),
        cell("Min", cols[4]), cell("Diff", cols[5]));
    println!("{}", hr("├", "┼", "┤"));

    let fastest = ranked[0].1;
    for (pos, &(idx, avg, p95, min)) in ranked.iter().enumerate() {
        let s = &servers[idx];
        let diff = if pos == 0 { "—".to_string() } else { format!("+{}", fmt_ms(avg - fastest)) };
        let rank = ranks.get(pos).map(|r| r.to_string()).unwrap_or_else(|| format!("{}th", pos + 1));
        println!("│{}│ {}{BOLD}{:<w$}{RESET} │{}│{}│{}│{}│",
            cell(&rank, cols[0]),
            s.color, s.label,
            cell(&fmt_ms(avg), cols[2]),
            cell(&fmt_ms(p95), cols[3]),
            cell(&fmt_ms(min), cols[4]),
            cell(&diff, cols[5]),
            w = cols[1],
        );
    }
    println!("{}", hr("└", "┴", "┘"));
}
