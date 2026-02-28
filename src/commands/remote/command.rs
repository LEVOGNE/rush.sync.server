use crate::commands::command::Command;
use crate::core::prelude::*;
use crate::sync::profiles::{parse_user_host, RemoteProfile, RemoteProfileStore};
use crate::sync::transport::test_connection;

#[derive(Debug, Default)]
pub struct RemoteCommand;

impl RemoteCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for RemoteCommand {
    fn name(&self) -> &'static str {
        "remote"
    }

    fn description(&self) -> &'static str {
        "Manage SSH remote profiles for sync/deploy"
    }

    fn matches(&self, command: &str) -> bool {
        command.trim().to_lowercase().starts_with("remote")
    }

    fn execute_sync(&self, args: &[&str]) -> Result<String> {
        let store = RemoteProfileStore::new()?;

        match args.first().copied() {
            None | Some("-h" | "--help" | "help") => Ok(self.help_text(store.path())),
            Some("list" | "ls") => self.list_profiles(&store),
            Some("add") => self.add_profile(&store, args),
            Some("show") => self.show_profile(&store, args),
            Some("remove" | "rm" | "delete") => self.remove_profile(&store, args),
            Some("test") => self.test_profile(&store, args),
            Some(sub) => Err(AppError::Validation(format!(
                "Unknown remote subcommand '{}'. Use 'remote help'.",
                sub
            ))),
        }
    }

    fn priority(&self) -> u8 {
        72
    }
}

impl RemoteCommand {
    fn add_profile(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        if args.len() < 4 {
            return Err(AppError::Validation(
                "Usage: remote add <name> <user@host> <remote_path> [port] [identity_file]"
                    .to_string(),
            ));
        }

        let name = args[1];
        let (user, host) = parse_user_host(args[2])?;
        let remote_path = args[3].to_string();

        let (port, identity_file) = match args.get(4) {
            None => (22, None),
            Some(port_or_identity) => {
                if let Ok(port) = port_or_identity.parse::<u16>() {
                    let identity = args.get(5).map(|s| (*s).to_string());
                    (port, identity)
                } else {
                    (22, Some((*port_or_identity).to_string()))
                }
            }
        };

        let profile = RemoteProfile::new(user, host, remote_path, port, identity_file)?;
        let existed = store.exists(name)?;
        store.upsert(name, profile)?;

        Ok(if existed {
            format!("Remote '{}' updated", name)
        } else {
            format!("Remote '{}' added", name)
        })
    }

    fn list_profiles(&self, store: &RemoteProfileStore) -> Result<String> {
        let profiles = store.list()?;
        if profiles.is_empty() {
            return Ok(format!(
                "No remotes configured yet.\nFile: {}",
                store.path().display()
            ));
        }

        let mut out = String::from("Configured remotes:\n");
        for (name, profile) in profiles {
            let identity = profile.identity_file.as_deref().unwrap_or("-");

            out.push_str(&format!(
                "  {} -> {}@{}:{} {}\n",
                name, profile.user, profile.host, profile.port, profile.remote_path
            ));
            out.push_str(&format!("     identity: {}\n", identity));
        }
        out.push_str(&format!("\nFile: {}", store.path().display()));
        Ok(out)
    }

    fn show_profile(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let name = args
            .get(1)
            .ok_or_else(|| AppError::Validation("Usage: remote show <name>".to_string()))?;

        let profile = store.get(name)?;
        Ok(format!(
            "Remote '{}'\n  user: {}\n  host: {}\n  port: {}\n  remote_path: {}\n  identity_file: {}",
            name,
            profile.user,
            profile.host,
            profile.port,
            profile.remote_path,
            profile.identity_file.as_deref().unwrap_or("-")
        ))
    }

    fn remove_profile(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let name = args
            .get(1)
            .ok_or_else(|| AppError::Validation("Usage: remote remove <name>".to_string()))?;

        store.remove(name)?;
        Ok(format!("Remote '{}' removed", name))
    }

    fn test_profile(&self, store: &RemoteProfileStore, args: &[&str]) -> Result<String> {
        let name = args
            .get(1)
            .ok_or_else(|| AppError::Validation("Usage: remote test <name>".to_string()))?;

        let profile = store.get(name)?;
        let message = test_connection(&profile)?;
        Ok(format!("{} [{}]", message, name))
    }

    fn help_text(&self, file_path: &std::path::Path) -> String {
        format!(
            "Remote profile management\n\n\
             Commands:\n\
               remote list\n\
               remote add <name> <user@host> <remote_path> [port] [identity_file]\n\
               remote show <name>\n\
               remote remove <name>\n\
               remote test <name>\n\n\
             Example:\n\
               remote add prod deploy@example.com /opt/rush-sync 22 ~/.ssh/id_ed25519\n\n\
             Storage:\n\
               {}",
            file_path.display()
        )
    }
}
