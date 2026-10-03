# **Functor — High‑Level Architecture**  
*A unified environment for coding, thinking, exploring, visualizing, and automating.*

---

## **1. Vision**
Functor is a hybrid **code editor**, **reactive notebook**, and **agentic workspace** designed to support:

- traditional programming workflows  
- computational exploration and visualization  
- narrative thinking and documentation  
- agent-assisted development  
- multi-language execution  
- project‑based organization  
- extensibility through plugins  

Functor aims to be a place where code, ideas, agents, and visualizations coexist seamlessly.

### Current Phase 1 Shell

The first Rust desktop shell targets Windows, macOS, and Linux through
`winit` 0.30.13. `src/main.rs` currently owns only the platform lifecycle:
window creation, event translation, redraw requests, resize notifications, and
clean shutdown. `Ctrl+O` opens the native folder picker, `Ctrl+P` opens a file,
and `Ctrl+S` saves the active file. These actions send selected paths and file
results through the existing effects. The MVU state,
backend-neutral view, and WGPU renderer are implemented in the sibling modules
under `src/`.

### Phase 1 Implementation Guidance

The current crate intentionally uses a flat module layout while the first
window is being proven:

- `main.rs` is the composition root and platform event loop.
- `input.rs` translates winit events into semantic messages.
- `mvu.rs` owns messages, pure update logic, commands, and transition tests.
- `model.rs` owns document editing, positions, selections, viewport state, and
	editor invariants.
- `workspace.rs` owns workspace data and directory discovery at the filesystem
	boundary; MVU receives its results as semantic messages.
- `file_io.rs` owns UTF-8 file loading and temporary-file saves; MVU receives
	file results and keeps dirty state authoritative.
- `view.rs` produces deterministic, backend-neutral frame geometry.
- `renderer.rs` owns WGPU setup and consumes only `FrameDescription`.

Keep this dependency direction stable:

```text
winit -> input adapter -> Message -> update -> Model -> view/layout
			-> FrameDescription -> renderer -> redraw request
                         workspace effect -> WorkspaceOpened -> update
```

Domain and view code must not depend on winit, WGPU, window handles, clocks, or
filesystem APIs. `workspace.rs` is the explicit filesystem boundary; it
returns stable workspace data and structured errors to the pure update loop.
`file_io.rs` is the corresponding file boundary, with UTF-8 as the initial
encoding policy.
Do not split the flat modules into the aspirational
directory tree merely for symmetry. Extract `app`/`platform` first when event
routing or transient pointer/modifier state grows; split model, MVU, view, and
renderer modules only when they have multiple ownership reasons or become hard
to test independently.

Keep document invariants at the document boundary, keep `FrameDescription`
inspectable and deterministic, and add headless tests before GUI tests. The
renderer currently loads Consolas as the Windows editor font and Segoe UI Emoji
as its fallback, with a public-domain bitmap path as a degraded fallback. The
intended production rendering path is Vello over WGPU, with Vello owning
vector composition and the eventual GPU text path. Text shaping, platform font
fallback, color-font support, and diagnostic overlays must be validated against
the selected Vello text stack. See
`docs/decisions/0006-vello-rendering-and-composition.md`.

For Rust changes, run `cargo fmt --check`, `cargo check`, and `cargo test`.
Changes to window lifecycle, input routing, rendering, or visible geometry
also require a `cargo run` smoke test on the primary desktop target.

---

## **2. Core Architectural Pillars**

### **2.1 Editor Engine (MVU Core)**
The editor is built on a deterministic **Model–View–Update** loop.

**Model**
- Rope-based text buffer  
- Cursor and selection state  
- Syntax highlighting metadata  
- Workspace structure (folder per project)  
- Open files, tabs, and editor layout  
- Cell metadata embedded in documents  

**View**
- GPU-accelerated rendering (Skia, Vello, wgpu)  
- Text shaping and layout  
- Inline cell output regions  
- HTML rendering surfaces  
- Panels, tabs, and workspace UI  

**Update**
- Input events → `Msg` → Model transitions  
- Deterministic state updates  
- Re-render on every state change  

The editor engine provides the foundation for all interactions.

---

### **2.2 Workspace System**
Functor organizes work into **projects**, each backed by a folder.

**Workspace Features**
- Project root folder  
- File tree navigation  
- Multiple open files  
- Project metadata (`functor.json`)  
- Cell execution cache  
- Agent configuration  
- Plugin configuration  

This enables reproducible, structured development workflows.

---

### **2.3 Reactive Notebook Engine**
Functor includes a Marimo-like reactive notebook system.

**Cell Types**
- Code cells (Rust, Python, Lua, WASM, etc.)  
- Markdown cells  
- HTML cells  
- Agent cells  
- Visualization cells  

**Cell Metadata**
- Language  
- Execution mode  
- Dependencies  
- Output type  
- Error state  
- Execution timestamp  

**Cell Graph (DAG)**
- Cells form a directed acyclic graph  
- Editing a cell invalidates dependents  
- Execution propagates updates  
- Outputs update reactively  

This enables narrative programming, exploration, and visualization.

---

### **2.4 Multi-Language Execution Layer**
Functor supports multiple languages through a unified execution interface.

**Executors**
- Rust (native or WASM)  
- Python (embedded or external)  
- Lua (mlua)  
- WASM (wasmtime)  
- Agentic (A2A, MCP)  

**Execution Contract**
Each executor returns:
- HTML  
- JSON  
- Text  
- Binary blobs (images, audio)  
- Logs  
- Errors  

This keeps Functor language-agnostic and extensible.

---

### **2.5 HTML Output Layer**
HTML is the universal output format for cell results.

#### **Native HTML Rendering**
- Vello vector engine  
- cosmic-text layout  
- Custom DOM tree  
- GPU-accelerated rendering  

HTML enables:
- rich visualizations  
- interactive widgets  
- charts, tables, SVG  
- custom components  
- multi-language output unification  

---

### **2.6 Agentic Layer**
Functor integrates agentic workflows directly into the environment.

**Agent Cells**
- OpFlow pipelines  
- A2A workflows  
- MCP tools  
- LLM-assisted transformations  
- Code generation and refactoring  
- Data analysis tasks  

**Agent Runtime**
- Async orchestration  
- Message passing  
- Stateful agents  
- Tool invocation  
- Streaming outputs  

Agents enhance both notebook and editor workflows.

---

### **2.7 Plugin System**
Functor is designed to be extensible.

**Plugin Types**
- Language executors  
- Syntax highlighters  
- Renderers  
- Agent runtimes  
- Custom cell types  
- Workspace tools  
- UI components  

**Plugin API**
- Traits  
- WASM modules  
- Proc-macro DSLs  

Plugins allow Functor to grow organically.

---

### **2.8 Persistence & Project Structure**
**Files**
- `.functor` project file  
- Source files  
- Notebook cell metadata  
- Execution cache  
- Agent configuration  
- Plugin configuration  

**Serialization**
- JSON or MessagePack  
- Deterministic snapshots  
- Reproducible notebooks  

---

## **3. Architecture Diagram**

```
+-------------------------------------------------------------------+
|                              Functor                              |
+-------------------------------------------------------------------+
|                         Editor Engine (MVU)                       |
|  - Text buffer (rope)                                             |
|  - Cursor/selection                                                |
|  - Syntax highlighting                                             |
|  - Rendering surface (Skia/Vello/wgpu)                            |
|  - Workspace UI                                                    |
+-------------------------------------------------------------------+
|                           Workspace System                         |
|  - Project folder                                                  |
|  - File tree                                                       |
|  - Project metadata                                                |
|  - Execution cache                                                 |
+-------------------------------------------------------------------+
|                           Notebook Engine                          |
|  - Cell boundaries                                                 |
|  - Cell languages                                                  |
|  - Cell DAG (dependencies)                                         |
|  - Reactive invalidation                                           |
+-------------------------------------------------------------------+
|                         Execution Engines                          |
|  - Rust executor                                                   |
|  - Python executor                                                 |
|  - Lua executor                                                    |
|  - WASM executor                                                   |
|  - Agentic executor (OpFlow)                                       |
+-------------------------------------------------------------------+
|                         HTML Output Layer                          |
|  - Native HTML renderer OR browser pane                            |
|  - Widget system                                                   |
|  - Interactive components                                          |
+-------------------------------------------------------------------+
|                           Agentic Layer                            |
|  - Pipelines                                                       |
|  - Tools                                                           |
|  - LLM integration                                                 |
+-------------------------------------------------------------------+
|                           Plugin System                            |
|  - Language plugins                                                |
|  - Render plugins                                                  |
|  - Agent plugins                                                   |
+-------------------------------------------------------------------+
|                           Persistence                              |
|  - Project files                                                   |
|  - Cell metadata                                                   |
|  - Execution cache                                                 |
+-------------------------------------------------------------------+
```

---

## **4. Summary**
Functor is a unified environment that blends:

- deterministic MVU editing  
- reactive notebook cells  
- multi-language execution  
- HTML rendering  
- agentic workflows  
- workspace organization  
- extensibility through plugins  

It is designed to support both **traditional programming** and **computational thinking**, giving you a powerful space to write code, explore ideas, visualize data, and collaborate with agents.
