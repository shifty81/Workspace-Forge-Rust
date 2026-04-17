//! Project manifest model and file I/O for NovaForge Workspace.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// File name written at the root of a NovaForge project directory.
pub const MANIFEST_FILE: &str = "novaforge.workspace.toml";

/// Contents of `novaforge.workspace.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    /// Human-readable project name shown in the launcher and editor title bar.
    pub project_name: String,

    /// Absolute or relative path to the Nova-Forge repository root.
    #[serde(default = "default_nova_forge_path")]
    pub nova_forge_path: PathBuf,

    /// Absolute or relative path to the asset root directory.
    #[serde(default = "default_asset_root")]
    pub asset_root: PathBuf,

    /// Last active scene file, relative to `asset_root`.
    pub active_scene: Option<String>,

    /// Recently opened scene files (most recent first).
    #[serde(default)]
    pub recent_scenes: Vec<String>,
}

fn default_nova_forge_path() -> PathBuf {
    PathBuf::from("../Nova-Forge")
}

fn default_asset_root() -> PathBuf {
    PathBuf::from("../Nova-Forge/assets")
}

impl Default for WorkspaceManifest {
    fn default() -> Self {
        Self {
            project_name: "My NovaForge Project".to_string(),
            nova_forge_path: default_nova_forge_path(),
            asset_root: default_asset_root(),
            active_scene: None,
            recent_scenes: Vec::new(),
        }
    }
}

/// Errors that can occur when loading or saving a [`WorkspaceManifest`].
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialisation error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}

impl WorkspaceManifest {
    /// Load a manifest from `path`.
    ///
    /// `path` may point directly to `novaforge.workspace.toml` or to the
    /// project directory that contains it.
    pub fn load(path: &Path) -> Result<Self, ProjectError> {
        let manifest_path = if path.is_dir() {
            path.join(MANIFEST_FILE)
        } else {
            path.to_path_buf()
        };
        let content = std::fs::read_to_string(&manifest_path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Serialise and write the manifest to `path`.
    pub fn save(&self, path: &Path) -> Result<(), ProjectError> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Resolve the path to the Nova-Forge client binary.
    pub fn nova_forge_binary(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        let bin = "nova-forge-voxygen.exe";
        #[cfg(not(target_os = "windows"))]
        let bin = "nova-forge-voxygen";

        self.nova_forge_path.join("target/release").join(bin)
    }

    /// Resolve the path to the `nova-forge.sh` build script.
    pub fn build_script(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        let script = "nova-forge.bat";
        #[cfg(not(target_os = "windows"))]
        let script = "nova-forge.sh";

        self.nova_forge_path.join(script)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_manifest_round_trips() {
        let manifest = WorkspaceManifest::default();
        let toml_str = toml::to_string_pretty(&manifest).expect("serialise");
        let parsed: WorkspaceManifest = toml::from_str(&toml_str).expect("parse");
        assert_eq!(manifest.project_name, parsed.project_name);
    }

    #[test]
    fn nova_forge_binary_path_is_non_empty() {
        let m = WorkspaceManifest::default();
        assert!(!m.nova_forge_binary().as_os_str().is_empty());
    }
}
