# NovaForge Workspace

**NovaForge Workspace** is a native Rust development platform built around the
[Nova-Forge](https://github.com/shifty81/Nova-Forge) game. It provides a
unified launcher and a full suite of editor tools for building, iterating, and
shipping Nova-Forge worlds, assets, and gameplay — all without an internet
account or central authentication requirement.

> Built for people who want to just launch the game and build.

---

## Built Upon & Acknowledgements

NovaForge Workspace stands on the shoulders of extraordinary open-source work.

| Project | Role | Link |
|---|---|---|
| **Nova-Forge** by shifty81 | The game this workspace is built to serve | [github.com/shifty81/Nova-Forge](https://github.com/shifty81/Nova-Forge) |
| **AtlasWorkspace** by shifty81 | C++ workspace whose design & editor roster inspired this project | [github.com/shifty81/AtlasWorkspace](https://github.com/shifty81/AtlasWorkspace) |
| **Veloren** open-source community | Voxel RPG engine at the heart of Nova-Forge | [veloren.net](https://veloren.net) |
| **egui / eframe** by emilk | Immediate-mode GUI powering every editor panel | [github.com/emilk/egui](https://github.com/emilk/egui) |
| **egui_dock** contributors | Docking layout system for the master editor | [github.com/Adanos020/egui_dock](https://github.com/Adanos020/egui_dock) |
| **The Rust Community** | Language, toolchain, and crates.io ecosystem | [rust-lang.org](https://www.rust-lang.org) |

> Full credits with contributor details are in the [Credits & Acknowledgements](#credits--acknowledgements) section below.

---

## What is NovaForge Workspace?

| Feature | Description |
|---|---|
| **Launcher** | One-click Play, Host LAN Game, and Open Workspace |
| **Master Editor** | Dockable, tabbed editor hosting all tool panels in a single window |
| **Standalone Editors** | Every panel can also run as its own window via the `standalone` feature flag |
| **AI Tool (stub)** | Broker interface ready to connect to any AI provider |
| **Build Integration** | Direct pipeline to `nova-forge.sh` streamed into the Build Tool panel |
| **No account required** | Inherits Nova-Forge's auth-free play model |

---

## Editor Roster

| Panel | Purpose |
|---|---|
| Workspace Browser | Project & asset tree navigation |
| Scene Editor | 3D world / scene editing viewport |
| Asset Editor | Asset browsing and editing |
| Material Editor | Material & shader node graph authoring |
| Visual Logic Editor | Blueprint / node-graph logic |
| UI Editor | In-game UI layout design canvas |
| Animation Editor | Timeline, keyframes, and state machines |
| Data Editor | Data tables and configuration sheets |
| Build Tool | Build pipeline with live log streaming |
| AI Tool | AI broker panel (stub — offline by default) |

---

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable, see `rust-toolchain.toml`)
- A GPU with Vulkan, Metal, or OpenGL support
- Linux, macOS, or Windows

### Quick build & run

```bash
# Clone the workspace *with* the Nova-Forge game submodule
git clone --recurse-submodules https://github.com/shifty81/Workspace-Forge-Rust.git
cd Workspace-Forge-Rust

# (If you already cloned without --recurse-submodules, run this once:)
git submodule update --init --depth 1

# Launch the NovaForge launcher
./workspace.sh run

# Launch the full editor suite
./workspace.sh editors

# Build everything (debug)
./workspace.sh build

# Run all tests
./workspace.sh test
```

Or use Cargo directly:

```bash
# Launcher
cargo run -p novaforge-workspace

# Full editor suite
cargo run -p novaforge-editors

# A single editor in standalone mode
cargo run -p editor-scene --features standalone
```

### Opening the Nova-Forge project in the editor

A ready-made `novaforge.workspace.toml` is included at the repository root,
pointing to the `nova-forge/` submodule:

```toml
project_name    = "Nova-Forge"
nova_forge_path = "nova-forge"
asset_root      = "nova-forge/assets"
```

In the editor suite, open **File → Open** and type the path to this file
(or navigate to it with the file chooser), then click **Open**. The
Workspace Browser and Asset Editor will populate with the real game assets.

### Building the game itself

Nova-Forge uses its own Rust workspace (nightly toolchain) and build script.
After initialising the submodule you can invoke `nova-forge.sh` through the
workspace convenience script:

```bash
# Uses nova-forge/nova-forge.sh internally
./workspace.sh build-game

# Or invoke nova-forge.sh directly
cd nova-forge
bash nova-forge.sh run
```

> **Note:** Building Nova-Forge itself requires the nightly Rust toolchain
> (`nightly-2025-09-08`) plus the game's own system dependencies (GPU drivers,
> audio libraries, etc.).  The `build-game` command is separate from the
> workspace's own `cargo build`, which only builds the editor tools.

---

## Repository Layout

```
nova-forge/              # Git submodule — the Nova-Forge game (shifty81/Nova-Forge)
novaforge.workspace.toml # Ready-made manifest pointing at the submodule
crates/
  novaforge-workspace/   # Launcher binary
  novaforge-ui/          # Shared UI primitives & EditorPanel trait
  novaforge-editors/     # Master editor binary (hosts all panels)
  novaforge-project/     # Project manifest model & file I/O
  novaforge-build/       # Build pipeline (wraps nova-forge.sh)
  novaforge-ai/          # AI broker trait + StubAI implementation
  editors/
    editor-scene/        # Scene / World Editor
    editor-asset/        # Asset Browser & Editor
    editor-material/     # Material / Shader Editor
    editor-vlogic/       # Visual Logic (node graph) Editor
    editor-ui/           # UI Layout Editor
    editor-animation/    # Animation Timeline Editor
    editor-data/         # Data Table Editor
    editor-build/        # Build Tool panel
```

---

## Standalone vs. Master Editor

All panels live inside `novaforge-editors` (the master host) by default. Any
panel can be compiled as a standalone binary by enabling its `standalone`
feature:

```bash
cargo run -p editor-asset --features standalone
```

---

## License

NovaForge Workspace is released under the
**[GNU General Public License v3.0](LICENSE)** to match the Nova-Forge and
upstream Veloren license. It is free to use, modify, and distribute.

---

## Credits & Acknowledgements

NovaForge Workspace stands on the shoulders of extraordinary open-source work.
A sincere thank you to everyone listed here.

### Nova-Forge

- **shifty81** — author of [Nova-Forge](https://github.com/shifty81/Nova-Forge),
  the auth-free, LAN-first RPG that this workspace is built to serve.

### AtlasWorkspace

- **shifty81** — [AtlasWorkspace](https://github.com/shifty81/AtlasWorkspace),
  the C++ workspace platform whose architecture, editor roster, and design
  decisions were audited and ported to inform the structure of NovaForge
  Workspace.

### Veloren

- The entire **[Veloren](https://veloren.net) open-source community** — software
  developers, artists, composers, and translators who built the voxel RPG engine
  at the heart of Nova-Forge. NovaForge Workspace exists because Veloren does.
  See <https://veloren.net/contributors> for the full contributor list.

### egui / eframe

- **Emil Ernerfeldt (emilk)** and the [egui contributors](https://github.com/emilk/egui/graphs/contributors)
  for the immediate-mode GUI toolkit that powers every editor panel.

### egui_dock

- The [egui_dock contributors](https://github.com/Adanos020/egui_dock)
  for the docking layout system used in the master editor.

### The Rust Community

- The **[Rust project](https://www.rust-lang.org/)** maintainers and the broader
  crates.io ecosystem for the language, toolchain, and libraries that make
  NovaForge Workspace possible.

---

*NovaForge Workspace is not affiliated with the official Veloren project.*

