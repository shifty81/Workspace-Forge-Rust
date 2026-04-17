# NovaForge Workspace — Editor Guide

This guide explains how to open, navigate, and use every panel in the
**NovaForge Editor Suite** (`novaforge-editors`).

---

## Opening the Editor Suite

```bash
# From the repo root (recommended)
./workspace.sh editors

# Or with Cargo directly
cargo run -p novaforge-editors
```

The editor opens a 1 600 × 960 window with a dockable multi-panel layout.

---

## Loading a Project

A project file (`novaforge.workspace.toml`) tells the editor where the
Nova-Forge game code and assets live.

1. Open the **File** menu in the menu bar.
2. Type the path to your `novaforge.workspace.toml` (or the directory that
   contains it) in the **Project** text field.
3. Click **Open**.

The ready-made manifest at the repository root works immediately after
initialising the `nova-forge/` git submodule:

```
project_name    = "Nova-Forge"
nova_forge_path = "nova-forge"
asset_root      = "nova-forge/assets"
```

Once a project is loaded the status bar at the bottom of the window shows the
project name, and the **Workspace Browser** and **Asset Editor** panels
populate automatically.

---

## Layout Overview

The default layout has three zones:

| Zone | Panels |
|---|---|
| **Left sidebar** | 📁 Workspace Browser |
| **Centre (tabbed)** | 🌐 Scene · 🖼 Assets · 🎨 Material · 🔗 V-Logic · 📐 UI · 🎬 Animation · 📋 Data |
| **Bottom strip** | 🔨 Build · 🤖 AI Tool |

Every tab can be **dragged** to a different location, **closed** (View menu →
uncheck the panel name), or **re-opened** (View menu → check it back on).
**View → Reset Layout** restores the default arrangement.

---

## Panel Reference

### 📁 Workspace Browser

Displays the file tree of the project's asset root directory.

| Action | How |
|---|---|
| Filter entries | Type in the 🔍 search box at the top |
| Refresh the file tree | Click the **⟳** button next to the search box |
| Expand a directory | Click **▾ 📁 dirname** (expanded by default) |
| Collapse a directory | Click **▾ 📁 dirname** to toggle to **▸ 📁 dirname** |
| Select a file | Click its row |

The browser auto-populates when you open a project and reflects the asset root
defined in `novaforge.workspace.toml`.  Directories start expanded; click the
arrow prefix to collapse any subtree.  File icons are inferred from their
extension using the same type map as the Asset Editor (🖼 textures, 📦 models,
🔊 sounds, 🌐 scenes, 📄 other).

---

### 🌐 Scene Editor

A world/scene editor with an entity list and transform inspector.

| Control | Description |
|---|---|
| **⬆ Translate / ↻ Rotate / ⤢ Scale** | Select the active gizmo mode |
| **＋ Entity** | Add a new entity to the scene |
| **🗑 Delete** | Remove the selected entity (enabled only when something is selected) |
| **💾 Save** | Serialise all entities to `<asset_root>/scenes/scene.toml` |
| **📂 Load** | Load entities from `<asset_root>/scenes/scene.toml` |
| Click an entity row | Select it; transform fields appear in the inspector below the viewport |
| **Name** field | Rename the selected entity |
| **Position / Rotation / Scale** drag-values | Edit the transform; drag left/right to change, or click to type |

> The 3-D viewport shows a grid placeholder.  Full rendering integration with
> the Nova-Forge engine is a future milestone.

---

### 🖼 Asset Editor

Browses and inspects asset files found under the project's asset root.

| Control | Description |
|---|---|
| 🔍 search box | Filter by filename |
| **All / 🖼 Texture / 📦 Model / 🔊 Sound / 🌐 Scene / 📄 Other** | Filter by asset type |
| **⟳ Refresh** | Re-scan the asset root directory |
| Click an entry | Select it; the **Asset Details** panel appears below |

Asset kinds are inferred from file extensions:

| Extension | Kind |
|---|---|
| png, jpg, jpeg, webp, tga, bmp | Texture |
| vox, obj, gltf, glb, fbx, mesh | Model |
| ogg, wav, mp3, flac | Sound |
| ron, scene | Scene |
| anything else | Other |

---

### 🎨 Material Editor

A node-graph canvas for authoring materials and shaders.

| Control | Description |
|---|---|
| **＋ Add Node** | Append a new generic node to the graph |
| **🗑 Delete Node** | Remove the currently selected node (enabled when a node is selected) |
| **🔍＋ / 🔍−** | Zoom the canvas in or out |
| **⊙ Reset View** | Return to default zoom and pan |
| Click a node | Select it (highlighted border; name shown in toolbar) |
| Drag a node | Reposition it on the canvas |
| Drag the canvas background | Pan the view |

Each node displays coloured port stubs on its left (inputs) and right (output)
edges.  Full wire-connection interaction and a real egui_node_graph integration
are planned for a future phase.

---

### 🔗 Visual Logic Editor

A blueprint-style node graph for scripting game logic without code.

| Control | Description |
|---|---|
| **＋ Event Node** | Add a blue event-trigger node |
| **＋ Action Node** | Add a green action node |
| **＋ Branch** | Add a grey branch / condition node |
| **🗑 Delete** | Remove the selected node |
| **🔍＋ / 🔍−** | Zoom in / out |
| **⊙ Reset** | Reset zoom and pan |
| Click a node | Select it (thicker border; name shown in toolbar) |
| Drag a node | Reposition it on the canvas |
| Drag the canvas background | Pan the view |

Edges between the default starter nodes are drawn as straight lines.  A full
wire-drawing interaction layer is planned for a future phase.

---

### 📐 UI Editor

A drag-and-drop canvas for designing in-game UI layouts.

| Control | Description |
|---|---|
| **＋ Panel** | Add a Panel widget (dark blue, 120 × 60 px default) |
| **＋ Label** | Add a Label widget (dark green, 100 × 20 px default) |
| **＋ Button** | Add a Button widget (dark purple, 90 × 28 px default) |
| **🗑 Delete** | Remove the selected widget (enabled when a widget is selected) |
| Click a widget | Select it (bright border highlight) |
| Drag a widget | Move it around the canvas |
| Click the canvas background | Deselect the current widget |
| Widget count badge | Shows total number of widgets in the toolbar |

Each widget type is colour-coded on the canvas.  Full property binding and
widget hierarchy editing are planned for a future phase.

---

### 🎬 Animation Editor

A timeline editor for skeletal animation clips.

| Control | Description |
|---|---|
| **⏮ Start** | Jump the playhead to time 0 |
| **▶ Play / ⏸ Pause** | Toggle playback |
| **⏹ Stop** | Stop and reset playhead to 0 |
| **Zoom ＋ / −** | Stretch / compress the timeline horizontally |
| **Track: [name] ＋ Track** | Type a track name then click **＋ Track** to add a new track |
| **🗑 Track** | Delete the selected track and all its keyframes (enabled when a track is selected) |
| **＋ Keyframe** | Add a keyframe at the current playhead position on the selected track |
| **🗑 Keyframe** | Delete the selected keyframe |
| **💾 Save** | Serialise tracks to `<asset_root>/animations/animation.toml` |
| **📂 Load** | Load tracks from `<asset_root>/animations/animation.toml` |
| Click a track label | Select the track (required for keyframe operations) |
| Click a keyframe diamond | Select the keyframe on its track |
| Click the ruler row | Scrub the playhead to that time |

Time is displayed as `<current> / <total>` seconds in the transport bar.  The
playhead (red vertical line) advances automatically during playback.

---

### 📋 Data Editor

A spreadsheet-style editor for game data tables (items, NPCs, zones, etc.).

| Control | Description |
|---|---|
| 🔍 search box | Filter rows across all columns |
| **＋ Row** | Append a new blank row |
| **🗑 Delete** | Remove the selected row (enabled only when a row is selected) |
| **💾 Save** | Write all rows to `<asset_root>/data/data_table.toml` |
| **📂 Load** | Load rows from `<asset_root>/data/data_table.toml` |
| Click a row | Select it; the inline **Edit Row** form appears below |
| **Apply** | Write the edit buffer back to the selected row |
| **Revert** | Discard edits and re-populate the buffer from the stored row |

Columns: **ID** · **Name** · **Type** · **Value** · **Tags** (comma-separated).

---

### 🔨 Build Tool

Runs `nova-forge.sh` commands and streams live log output.

| Button | Equivalent command |
|---|---|
| **🔨 Build** | `nova-forge.sh build` (debug) |
| **🚀 Release** | `nova-forge.sh release` (optimised) |
| **🧹 Clean** | `nova-forge.sh clean` |
| **▶ Run** | `nova-forge.sh run` (build + launch client) |
| **🖥 Server** | `nova-forge.sh server` (build + launch dedicated server) |
| **🧪 Test** | `nova-forge.sh test` |
| **🗑 Clear** | Clear the log pane |
| **Auto-scroll** checkbox | Keep the log scrolled to the newest line |

A spinner and **Building…** indicator appear while a command is running.
All buttons are disabled during a build to prevent concurrent invocations.

> The Build Tool requires a project to be loaded so it can locate
> `nova-forge.sh` inside the `nova_forge_path` directory.

---

### 🤖 AI Tool

An AI assistant broker panel (offline stub by default).

| Element | Description |
|---|---|
| **● Online / ● Offline** status | Shows whether an AI provider is connected |
| **Provider** label | Displays the active provider name (currently "Offline (stub)") |
| Chat history | Scrollable log of prompts and responses |
| Text input + **Send** | Type a prompt and submit (Enter or click Send) |

To connect a real AI provider implement the `WorkspaceAI` trait from the
`novaforge-ai` crate and swap in your `Box<dyn WorkspaceAI>` in `AiToolPanel::new()`.

---

## Standalone Panels

Any panel can be compiled and run as a standalone window by enabling its
`standalone` Cargo feature:

```bash
cargo run -p editor-scene     --features standalone
cargo run -p editor-asset     --features standalone
cargo run -p editor-material  --features standalone
cargo run -p editor-vlogic    --features standalone
cargo run -p editor-ui        --features standalone
cargo run -p editor-animation --features standalone
cargo run -p editor-data      --features standalone
cargo run -p editor-build     --features standalone
```

Standalone windows are useful when you want to focus on a single tool without
the full multi-panel host.

---

## Keyboard & Mouse Reference (global)

| Input | Effect |
|---|---|
| Drag a tab header | Re-dock the panel to a different position |
| Middle-click / drag canvas | Pan the node-graph canvases |
| Scroll wheel on canvas | (planned) Zoom in/out |
| `Enter` in the File → Project field | Open the project immediately |

---

*See [building.md](building.md) for instructions on compiling the workspace and
the Nova-Forge game.*
