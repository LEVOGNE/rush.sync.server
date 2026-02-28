use crate::core::prelude::*;
use crate::sync::profiles::RemoteProfile;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const DEFAULT_EXCLUDES: &[&str] = &[".git", ".rss", "target", ".DS_Store"];

#[derive(Debug)]
struct ProcessResult {
    stdout: String,
    stderr: String,
    status_code: i32,
}

pub fn test_connection(profile: &RemoteProfile) -> Result<String> {
    ensure_tool_available("ssh", "-V")?;

    let mut args = ssh_base_args(profile);
    args.push(profile.ssh_target());
    args.push("echo rush-sync-remote-ok".to_string());

    let output = run_process("ssh", &args, false)?;
    let merged = format!("{}\n{}", output.stdout, output.stderr);

    if merged.contains("rush-sync-remote-ok") {
        Ok(format!(
            "Remote '{}' reachable via SSH ({}:{})",
            profile.ssh_target(),
            profile.host,
            profile.port
        ))
    } else {
        Err(AppError::Validation(format!(
            "SSH connection failed for {}:{}.\nOutput: {}",
            profile.host,
            profile.port,
            if merged.trim().is_empty() {
                "no output".to_string()
            } else {
                merged.trim().to_string()
            }
        )))
    }
}

pub fn run_remote_command(profile: &RemoteProfile, command: &str) -> Result<String> {
    ensure_tool_available("ssh", "-V")?;

    let mut args = ssh_base_args(profile);
    args.push(profile.ssh_target());
    args.push(command.to_string());

    let output = run_process("ssh", &args, false)?;
    Ok(format_process_output("Remote command executed", &output))
}

pub fn sync_push(
    profile: &RemoteProfile,
    local_path: &Path,
    delete: bool,
    dry_run: bool,
) -> Result<String> {
    if !local_path.exists() {
        return Err(AppError::Validation(format!(
            "Local path '{}' does not exist",
            local_path.display()
        )));
    }

    ensure_remote_directory(profile)?;

    if tool_available("rsync", "--version") {
        sync_push_rsync(profile, local_path, delete, dry_run)
    } else {
        if dry_run {
            return Err(AppError::Validation(
                "Dry-run is only supported with rsync (rsync not found in PATH)".to_string(),
            ));
        }
        sync_push_scp(profile, local_path)
    }
}

pub fn sync_pull(
    profile: &RemoteProfile,
    local_path: &Path,
    delete: bool,
    dry_run: bool,
) -> Result<String> {
    ensure_local_directory(local_path)?;

    if tool_available("rsync", "--version") {
        sync_pull_rsync(profile, local_path, delete, dry_run)
    } else {
        if dry_run {
            return Err(AppError::Validation(
                "Dry-run is only supported with rsync (rsync not found in PATH)".to_string(),
            ));
        }
        sync_pull_scp(profile, local_path)
    }
}

pub fn restart_service(profile: &RemoteProfile, service_name: &str) -> Result<String> {
    let name = service_name.trim();
    if name.is_empty() {
        return Err(AppError::Validation(
            "Service name cannot be empty".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@')
    {
        return Err(AppError::Validation(
            "Service name contains invalid characters (allowed: a-z, 0-9, - _ . @)".to_string(),
        ));
    }

    let cmd = format!(
        "sudo systemctl restart {} && sudo systemctl status {} --no-pager --lines=5",
        shell_quote(service_name),
        shell_quote(service_name)
    );

    run_remote_command(profile, &cmd)
}

pub fn git_pull(profile: &RemoteProfile, branch: Option<&str>) -> Result<String> {
    let branch = branch.unwrap_or("main");
    if !branch
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '/' || c == '.')
    {
        return Err(AppError::Validation(
            "Branch name contains invalid characters".to_string(),
        ));
    }
    let cmd = format!(
        "cd {} && git fetch --all && git pull --ff-only origin {}",
        shell_quote(&profile.remote_path),
        shell_quote(branch)
    );
    run_remote_command(profile, &cmd)
}

fn sync_push_rsync(
    profile: &RemoteProfile,
    local_path: &Path,
    delete: bool,
    dry_run: bool,
) -> Result<String> {
    ensure_tool_available("rsync", "--version")?;

    let mut args = Vec::<String>::new();
    args.push("-az".to_string());
    args.push("--human-readable".to_string());

    for exclude in DEFAULT_EXCLUDES {
        args.push("--exclude".to_string());
        args.push((*exclude).to_string());
    }

    if delete {
        args.push("--delete".to_string());
    }

    if dry_run {
        args.push("--dry-run".to_string());
    }

    args.push("-e".to_string());
    args.push(rsync_ssh_transport(profile));
    args.push(rsync_source_arg(local_path));
    args.push(format!(
        "{}:{}/",
        profile.ssh_target(),
        escape_remote_path(&profile.remote_path)
    ));

    let output = run_process("rsync", &args, false)?;
    Ok(format!(
        "{}\n{}",
        if dry_run {
            "Sync push dry-run via rsync"
        } else {
            "Sync push completed via rsync"
        },
        format_process_output("rsync", &output)
    ))
}

fn sync_pull_rsync(
    profile: &RemoteProfile,
    local_path: &Path,
    delete: bool,
    dry_run: bool,
) -> Result<String> {
    ensure_tool_available("rsync", "--version")?;

    let mut args = Vec::<String>::new();
    args.push("-az".to_string());
    args.push("--human-readable".to_string());

    if delete {
        args.push("--delete".to_string());
    }

    if dry_run {
        args.push("--dry-run".to_string());
    }

    args.push("-e".to_string());
    args.push(rsync_ssh_transport(profile));
    args.push(format!(
        "{}:{}/",
        profile.ssh_target(),
        escape_remote_path(&profile.remote_path)
    ));
    args.push(rsync_source_arg(local_path));

    let output = run_process("rsync", &args, false)?;
    Ok(format!(
        "{}\n{}",
        if dry_run {
            "Sync pull dry-run via rsync"
        } else {
            "Sync pull completed via rsync"
        },
        format_process_output("rsync", &output)
    ))
}

fn sync_push_scp(profile: &RemoteProfile, local_path: &Path) -> Result<String> {
    ensure_tool_available("scp", "-V")?;

    let mut args = scp_base_args(profile);
    if local_path.is_dir() {
        args.push("-r".to_string());
    }
    args.push(scp_source_arg(local_path));
    args.push(format!(
        "{}:{}",
        profile.ssh_target(),
        escape_remote_path(&profile.remote_path)
    ));

    let output = run_process("scp", &args, false)?;
    Ok(format!(
        "{}\n{}",
        "Sync push completed via scp fallback",
        format_process_output("scp", &output)
    ))
}

fn sync_pull_scp(profile: &RemoteProfile, local_path: &Path) -> Result<String> {
    ensure_tool_available("scp", "-V")?;

    let mut args = scp_base_args(profile);
    args.push("-r".to_string());
    args.push(format!(
        "{}:{}/.",
        profile.ssh_target(),
        escape_remote_path(&profile.remote_path)
    ));
    args.push(local_path.display().to_string());

    let output = run_process("scp", &args, false)?;
    Ok(format!(
        "{}\n{}",
        "Sync pull completed via scp fallback",
        format_process_output("scp", &output)
    ))
}

fn ensure_remote_directory(profile: &RemoteProfile) -> Result<()> {
    let cmd = format!("mkdir -p {}", shell_quote(&profile.remote_path));
    let _ = run_remote_command(profile, &cmd)?;
    Ok(())
}

fn ensure_local_directory(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(AppError::Io)
}

fn base_connection_args(profile: &RemoteProfile, port_flag: &str) -> Vec<String> {
    let mut args = Vec::new();
    args.push(port_flag.to_string());
    args.push(profile.port.to_string());

    if let Some(identity) = expanded_identity(profile) {
        args.push("-i".to_string());
        args.push(identity.display().to_string());
    }

    args.push("-o".to_string());
    args.push("BatchMode=yes".to_string());
    args.push("-o".to_string());
    args.push("ConnectTimeout=30".to_string());
    args
}

fn ssh_base_args(profile: &RemoteProfile) -> Vec<String> {
    base_connection_args(profile, "-p")
}

fn scp_base_args(profile: &RemoteProfile) -> Vec<String> {
    base_connection_args(profile, "-P")
}

fn rsync_ssh_transport(profile: &RemoteProfile) -> String {
    let mut transport = format!("ssh -p {} -o BatchMode=yes -o ConnectTimeout=30", profile.port);
    if let Some(identity) = expanded_identity(profile) {
        transport.push_str(&format!(
            " -i {}",
            shell_quote(&identity.display().to_string())
        ));
    }
    transport
}

fn expanded_identity(profile: &RemoteProfile) -> Option<PathBuf> {
    profile
        .identity_file
        .as_ref()
        .map(|path| expand_tilde(path.trim()))
}

fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
        return PathBuf::from(path);
    }

    if let Some(suffix) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(suffix);
        }
    }

    PathBuf::from(path)
}

fn rsync_source_arg(path: &Path) -> String {
    if path.is_dir() {
        format!("{}/", path.display())
    } else {
        path.display().to_string()
    }
}

fn scp_source_arg(path: &Path) -> String {
    if path.is_dir() {
        format!("{}/.", path.display())
    } else {
        path.display().to_string()
    }
}

fn escape_remote_path(path: &str) -> String {
    path.replace(' ', "\\ ")
}

fn tool_available(tool: &str, version_arg: &str) -> bool {
    Command::new(tool)
        .arg(version_arg)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn ensure_tool_available(tool: &str, version_arg: &str) -> Result<()> {
    if tool_available(tool, version_arg) {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "Required tool '{}' was not found in PATH",
            tool
        )))
    }
}

fn run_process(binary: &str, args: &[String], allow_non_zero: bool) -> Result<ProcessResult> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                AppError::Validation(format!("Command '{}' not found. Install it first.", binary))
            }
            _ => AppError::Io(e),
        })?;

    let result = ProcessResult {
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        status_code: output.status.code().unwrap_or(-1),
    };

    if !allow_non_zero && !output.status.success() {
        return Err(AppError::Validation(format!(
            "Command '{}' failed with exit code {}: {}",
            binary,
            result.status_code,
            if result.stderr.is_empty() {
                "No error output".to_string()
            } else {
                result.stderr.clone()
            }
        )));
    }

    Ok(result)
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{}'", escaped)
}

fn format_process_output(prefix: &str, result: &ProcessResult) -> String {
    let mut output = format!("{} (exit={})", prefix, result.status_code);
    if !result.stdout.is_empty() {
        output.push_str(&format!("\nstdout:\n{}", result.stdout));
    }
    if !result.stderr.is_empty() {
        output.push_str(&format!("\nstderr:\n{}", result.stderr));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_simple() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn shell_quote_with_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\"'\"'s'");
    }

    #[test]
    fn shell_quote_empty() {
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn expand_tilde_home() {
        let result = expand_tilde("~/test");
        assert!(result.to_str().unwrap().ends_with("/test"));
        assert!(!result.to_str().unwrap().starts_with("~"));
    }

    #[test]
    fn expand_tilde_no_tilde() {
        assert_eq!(expand_tilde("/absolute/path"), PathBuf::from("/absolute/path"));
    }

    #[test]
    fn escape_remote_path_spaces() {
        assert_eq!(escape_remote_path("/my path/dir"), "/my\\ path/dir");
    }

    #[test]
    fn escape_remote_path_no_spaces() {
        assert_eq!(escape_remote_path("/opt/app"), "/opt/app");
    }

    #[test]
    fn service_name_rejects_injection() {
        let profile =
            RemoteProfile::new("u".into(), "h".into(), "/opt".into(), 22, None).unwrap();
        let res = restart_service(&profile, "foo;rm -rf /");
        assert!(res.is_err());
    }

    #[test]
    fn git_branch_rejects_injection() {
        let profile =
            RemoteProfile::new("u".into(), "h".into(), "/opt".into(), 22, None).unwrap();
        let res = git_pull(&profile, Some("main;rm -rf /"));
        assert!(res.is_err());
    }

    #[test]
    fn rsync_source_arg_dir_trailing_slash() {
        let tmp = std::env::temp_dir();
        assert!(rsync_source_arg(&tmp).ends_with('/'));
    }
}
