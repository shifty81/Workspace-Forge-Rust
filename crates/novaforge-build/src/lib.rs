//! Build pipeline integration for NovaForge Workspace.
//!
//! [`BuildRunner`] spawns `nova-forge.sh` (or `nova-forge.bat` on Windows) as
//! a child process and streams its stdout/stderr back through an
//! [`std::sync::mpsc`] channel so editor panels can display live log output.

use novaforge_project::WorkspaceManifest;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

/// A build command that maps to a `nova-forge.sh` sub-command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildCommand {
    /// `nova-forge.sh build` — debug build.
    Build,
    /// `nova-forge.sh release` — optimised release build.
    Release,
    /// `nova-forge.sh clean` — remove build artefacts.
    Clean,
    /// `nova-forge.sh run` — build and launch the game client.
    Run,
    /// `nova-forge.sh server` — build and launch a dedicated server.
    RunServer,
    /// `nova-forge.sh test` — run the Nova-Forge test suite.
    Test,
}

impl BuildCommand {
    /// The sub-command argument passed to `nova-forge.sh`.
    pub fn script_arg(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Release => "release",
            Self::Clean => "clean",
            Self::Run => "run",
            Self::RunServer => "server",
            Self::Test => "test",
        }
    }

    /// Human-readable label used in the Build Tool UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Build => "Build (debug)",
            Self::Release => "Build (release)",
            Self::Clean => "Clean",
            Self::Run => "Run",
            Self::RunServer => "Run Server",
            Self::Test => "Test",
        }
    }
}

/// Spawns `nova-forge.sh` commands and streams their output.
pub struct BuildRunner {
    /// Path to the Nova-Forge repository root.
    pub nova_forge_path: PathBuf,
}

impl BuildRunner {
    /// Create a new runner pointing at the given Nova-Forge repository root.
    pub fn new(nova_forge_path: PathBuf) -> Self {
        Self { nova_forge_path }
    }

    /// Create a runner from a [`WorkspaceManifest`].
    pub fn from_manifest(manifest: &WorkspaceManifest) -> Self {
        Self::new(manifest.nova_forge_path.clone())
    }

    /// Spawn `cmd` in a background thread.
    ///
    /// Returns a [`Receiver<String>`] that yields log lines as they arrive.
    /// The channel is closed when the child process exits.
    pub fn spawn(&self, cmd: BuildCommand) -> Receiver<String> {
        let (tx, rx) = mpsc::channel::<String>();

        let path = self.nova_forge_path.clone();
        let script = manifest_build_script(&path);
        let arg = cmd.script_arg();

        thread::spawn(move || {
            let _ = tx.send(format!(
                "[workspace] Running: {} {}",
                script.display(),
                arg
            ));

            let result = Command::new(&script)
                .arg(arg)
                .current_dir(&path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Err(e) => {
                    let _ = tx.send(format!("[error] Failed to start build: {e}"));
                }
                Ok(mut child) => {
                    // Stream stdout
                    if let Some(stdout) = child.stdout.take() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            if tx.send(line).is_err() {
                                // Receiver dropped — abort.
                                let _ = child.kill();
                                return;
                            }
                        }
                    }
                    match child.wait() {
                        Ok(status) => {
                            let _ = tx.send(format!("[done] Exit status: {status}"));
                        }
                        Err(e) => {
                            let _ = tx.send(format!("[error] {e}"));
                        }
                    }
                }
            }
        });

        rx
    }
}

fn manifest_build_script(nova_forge_path: &std::path::Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    return nova_forge_path.join("nova-forge.bat");
    #[cfg(not(target_os = "windows"))]
    return nova_forge_path.join("nova-forge.sh");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_command_labels_are_non_empty() {
        for cmd in [
            BuildCommand::Build,
            BuildCommand::Release,
            BuildCommand::Clean,
            BuildCommand::Run,
            BuildCommand::RunServer,
            BuildCommand::Test,
        ] {
            assert!(!cmd.label().is_empty());
            assert!(!cmd.script_arg().is_empty());
        }
    }
}
