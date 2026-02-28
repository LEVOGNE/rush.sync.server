use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::sync::profiles::RemoteProfileStore;
use crate::sync::transport::{
    git_pull, restart_service, run_remote_command, sync_pull, sync_push, test_connection,
};
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct SyncCommand;

impl SyncCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for SyncCommand {
    fn name(&self) -> &'static str {
        "sync"
    }

    fn description(&self) -> &'static str {
        "Sync files and run remote deployment actions"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("sync")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let store = RemoteProfileStore::new()?;

        match args.first().copied() {
            None | Some("-h" | "--help" | "help") => Ok(self.help_text()),
            Some("push") => self.push(&store, args),
            Some("pull") => self.pull(&store, args),
            Some("test") => self.test(&store, args),
            Some("exec") => self.exec(&store, args),
            Some("restart") => self.restart(&store, args),
            Some("git-pull") => self.git_pull(&store, args),
            Some(sub) => Err(AppError::Validation(format!(
                "Unknown sync subcommand '{}'. Use 'sync help'.",
                sub
            ))),
        }
    }

    fn priority(&self) -> u8 {
        73
    }
}

impl SyncCommand {
    fn push(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let (positionals, delete, dry_run) = parse_flags(args);
        let remote_name = positionals.get(1).ok_or_else(|| {
            AppError::Validation(
                "Usage: sync push <remote> [local_path] [--delete] [--dry-run]".to_string(),
            )
        })?;

        let profile = store.get(remote_name)?;
        let local_path = positionals
            .get(2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("www"));

        let result = sync_push(&profile, &local_path, delete, dry_run)?;
        Ok(format!(
            "{}PUSH {} -> {} [{}]\n{}",
            if dry_run { "[DRY-RUN] " } else { "" },
            local_path.display(),
            profile.remote_path,
            remote_name,
            result
        ))
    }

    fn pull(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let (positionals, delete, dry_run) = parse_flags(args);
        let remote_name = positionals.get(1).ok_or_else(|| {
            AppError::Validation(
                "Usage: sync pull <remote> [local_path] [--delete] [--dry-run]".to_string(),
            )
        })?;

        let profile = store.get(remote_name)?;
        let local_path = positionals
            .get(2)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(format!("download-{}", remote_name)));

        let result = sync_pull(&profile, &local_path, delete, dry_run)?;
        Ok(format!(
            "{}PULL {} <- {} [{}]\n{}",
            if dry_run { "[DRY-RUN] " } else { "" },
            local_path.display(),
            profile.remote_path,
            remote_name,
            result
        ))
    }

    fn test(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let remote_name = args
            .get(1)
            .ok_or_else(|| AppError::Validation("Usage: sync test <remote>".to_string()))?;

        let profile = store.get(remote_name)?;
        test_connection(&profile)
    }

    fn exec(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        if args.len() < 3 {
            return Err(AppError::Validation(
                "Usage: sync exec <remote> <command...>".to_string(),
            ));
        }

        let remote_name = args[1];
        let profile = store.get(remote_name)?;
        let command = args[2..].join(" ");
        run_remote_command(&profile, &command)
    }

    fn restart(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let remote_name = args.get(1).ok_or_else(|| {
            AppError::Validation("Usage: sync restart <remote> [service]".to_string())
        })?;

        let service = args.get(2).copied().unwrap_or("rush-sync");
        let profile = store.get(remote_name)?;

        let result = restart_service(&profile, service)?;
        Ok(format!(
            "Service '{}' restarted on '{}'\n{}",
            service, remote_name, result
        ))
    }

    fn git_pull(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let remote_name = args.get(1).ok_or_else(|| {
            AppError::Validation("Usage: sync git-pull <remote> [branch]".to_string())
        })?;

        let branch = args.get(2).copied();
        let profile = store.get(remote_name)?;
        let result = git_pull(&profile, branch)?;
        Ok(format!(
            "Remote git pull on '{}' (branch: {})\n{}",
            remote_name,
            branch.unwrap_or("main"),
            result
        ))
    }

    fn help_text(&self) -> String {
        "Sync and remote actions\n\n\
         Commands:\n\
           sync push <remote> [local_path] [--delete] [--dry-run]\n\
           sync pull <remote> [local_path] [--delete] [--dry-run]\n\
           sync test <remote>\n\
           sync exec <remote> <command...>\n\
           sync restart <remote> [service]\n\
           sync git-pull <remote> [branch]\n\n\
         Flags:\n\
           --delete    Remove files on destination not present in source\n\
           --dry-run   Show what would be transferred without making changes (rsync only)\n\n\
         Notes:\n\
           - Uses rsync over SSH when available.\n\
           - Falls back to scp when rsync is not installed.\n\
           - Configure remotes via the 'remote' command."
            .to_string()
    }
}

fn parse_flags(args: &[&str]) -> (Vec<String>, bool, bool) {
    let mut delete = false;
    let mut dry_run = false;
    let mut positionals = Vec::new();

    for arg in args {
        match *arg {
            "--delete" => delete = true,
            "--dry-run" | "-n" => dry_run = true,
            _ => positionals.push((*arg).to_string()),
        }
    }

    (positionals, delete, dry_run)
}
