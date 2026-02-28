use crate::memory;
use crate::Result;

#[derive(Debug)]
pub struct MemoryCommand;

impl MemoryCommand {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug, Clone)]
enum MemorySubcommand {
    Help,
    Info {
        json: bool,
        top: Option<usize>,
        all: bool,
    },
}

#[async_trait::async_trait]
impl crate::commands::command::Command for MemoryCommand {
    fn name(&self) -> &'static str {
        "mem"
    }

    fn description(&self) -> &'static str {
        "Show memory registry snapshot and process details"
    }

    fn matches(&self, command: &str) -> bool {
        if let Some(first) = command.split_whitespace().next() {
            return matches_exact!(first, "mem");
        }
        false
    }

    async fn execute(&self, args: &[&str]) -> Result<String> {
        // Examples:
        // - ["mem"] -> Help
        // - ["mem","help"]
        // - ["mem","info","--top","10","--json","--all"]
        // - ["info","--all"] (if handler already stripped "mem")
        let sub = parse_from_args(args);

        match sub {
            MemorySubcommand::Help => {
                return Ok(help_text());
            }
            MemorySubcommand::Info { json, top, all } => {
                let items = memory::snapshot();

                if json {
                    let total = memory::total_bytes();
                    let rss = crate::memory::process_rss_bytes();
                    let total_ram = crate::memory::total_ram_bytes();

                    let total_kb = (total as f64) / 1024.0;
                    let total_mb = total_kb / 1024.0;
                    let rss_kb = (rss as f64) / 1024.0;
                    let rss_mb = rss_kb / 1024.0;

                    let pct_registry_over_rss = if rss > 0 {
                        (total as f64) * 100.0 / (rss as f64)
                    } else {
                        0.0
                    };
                    let pct_rss_over_totalram = if total_ram > 0 {
                        (rss as f64) * 100.0 / (total_ram as f64)
                    } else {
                        0.0
                    };

                    let mut rows: Vec<_> = items
                        .iter()
                        .map(|r| (r.id.as_str(), format!("{:?}", r.kind), r.bytes))
                        .collect();
                    rows.sort_by(|a, b| b.2.cmp(&a.2));
                    if let Some(n) = top {
                        rows.truncate(n);
                    }

                    let mut out = String::new();
                    out.push_str("{\n  \"resources\": [\n");
                    for (i, (id, kind, bytes)) in rows.iter().enumerate() {
                        out.push_str(&format!(
                            "    {{ \"id\": \"{}\", \"kind\": \"{}\", \"bytes\": {} }}{}",
                            id,
                            kind,
                            bytes,
                            if i + 1 != rows.len() { ",\n" } else { "\n" }
                        ));
                    }
                    out.push_str("  ],\n");
                    out.push_str(&format!(
                        "  \"total_bytes\": {},\n  \"total_kb\": {:.2},\n  \"total_mb\": {:.3},\n",
                        total, total_kb, total_mb
                    ));
                    out.push_str(&format!(
                        "  \"rss_bytes\": {},\n  \"rss_kb\": {:.2},\n  \"rss_mb\": {:.3},\n",
                        rss, rss_kb, rss_mb
                    ));
                    out.push_str(&format!("  \"total_ram_bytes\": {},\n", total_ram));
                    out.push_str(&format!(
                        "  \"pct_registry_over_rss\": {:.2},\n",
                        pct_registry_over_rss
                    ));
                    out.push_str(&format!(
                        "  \"pct_rss_over_totalram\": {:.2}",
                        pct_rss_over_totalram
                    ));

                    // --all: additional process info as JSON fields
                    if all {
                        let fds = fd_summary();
                        out.push_str(",\n  \"fd_summary\": {\n");
                        out.push_str(&format!("    \"total\": {},\n", fds.total));
                        out.push_str(&format!("    \"sockets\": {},\n", fds.sockets));
                        out.push_str(&format!("    \"pipes\": {},\n", fds.pipes));
                        out.push_str(&format!("    \"regular\": {},\n", fds.regular));
                        out.push_str(&format!("    \"chardev\": {},\n", fds.chardev));
                        out.push_str(&format!("    \"dir\": {},\n", fds.dir));
                        out.push_str(&format!("    \"symlink\": {},\n", fds.symlink));
                        out.push_str(&format!("    \"other\": {}\n", fds.other));
                        out.push_str("  }");

                        if let Some(status) = linux_proc_status_json() {
                            out.push_str(",\n  \"proc_status\": ");
                            out.push_str(&status);
                        }
                        if let Some(limits) = linux_proc_limits_json() {
                            out.push_str(",\n  \"proc_limits\": ");
                            out.push_str(&limits);
                        }
                    }
                    out.push_str("\n}\n");
                    return Ok(out);
                }

                // TUI/CLI text output
                let mut rows: Vec<(&str, &str, u64)> = items
                    .iter()
                    .map(|r| (r.id.as_str(), kind_str(&r.kind), r.bytes))
                    .collect();
                rows.sort_by(|a, b| b.2.cmp(&a.2));
                if let Some(n) = top {
                    rows.truncate(n);
                }

                let total = memory::total_bytes();
                let (total_b, total_h) = fmt_bytes(total);

                let rss = crate::memory::process_rss_bytes();
                let (rss_b, rss_h) = fmt_bytes(rss);

                let vms = crate::memory::process_vms_bytes();
                let (vms_b, vms_h) = fmt_bytes(vms);

                let total_ram = crate::memory::total_ram_bytes();
                let pct_registry_over_rss = if rss > 0 {
                    (total as f64) * 100.0 / (rss as f64)
                } else {
                    0.0
                };
                let pct_rss_over_totalram = if total_ram > 0 {
                    (rss as f64) * 100.0 / (total_ram as f64)
                } else {
                    0.0
                };

                let mut out = String::new();
                out.push_str("MEMORY SNAPSHOT\n");
                out.push_str("================\n");
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "ID", "KIND", "BYTES"));
                out.push_str(&format!("{}\n", "-".repeat(36 + 2 + 16 + 2 + 12)));

                for (id, kind, bytes) in rows {
                    out.push_str(&format!(
                        "{:<36}  {:<16}  {:>12}\n",
                        truncate(id, 36),
                        truncate(kind, 16),
                        bytes
                    ));
                }

                out.push_str(&format!("{}\n", "-".repeat(36 + 2 + 16 + 2 + 12)));
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "TOTAL", "", total_b));
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "", "", total_h));

                out.push_str("\nLEGEND\n");
                out.push_str("------\n");
                out.push_str("Registry: explizit registrierte Ressourcen (Assets, Constants, Phasen-Deltas)\n");
                out.push_str(
                    "RSS:      tatsächlicher RAM-Verbrauch des Prozesses (Resident Set Size)\n",
                );
                out.push_str("VMS:      virtueller Adressraum (nicht gleich realer RAM)\n");

                // reuse the existing `items` vec (lives until function end)
                let mut phases: Vec<_> = items
                    .iter()
                    .filter(|r| matches!(r.kind, crate::memory::ResourceKind::Phase))
                    .map(|r| (r.id.as_str(), r.bytes))
                    .collect();

                phases.sort_by(|a, b| b.1.cmp(&a.1));
                if let Some(n) = top {
                    phases.truncate(n);
                }

                if !phases.is_empty() {
                    out.push_str("\nPHASES (ΔRSS)\n");
                    out.push_str("--------------\n");
                    out.push_str(&format!("{:<36}  {:>12}\n", "PHASE ID", "BYTES"));
                    out.push_str(&format!("{}\n", "-".repeat(36 + 2 + 12)));
                    for (id, bytes) in phases {
                        out.push_str(&format!("{:<36}  {:>12}\n", truncate(id, 36), bytes));
                    }
                }

                // RUNTIME
                let thread_count = crate::memory::process_thread_count();
                let thread_text = if thread_count == 0 && cfg!(not(target_os = "linux")) {
                    "n/a".to_string()
                } else {
                    thread_count.to_string()
                };

                out.push_str("\nRUNTIME\n");
                out.push_str("-------\n");
                out.push_str(&format!(
                    "{:<36}  {:<16}  {:>12}\n",
                    "Threads", "", thread_text
                ));

                out.push_str("\nPROCESS MEMORY (RSS)\n");
                out.push_str("--------------------\n");
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "RSS", "", rss_b));
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "", "", rss_h));

                out.push_str("\nVIRTUAL MEMORY (VMS)\n");
                out.push_str("--------------------\n");
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "VMS", "", vms_b));
                out.push_str(&format!("{:<36}  {:<16}  {:>12}\n", "", "", vms_h));

                out.push_str("\nPERCENTAGES\n");
                out.push_str("-----------\n");
                out.push_str(&format!(
                    "{:<36}  {:<16}  {:>11.2}%\n",
                    "Registry TOTAL / RSS", "", pct_registry_over_rss
                ));
                out.push_str(&format!(
                    "{:<36}  {:<16}  {:>11.2}%\n",
                    "RSS / Total RAM", "", pct_rss_over_totalram
                ));

                // --all: deep process info
                if all {
                    let fds = fd_summary();
                    out.push_str("\nOPEN FILE DESCRIPTORS\n");
                    out.push_str("----------------------\n");
                    out.push_str(&format!("{:<24} {:>12}\n", "Total", fds.total));
                    out.push_str(&format!("{:<24} {:>12}\n", "Sockets", fds.sockets));
                    out.push_str(&format!("{:<24} {:>12}\n", "Pipes", fds.pipes));
                    out.push_str(&format!("{:<24} {:>12}\n", "Regular Files", fds.regular));
                    out.push_str(&format!("{:<24} {:>12}\n", "Char Devices", fds.chardev));
                    out.push_str(&format!("{:<24} {:>12}\n", "Directories", fds.dir));
                    out.push_str(&format!("{:<24} {:>12}\n", "Symlinks", fds.symlink));
                    out.push_str(&format!("{:<24} {:>12}\n", "Other", fds.other));

                    if let Some(status_txt) = linux_proc_status_text() {
                        out.push_str("\nPROC STATUS (/proc/self/status)\n");
                        out.push_str("-------------------------------\n");
                        out.push_str(&status_txt);
                    } else {
                        out.push_str("\nPROC STATUS: n/a on this platform\n");
                    }

                    if let Some(limits_txt) = linux_proc_limits_text() {
                        out.push_str("\nPROC LIMITS (/proc/self/limits)\n");
                        out.push_str("-------------------------------\n");
                        out.push_str(&limits_txt);
                    }
                }

                return Ok(out);
            }
        }
    }
}

// --- Parser & Helpers ---

fn parse_from_args(args: &[&str]) -> MemorySubcommand {
    // If args starts with "mem", shift index
    let offset = if args
        .first()
        .map(|s| s.eq_ignore_ascii_case("mem"))
        .unwrap_or(false)
    {
        1
    } else {
        0
    };

    let sub = args
        .get(offset)
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_else(|| "help".into());

    let mut json = false;
    let mut all = false;
    let mut top: Option<usize> = None;

    // Collect flags from offset+1
    let mut i = offset + 1;
    while i < args.len() {
        let a = args[i];
        if a == "--json" {
            json = true;
            i += 1;
            continue;
        }
        if a == "--all" {
            all = true;
            i += 1;
            continue;
        }
        if a.starts_with("--top=") {
            if let Some(n) = a.split_once('=').and_then(|(_, v)| v.parse::<usize>().ok()) {
                top = Some(n);
            }
            i += 1;
            continue;
        }
        if a == "--top" {
            if let Some(n) = args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                top = Some(n);
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        i += 1;
    }

    match sub.as_str() {
        "help" | "" => MemorySubcommand::Help,
        "info" => MemorySubcommand::Info { json, top, all },
        _ => MemorySubcommand::Help,
    }
}

fn kind_str(k: &crate::memory::ResourceKind) -> &'static str {
    use crate::memory::ResourceKind::*;
    match k {
        EmbeddedAsset => "EmbeddedAsset",
        Phase => "Phase",
        _ => "Other",
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}

fn fmt_bytes(b: u64) -> (String, String) {
    let kb = (b as f64) / 1024.0;
    let mb = kb / 1024.0;
    (format!("{b}"), format!("{kb:.2} KB / {mb:.3} MB"))
}

// --- Help ---
fn help_text() -> String {
    let mut s = String::new();
    s.push_str("mem – Memory & Prozess-Introspektion\n");
    s.push_str("===================================\n");
    s.push_str("USAGE:\n");
    s.push_str("  mem help                 Zeigt diese Hilfe\n");
    s.push_str("  mem info [--top N]      Snapshot der Registry + Prozess-RSS/VMS\n");
    s.push_str("  mem info --json         Ausgabe als JSON\n");
    s.push_str("  mem info --all          Erweiterte Prozessinfos (FDs, /proc/status, limits)\n");
    s.push_str("  mem info --json --all   JSON inkl. erweiterter Prozessinfos\n");
    s
}

// --- FD summary with fine-grained categories ---
#[derive(Debug, Clone, Copy)]
struct FdSummary {
    total: usize,
    sockets: usize,
    pipes: usize,
    regular: usize,
    chardev: usize,
    dir: usize,
    symlink: usize,
    other: usize,
}

#[cfg(unix)]
fn fd_summary() -> FdSummary {
    use std::os::unix::fs::MetadataExt;
    use std::{fs, path::Path};

    let fd_dir = if Path::new("/proc/self/fd").exists() {
        "/proc/self/fd"
    } else {
        "/dev/fd" // macOS/*BSD
    };

    let mut s = FdSummary {
        total: 0,
        sockets: 0,
        pipes: 0,
        regular: 0,
        chardev: 0,
        dir: 0,
        symlink: 0,
        other: 0,
    };

    if let Ok(entries) = fs::read_dir(fd_dir) {
        for e in entries.flatten() {
            s.total += 1;
            if let Ok(md) = e.metadata() {
                // md.mode(): u32, libc::S_IF* may be u16 -> cast to u32
                let m = md.mode() & (libc::S_IFMT as u32);

                if m == (libc::S_IFSOCK as u32) {
                    s.sockets += 1;
                } else if m == (libc::S_IFIFO as u32) {
                    s.pipes += 1;
                } else if m == (libc::S_IFREG as u32) {
                    s.regular += 1;
                } else if m == (libc::S_IFCHR as u32) {
                    s.chardev += 1;
                } else if m == (libc::S_IFDIR as u32) {
                    s.dir += 1;
                } else if m == (libc::S_IFLNK as u32) {
                    s.symlink += 1;
                } else {
                    s.other += 1;
                }
            } else {
                s.other += 1;
            }
        }
    }
    s
}

#[cfg(not(unix))]
fn fd_summary() -> FdSummary {
    FdSummary {
        total: 0,
        sockets: 0,
        pipes: 0,
        regular: 0,
        chardev: 0,
        dir: 0,
        symlink: 0,
        other: 0,
    }
}

#[cfg(target_os = "linux")]
fn linux_proc_status_text() -> Option<String> {
    std::fs::read_to_string("/proc/self/status").ok()
}

#[cfg(not(target_os = "linux"))]
fn linux_proc_status_text() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn linux_proc_limits_text() -> Option<String> {
    std::fs::read_to_string("/proc/self/limits").ok()
}

#[cfg(not(target_os = "linux"))]
fn linux_proc_limits_text() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn linux_proc_status_json() -> Option<String> {
    use std::collections::BTreeMap;
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    let mut map = BTreeMap::new();
    for line in s.lines() {
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    Some(serde_json::to_string(&map).unwrap_or_else(|_| "{}".into()))
}

#[cfg(not(target_os = "linux"))]
fn linux_proc_status_json() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn linux_proc_limits_json() -> Option<String> {
    let txt = std::fs::read_to_string("/proc/self/limits").ok()?;
    let mut rows = Vec::<serde_json::Value>::new();
    for line in txt.lines().skip(1) {
        let cols: Vec<_> = line.split_whitespace().collect();
        if cols.len() >= 4 {
            rows.push(serde_json::json!({
                "resource": cols[0],
                "soft": cols[1],
                "hard": cols[2],
                "units": cols[3]
            }));
        }
    }
    Some(serde_json::to_string(&rows).unwrap_or_else(|_| "[]".into()))
}

#[cfg(not(target_os = "linux"))]
fn linux_proc_limits_json() -> Option<String> {
    None
}
