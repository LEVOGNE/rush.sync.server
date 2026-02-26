use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::SystemTime;

use sysinfo; // 0.30+

// ---------------- Prozess-Systemhandle (einmalig, wiederverwendbar) ----------------

static SYS: OnceLock<Mutex<sysinfo::System>> = OnceLock::new();

fn sys_handle() -> &'static Mutex<sysinfo::System> {
    SYS.get_or_init(|| Mutex::new(sysinfo::System::new()))
}

// ---------------- Registry ----------------

#[derive(Clone, Copy, Debug)]
pub enum ResourceKind {
    EmbeddedAsset,
    Phase, // ΔRSS je Start-/Laufzeitphase
    Other,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub id: String,
    pub kind: ResourceKind,
    pub bytes: u64,
    pub created_at: SystemTime,
}

static REGISTRY: OnceLock<RwLock<HashMap<String, Resource>>> = OnceLock::new();

fn reg() -> &'static RwLock<HashMap<String, Resource>> {
    REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn register_embedded(id: &str, kind: ResourceKind, bytes: u64) {
    let res = Resource {
        id: id.to_string(),
        kind,
        bytes,
        created_at: SystemTime::now(),
    };
    let mut map = reg().write().expect("memory registry poisoned");
    map.insert(res.id.clone(), res);
}

pub fn snapshot() -> Vec<Resource> {
    reg()
        .read()
        .expect("memory registry poisoned")
        .values()
        .cloned()
        .collect()
}

pub fn total_bytes() -> u64 {
    reg()
        .read()
        .expect("memory registry poisoned")
        .values()
        .map(|r| r.bytes)
        .sum()
}

// ---------------- Prozess-Metriken ----------------

// RSS in BYTES (sysinfo 0.30+ liefert Bytes)
pub fn process_rss_bytes() -> u64 {
    let mut sys = sys_handle().lock().expect("sysinfo mutex poisoned");
    sys.refresh_processes();
    if let Ok(pid) = sysinfo::get_current_pid() {
        if let Some(p) = sys.process(pid) {
            p.memory() as u64
        } else {
            0
        }
    } else {
        0
    }
}

pub fn process_vms_bytes() -> u64 {
    let mut sys = sys_handle().lock().expect("sysinfo mutex poisoned");
    sys.refresh_processes();
    if let Ok(pid) = sysinfo::get_current_pid() {
        if let Some(p) = sys.process(pid) {
            p.virtual_memory() as u64
        } else {
            0
        }
    } else {
        0
    }
}

pub fn total_ram_bytes() -> u64 {
    let mut sys = sys_handle().lock().expect("sysinfo mutex poisoned");
    sys.refresh_memory();
    sys.total_memory() as u64
}

/// Anzahl Threads (falls nicht verfügbar: 0)
#[cfg(target_os = "linux")]
pub fn process_thread_count() -> usize {
    // Best-effort: aus /proc/self/status die "Threads:"-Zeile parsen
    // (keine extra Dependencies)
    use std::fs;
    if let Ok(s) = fs::read_to_string("/proc/self/status") {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("Threads:") {
                return rest.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }
    0
}

#[cfg(not(target_os = "linux"))]
pub fn process_thread_count() -> usize {
    // sysinfo 0.30 bietet hier plattformübergreifend keine Threads-API.
    // Fallback: 0 (oder du gibst in der Anzeige "n/a" aus).
    0
}

// ---------------- Scopes / Phasen-Messung ----------------

pub struct ScopeGuard {
    id: String,
    start_rss: u64,
}

pub fn begin_scope(id: &str) -> ScopeGuard {
    ScopeGuard {
        id: id.to_string(),
        start_rss: process_rss_bytes(),
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let end = process_rss_bytes();
        let delta = end.saturating_sub(self.start_rss);
        register_embedded(&self.id, ResourceKind::Phase, delta);
    }
}

// ---------------- Debug ----------------

pub fn debug_dump_to_log() {
    #[cfg(debug_assertions)]
    {
        let map = reg().read().expect("memory registry poisoned");
        let total: u64 = map.values().map(|r| r.bytes).sum();
        log::debug!("[memory] resources: {}", map.len());
        log::debug!("[memory] total bytes (registered): {}", total);
        for r in map.values() {
            log::debug!("  - {} ({:?}) {} bytes", r.id, r.kind, r.bytes);
        }
    }
}
