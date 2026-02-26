use crate::memory::{register_embedded, ResourceKind};
use include_dir::{include_dir, Dir, File};

pub static SRC_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src");

pub fn register_all_src() {
    register_dir_recursive(&SRC_DIR, "src");
}

pub fn register_all_src_filtered() {
    register_dir_recursive_filtered(
        &SRC_DIR,
        "src",
        &["html", "css", "js", "svg", "txt", "json", "otf", "ttf"],
    );
}

/// Rekursiv, Pfad relativ zum `SRC_DIR` sauber zusammenbauen.
pub fn register_dir_recursive(dir: &'static Dir, base_id: &str) {
    recurse(dir, base_id, "");
}

pub fn register_dir_recursive_filtered(dir: &'static Dir, base_id: &str, exts: &[&str]) {
    recurse_filtered(dir, base_id, "", exts);
}

fn recurse(dir: &'static Dir, base_id: &str, prefix: &str) {
    for f in dir.files() {
        let rel = join_rel(prefix, f.path());
        register_file(&rel, f, base_id);
    }
    for d in dir.dirs() {
        let new_prefix = join_rel(prefix, d.path());
        // `d.path()` kann „server/handlers“ enthalten – wir hängen nur das letzte Segment an
        let new_prefix = trim_to_last_segment(&new_prefix);
        recurse(d, base_id, &new_prefix);
    }
}

fn recurse_filtered(dir: &'static Dir, base_id: &str, prefix: &str, exts: &[&str]) {
    for f in dir.files() {
        let rel = join_rel(prefix, f.path());
        if has_ext(&rel, exts) {
            register_file(&rel, f, base_id);
        }
    }
    for d in dir.dirs() {
        let new_prefix = join_rel(prefix, d.path());
        let new_prefix = trim_to_last_segment(&new_prefix);
        recurse_filtered(d, base_id, &new_prefix, exts);
    }
}

fn register_file(rel: &str, file: &File, base_id: &str) {
    let rel = rel.trim_start_matches("src/"); // optional: führendes src/ entfernen
    let id = format!("{base_id}:{rel}@v1");
    let bytes = file.contents().len() as u64;
    register_embedded(&id.replace('\\', "/"), ResourceKind::EmbeddedAsset, bytes);
}

fn join_rel(prefix: &str, path: &std::path::Path) -> String {
    if prefix.is_empty() {
        path.to_string_lossy().into_owned()
    } else {
        format!("{prefix}/{}", path.display())
    }
}

/// aus "server/handlers" -> "handlers" (nur letztes Segment),
/// damit sich der Pfad beim Abstieg nicht doppelt aufbläht.
fn trim_to_last_segment(p: &str) -> String {
    std::path::Path::new(p)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| p.to_string())
}

fn has_ext(rel: &str, allow: &[&str]) -> bool {
    let rel = rel.to_ascii_lowercase();
    allow.iter().any(|e| rel.ends_with(&format!(".{e}")))
}
