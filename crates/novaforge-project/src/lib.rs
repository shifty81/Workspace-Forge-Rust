//! Project manifest model and file I/O for NovaForge Workspace.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Asset scanning
// ---------------------------------------------------------------------------

/// Kind of a discovered asset file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Texture,
    Model,
    Sound,
    Scene,
    Other,
}

impl AssetKind {
    /// Infer the kind from a file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" | "jpg" | "jpeg" | "webp" | "tga" | "bmp" => Self::Texture,
            "vox" | "obj" | "gltf" | "glb" | "fbx" | "mesh" => Self::Model,
            "ogg" | "wav" | "mp3" | "flac" => Self::Sound,
            "ron" | "scene" => Self::Scene,
            _ => Self::Other,
        }
    }

    /// Icon glyph for display in UI.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Texture => "🖼",
            Self::Model => "📦",
            Self::Sound => "🔊",
            Self::Scene => "🌐",
            Self::Other => "📄",
        }
    }
}

/// A discovered asset file under the asset root.
#[derive(Debug, Clone)]
pub struct AssetEntry {
    /// Display path relative to the asset root.
    pub relative_path: String,
    /// Inferred asset kind.
    pub kind: AssetKind,
}

/// Recursively scan `root` for asset files up to `max_depth` directory levels.
///
/// Returns entries sorted: directories are listed before files, then both
/// groups are sorted by name.
///
/// # Example
/// ```rust
/// use novaforge_project::scan_assets;
/// use std::path::Path;
/// let entries = scan_assets(Path::new("."), 2);
/// // entries is a Vec<AssetEntry>
/// ```
pub fn scan_assets(root: &Path, max_depth: usize) -> Vec<AssetEntry> {
    let mut out = Vec::new();
    scan_dir_inner(root, root, 0, max_depth, &mut out);
    out
}

fn scan_dir_inner(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    out: &mut Vec<AssetEntry>,
) {
    if depth > max_depth {
        return;
    }
    let Ok(read) = std::fs::read_dir(current) else {
        return;
    };

    let mut entries: Vec<std::fs::DirEntry> = read.flatten().collect();
    // Directories first, then files; each group sorted by name.
    entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));

    for entry in entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());

        if path.is_dir() {
            scan_dir_inner(root, &path, depth + 1, max_depth, out);
        } else {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            out.push(AssetEntry {
                relative_path: rel,
                kind: AssetKind::from_extension(ext),
            });
        }
    }
}

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

    #[test]
    fn scan_assets_returns_files_in_tmp() {
        let dir = std::env::temp_dir().join("novaforge_test_scan");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("hero.png"), b"").unwrap();
        std::fs::write(dir.join("map.ron"), b"").unwrap();

        let entries = scan_assets(&dir, 2);
        assert!(entries.iter().any(|e| e.relative_path.contains("hero.png")));
        assert!(entries.iter().any(|e| e.relative_path.contains("map.ron")));

        let png = entries
            .iter()
            .find(|e| e.relative_path.contains("hero.png"))
            .unwrap();
        assert_eq!(png.kind, AssetKind::Texture);

        let ron = entries
            .iter()
            .find(|e| e.relative_path.contains("map.ron"))
            .unwrap();
        assert_eq!(ron.kind, AssetKind::Scene);

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn asset_kind_from_extension() {
        assert_eq!(AssetKind::from_extension("png"), AssetKind::Texture);
        assert_eq!(AssetKind::from_extension("PNG"), AssetKind::Texture);
        assert_eq!(AssetKind::from_extension("vox"), AssetKind::Model);
        assert_eq!(AssetKind::from_extension("ogg"), AssetKind::Sound);
        assert_eq!(AssetKind::from_extension("ron"), AssetKind::Scene);
        assert_eq!(AssetKind::from_extension("xyz"), AssetKind::Other);
    }
}
