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
2. Click **📂 Browse…** to open a native file dialog — navigate to and select
   your `novaforge.workspace.toml`.  The path is filled in automatically and
   the project loads immediately after you confirm the dialog.
3. Alternatively, type the path directly into the **Project** text field and
   click **Open**.

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

### Recent Projects

After you successfully open a project its path is remembered for the current
session.  Up to **5 recent paths** are stored.  To re-open one:

1. Open the **File** menu.
2. Hover over **Recent Projects** (appears once at least one project has been
   opened).
3. Click any entry in the list — the path fills the field and the project loads
   immediately.

---

## Layout Overview

The default layout has three zones:

| Zone | Panels |
|---|---|
| **Left sidebar** | 📁 Workspace Browser |
| **Centre (tabbed)** | 🌐 Scene · 🖼 Assets · 🎨 Material · 🔗 V-Logic · 📐 UI · 🎬 Animation · 📋 Data · 📝 File Editor |
| **Bottom strip** | 🔨 Build · 🤖 AI Tool |

Every tab can be **dragged** to a different location, **closed** (View menu →
uncheck the panel name), or **re-opened** (View menu → check it back on).
**View → Reset Layout** restores the default arrangement.

The editor opens in **dark mode** by default.  Switch between dark and light via
**View → Theme → 🌙 Dark / ☀ Light** at any time.

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
| **⧉ Duplicate** | Clone the selected entity (position offset by +1 on X; placed at end of list) |
| **💾 Save** | Serialise all entities to `<asset_root>/scenes/scene.toml` |
| **📂 Load** | Load entities from `<asset_root>/scenes/scene.toml` |
| Click an entity row | Select it; transform fields appear in the inspector below the viewport |
| **Name** field | Rename the selected entity |
| **Position / Rotation / Scale** drag-values | Edit the transform; drag left/right to change, or click to type |

> The 3-D viewport shows a grid placeholder.  Full rendering integration with
> the Nova-Forge engine is a future milestone.
>
> **Pie menu** — right-click anywhere inside the viewport to open a radial
> action menu:
>
> | Slice | Action |
> |---|---|
> | ⬆ Translate | Switch gizmo to Translate mode |
> | ↻ Rotate | Switch gizmo to Rotate mode |
> | ⤢ Scale | Switch gizmo to Scale mode |
> | ＋ Entity | Add a new entity |
> | ⧉ Dup | Duplicate the selected entity (dimmed when nothing selected) |
> | 🗑 Delete | Delete the selected entity (dimmed when nothing selected) |
>
> Hover over a slice to highlight it, then **left-click** to execute.
> Click the central **✕** or press **Escape** to cancel without acting.

---

### 🖼 Asset Editor

Browses and inspects asset files found under the project's asset root.

| Control | Description |
|---|---|
| 🔍 search box | Filter by filename |
| **All / 🖼 Texture / 📦 Model / 🔊 Sound / 🌐 Scene / 📄 Other** | Filter by asset type |
| **⟳ Refresh** | Re-scan the asset root directory |
| Click an entry | Select it; the **Asset Details** panel appears below with path, type, file size, and last-modified date |

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
| **🗑 Clear Wires** | Remove all wire connections from the graph |
| Click a node | Select it (highlighted border; name shown in toolbar) |
| Drag a node | Reposition it on the canvas |
| Drag the canvas background | Pan the view |
| **Scroll wheel** | Zoom in or out (canvas must be hovered) |
| Drag from an **output port** (right edge, green circle) | Start drawing a wire |
| Release over an **input port** (left edge, orange circle) | Complete the wire connection |
| Release anywhere else | Cancel the in-progress wire |

Output ports appear on the right edge of each node; input ports appear on the left.  A yellow cubic Bézier curve is drawn while a wire is in flight.  Completed wires are drawn in amber.  Each input slot accepts at most one wire; connecting a second wire to the same slot is silently ignored.

---

### 🔗 Visual Logic Editor

A blueprint-style node graph for scripting game logic without code.

| Control | Description |
|---|---|
| **＋ Event Node** | Add a blue event-trigger node |
| **＋ Action Node** | Add a green action node |
| **＋ Branch** | Add a grey branch / condition node |
| **🗑 Delete** | Remove the selected node (and all edges touching it) |
| **🔍＋ / 🔍−** | Zoom in / out |
| **⊙ Reset** | Reset zoom and pan |
| **🗑 Clear Edges** | Remove all edge connections |
| Click a node | Select it (thicker border; name shown in toolbar) |
| Drag a node | Reposition it on the canvas |
| Drag the canvas background | Pan the view |
| **Scroll wheel** | Zoom in or out (canvas must be hovered) |
| Drag from the **output port** (right edge, green circle) | Start drawing an edge |
| Release over another node's **input port** (left edge, orange circle) | Connect the edge |
| Release anywhere else | Cancel the in-progress edge |

Edges are drawn as cubic Bézier curves.  Duplicate edges (same source → same target) are silently rejected.

---

### 📐 UI Editor

A drag-and-drop canvas for designing in-game UI layouts.

| Control | Description |
|---|---|
| **＋ Panel** | Add a Panel widget (dark blue, 120 × 60 px default) |
| **＋ Label** | Add a Label widget (dark green, 100 × 20 px default) |
| **＋ Button** | Add a Button widget (dark purple, 90 × 28 px default) |
| **🗑 Delete** | Remove the selected widget (enabled when a widget is selected) |
| Click a widget | Select it (bright border highlight); the **Inspector** appears below |
| Drag a widget | Move it around the canvas |
| Click the canvas background | Deselect the current widget |
| Widget count badge | Shows total number of widgets in the toolbar |

**Inspector** (visible when a widget is selected):

| Field | Description |
|---|---|
| **Label** | Editable display name shown on the widget |
| **Type** | Read-only widget type (Panel / Label / Button) |
| **Position X / Y** | Drag or type to move the widget's top-left corner |
| **Size W / H** | Drag or type to resize the widget |

Changes made in the Inspector are reflected on the canvas immediately.  Full property binding and widget hierarchy editing are planned for a future phase.

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
| **Drag a keyframe diamond** | Reposition it in time; keyframes re-sort automatically on release |
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
| **Click a column header (ID / Name / Type / Value / Tags)** | Sort rows ascending by that column; click again to reverse |
| **Apply** | Write the edit buffer back to the selected row |
| **Revert** | Discard edits and re-populate the buffer from the stored row |

Columns: **ID** · **Name** · **Type** · **Value** · **Tags** (comma-separated).

---

### 📝 Game File Editor

An inline text editor for Nova-Forge source and configuration files.  Any text
file selected in the **📁 Workspace Browser** is automatically opened here if
its extension is recognised as plain-text.

Supported extensions: `.toml`, `.ron`, `.json`, `.yaml`, `.yml`, `.lua`, `.txt`,
`.md`, `.conf`, `.cfg`, `.ini`, `.glsl`, `.wgsl`, `.vert`, `.frag`, `.comp`,
`.hlsl`, `.py`, `.sh`, `.bat`, `.rs`.

| Control | Description |
|---|---|
| File name badge | Shows `📄 filename` (or `✏ filename *` when unsaved changes exist) |
| **💾 Save** | Write the buffer to disk (enabled only when there are unsaved edits) |
| **✖ Close** | Close the file (unsaved changes are discarded without warning) |
| Editor area | Monospace text editor; any keystroke marks the file as dirty |
| **Ctrl+S** | Save the open file from anywhere in the editor |

> The editor stores one buffer at a time.  Opening a second file from the
> Workspace Browser replaces the current buffer.

---

### 🔨 Build Tool

Runs `nova-forge.sh` commands and streams live log output.

Builds can be triggered from three places:
- Buttons inside the **🔨 Build** panel
- The **Build** menu in the menu bar (also switches focus to the Build panel)
- Keyboard shortcuts **Ctrl+B** (debug build) and **Ctrl+R** (run)

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
cargo run -p editor-gamefile  --features standalone
```

Standalone windows are useful when you want to focus on a single tool without
the full multi-panel host.

---

## Keyboard & Mouse Reference (global)

| Input | Effect |
|---|---|
| Drag a tab header | Re-dock the panel to a different position |
| Middle-click / drag canvas | Pan the node-graph canvases |
| Scroll wheel on canvas | Zoom in/out (Material, V-Logic canvases) |
| `Enter` in the File → Project field | Open the project immediately |
| **Ctrl+S** | Save the currently open file in the Game File Editor (if dirty) |
| **Ctrl+B** | Trigger a debug build (opens the Build panel and starts `nova-forge.sh build`) |
| **Ctrl+R** | Build and run the game client (opens the Build panel and starts `nova-forge.sh run`) |

---

*See [building.md](building.md) for instructions on compiling the workspace and
the Nova-Forge game.*
