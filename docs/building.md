# NovaForge Workspace — Build Guide

This guide explains how to build the **NovaForge Workspace** editor tools and
how to build and run the **Nova-Forge game** itself.

---

## Prerequisites

### For the Workspace (editor tools)

| Requirement | Notes |
|---|---|
| **Rust stable toolchain** | Install via [rustup.rs](https://rustup.rs/). The required version is pinned in `rust-toolchain.toml`. |
| **GPU with Vulkan, Metal, or OpenGL support** | Required by the egui rendering backend. |
| **Linux** (X11 or Wayland), **macOS**, or **Windows** | All three are supported. |

On **Linux** you may need additional system packages for GPU / audio access,
for example on Debian/Ubuntu:

```bash
sudo apt install libx11-dev libxcb1-dev libwayland-dev libxkbcommon-dev \
                 libvulkan-dev vulkan-tools
```

### For the Nova-Forge Game

Nova-Forge uses the **nightly Rust toolchain** (`nightly-2025-09-08`) plus its
own system dependencies (GPU drivers, audio libraries, etc.).  These are
managed by the game's own `rust-toolchain.toml` inside the `nova-forge/`
submodule.

See the [Nova-Forge README](https://github.com/shifty81/Nova-Forge) for the
full dependency list.

---

## Getting the source

```bash
# Clone the workspace repository including the Nova-Forge game submodule
git clone --recurse-submodules https://github.com/shifty81/Workspace-Forge-Rust.git
cd Workspace-Forge-Rust

# If you already cloned without --recurse-submodules, initialise the submodule now:
git submodule update --init --depth 1
```

---

## Building the Workspace (editor tools)

The `workspace.sh` convenience script wraps the most common Cargo commands.

### Quick reference

```bash
# Build all workspace crates (debug)
./workspace.sh build

# Build all workspace crates (optimised release)
./workspace.sh release

# Fast compile check (no codegen)
./workspace.sh check

# Run the launcher
./workspace.sh run

# Run the full editor suite
./workspace.sh editors

# Run all tests
./workspace.sh test

# Run Clippy linter
./workspace.sh clippy

# Remove build artefacts
./workspace.sh clean
```

### Using Cargo directly

```bash
# Build everything
cargo build --workspace

# Build release
cargo build --workspace --release

# Run the launcher
cargo run -p novaforge-workspace

# Run the editor suite
cargo run -p novaforge-editors

# Run a single panel in standalone mode
cargo run -p editor-scene --features standalone

# Run tests
cargo test --workspace

# Clippy
cargo clippy --workspace --all-targets -- -D warnings
```

### Build outputs

After a debug build all binaries land in `target/debug/`:

| Binary | Description |
|---|---|
| `novaforge-workspace` | The launcher |
| `novaforge-editors` | The full editor suite |
| `editor-scene` *(standalone only)* | Scene Editor standalone |
| `editor-asset` *(standalone only)* | Asset Editor standalone |
| `editor-material` *(standalone only)* | Material Editor standalone |
| `editor-vlogic` *(standalone only)* | Visual Logic Editor standalone |
| `editor-ui` *(standalone only)* | UI Editor standalone |
| `editor-animation` *(standalone only)* | Animation Editor standalone |
| `editor-data` *(standalone only)* | Data Editor standalone |
| `editor-build` *(standalone only)* | Build Tool standalone |

Standalone binaries are only compiled when you pass `--features standalone` for
their crate.

### Build profiles

| Profile | Command | `opt-level` | LTO |
|---|---|---|---|
| **debug** | `cargo build` | 1 | none |
| **release** | `cargo build --release` | 3 | thin |

---

## Building and Running the Nova-Forge Game

Nova-Forge is included as a git submodule at `nova-forge/`.  It ships its own
build script (`nova-forge.sh` / `nova-forge.bat`) that wraps Cargo with the
correct nightly toolchain.

### Via the workspace script

```bash
# Build the game (debug by default; pass 'release' for an optimised build)
./workspace.sh build-game

# Pass extra arguments straight through to nova-forge.sh
./workspace.sh build-game release
```

`workspace.sh build-game` is equivalent to:

```bash
cd nova-forge
bash nova-forge.sh
```

### Via nova-forge.sh directly

```bash
cd nova-forge

# Build and run the game client
bash nova-forge.sh run

# Build only (debug)
bash nova-forge.sh build

# Build only (release)
bash nova-forge.sh release

# Start a dedicated LAN server
bash nova-forge.sh server

# Run the game test suite
bash nova-forge.sh test

# Remove game build artefacts
bash nova-forge.sh clean
```

### Via the launcher or editor Build Tool

Once you have opened `novaforge.workspace.toml` in the launcher or editor:

* **Launcher → ▶ Play** — launches the pre-built game client binary.
* **Launcher → 🌐 Host LAN** — starts a LAN server then connects the client.
* **Editor → Build Tool → ▶ Run** — builds and launches the client with live
  log streaming in the editor.
* **Editor → Build Tool → 🔨 Build** — debug build with live output.
* **Editor → Build Tool → 🚀 Release** — release build with live output.

> **Note:** The launcher's **Play** and **Host LAN** buttons expect the game
> client binary to already exist at `nova_forge_path/target/release/nova-forge-voxygen`.
> Run a release build first if you are launching for the first time.

---

## Continuous Integration

The repository's CI pipeline (`.github/workflows/ci.yml`) runs:

1. `cargo check --workspace` — compile check
2. `cargo clippy --workspace --all-targets -- -D warnings` — lint (zero warnings)
3. `cargo fmt --all -- --check` — formatting check
4. `cargo test --workspace` — unit tests

These all target the workspace's own stable toolchain and do **not** build the
Nova-Forge game itself (the nightly toolchain is not installed in CI).

---

## Troubleshooting

### `nova-forge/` submodule is empty

```
ERROR: nova-forge/ submodule not initialised.
```

Run:

```bash
git submodule update --init --depth 1
```

### GPU / display errors on Linux

If you see errors like `No display server` or `wgpu backend not found`, make
sure the relevant Vulkan or OpenGL packages are installed and that you are
running from a graphical desktop session (not a headless SSH connection without
display forwarding).

### Build fails with toolchain mismatch

The workspace uses the Rust stable toolchain specified in `rust-toolchain.toml`.
The game uses the nightly toolchain specified in `nova-forge/rust-toolchain.toml`.
Make sure both are installed:

```bash
# Workspace toolchain (read from rust-toolchain.toml automatically)
rustup show

# Game toolchain
rustup toolchain install nightly-2025-09-08
```

### Clippy reports warnings as errors

Run `./workspace.sh clippy` or `cargo clippy --workspace --all-targets -- -D warnings`
and fix any warnings before pushing to CI.

---

*See [editor-guide.md](editor-guide.md) for a full walkthrough of every editor
panel.*
