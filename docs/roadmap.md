# **Functors Roadmap**
*A living document outlining the initial development path for the Functors editor.*

---

## **Phase 0 — Foundations & Exploration**  
**Goal:** Establish the core project structure and validate architectural assumptions.

- Set up repository structure (`src/`, `docs/`, `crates/`)  
- Define initial architecture documents  
- Explore rendering options (Skia, Vello, wgpu)  
- Prototype MVU loop in Rust  
- Evaluate text buffer libraries (ropey, xi-rope)  
- Experiment with HTML rendering approaches  
- Identify minimal plugin architecture shape  

**Outcome:** A clear technical direction and a stable foundation for Phase 1.

---

## **Phase 1 — Core Editor Engine (MVU)**  
**Goal:** Build the minimal editor capable of opening, editing, and rendering text.

- Implement MVU core (Model, Msg, Update, View)  
- Integrate GPU rendering surface  
- Add basic text layout + cursor movement  
- Implement rope-based text buffer  
- Add simple syntax highlighting (Rust + Markdown)  
- Create basic workspace structure (open folder, list files)  
- Add file open/save support  

**Outcome:** Core MVU implementation complete. The usability gate remains open
until richer menu interaction, scrollbar dragging, and a complete
command/settings workflow are in place.

### Phase 1 usability extension

- [x] Semantic vertical and horizontal scrolling with visible scrollbar thumbs.
- [x] Unicode bitmap glyph coverage for supported scripts with visible fallback
  for unsupported characters.
- [x] Clickable menu strip and `Ctrl+K` command bar shell.
- [x] Settings command opens a visible settings surface.
- [x] Integrate a real font rasterizer with Consolas and Segoe UI Emoji fallback.
- [x] Adopt Skia for production cross-platform text shaping, emoji rendering,
      and diagnostics overlays.
- [ ] Add scrollbar thumb dragging and complete menu command behavior.

Phase 2 should not begin until this extension is complete. The bitmap/fontdue
path remains an intermediate fallback. Native color emoji validation and richer
visual programming diagnostics remain follow-up work on the Skia foundation.

---

## **Phase 2 — Notebook Cell System (Foundations)**  
**Goal:** Introduce reactive cells inside documents.

- Define cell boundaries (fenced blocks, markers)  
- Add cell metadata to the model  
- Implement cell detection + parsing  
- Render inline cell output regions  
- Create minimal cell DAG (dependencies, invalidation)  
- Add basic cell execution triggers  

**Outcome:** Documents can contain executable cells with reactive behavior.

---

## **Phase 3 — Multi-Language Execution Layer**  
**Goal:** Enable execution of code cells in multiple languages.

- Define unified executor trait  
- Implement Rust executor (native or WASM)  
- Implement Python executor (embedded or external)  
- Implement Lua executor (mlua)  
- Add WASM executor (wasmtime)  
- Standardize execution output format (HTML, JSON, text, binary)  
- Add error reporting + logs  

**Outcome:** Cells can run code and produce structured output.

---

## **Phase 4 — HTML Output Layer**  
**Goal:** Render cell results using HTML as the universal output format.

- Choose rendering strategy (native HTML vs embedded webview)  
- Implement HTML rendering surface  
- Add support for:  
  - text  
  - tables  
  - SVG  
  - images  
  - interactive widgets  
- Add sandboxing + isolation for HTML content  

**Outcome:** Cells can display rich, interactive visualizations.

---

## **Phase 5 — Agentic Layer (Copilot + Local LLMs)**  
**Goal:** Integrate agent workflows into the editor and notebook.

- Define agent cell type  
- Integrate Copilot APIs  
- Add local LLM support (via plugins)  
- Implement OpFlow runtime integration  
- Add agent pipelines + tools  
- Support streaming outputs  
- Add agent-assisted refactoring + code generation  

**Outcome:** Functors becomes an agentic workspace capable of automation and exploration.

---

## **Phase 6 — Plugin System**  
**Goal:** Make Functors extensible.

- Define plugin API (traits + WASM modules)  
- Add plugin loader  
- Support:  
  - language executors  
  - syntax highlighters  
  - renderers  
  - agent runtimes  
  - custom cell types  
- Add plugin configuration per workspace  

**Outcome:** Functors becomes a platform rather than a monolithic tool.

---

## **Phase 7 — Workspace & UX Enhancements**  
**Goal:** Polish the user experience and project workflow.

- Improve workspace navigation  
- Add tabs, panels, split views  
- Add project metadata (`functor.json`)  
- Add execution cache  
- Add notebook export (HTML, PDF — via plugins)  
- Add settings UI  
- Add theming + customization  

**Outcome:** A cohesive, comfortable environment for daily use.

---

## **Phase 8 — Stabilization & Release Prep**  
**Goal:** Prepare for public release.

- Performance tuning  
- Memory profiling  
- Rendering optimizations  
- Executor sandboxing  
- Plugin security review  
- Documentation + examples  
- Branding + website  

**Outcome:** A stable, polished version ready for early adopters.

---

## **Status**
Phase 1 is complete. This roadmap remains a living plan and will evolve as
notebook, execution, agent, plugin, and workspace UX capabilities are added.
