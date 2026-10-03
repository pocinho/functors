# Considering Floem for Functors

This note evaluates the Floem project (`https://github.com/lapce/floem`) as a
possible application UI layer for Functors. It is based on the inspected
workspace state on 2026-10-03, including Floem's Vello renderer, renderer
abstraction, text layout, editor core, reactive runtime, and application shell.
It is a comparison and evaluation plan, not an adoption decision.

## Short Recommendation

Floem deserves a focused proof of concept. It is the strongest candidate so far
for Functors' application chrome, widgets, layout, text interaction, themes,
animations, and window integration. Its default Vello path also aligns with the
current Functors rendering direction.

Do not replace Functors' MVU and domain model with Floem signals during the
proof of concept. Keep document editing, workspace effects, semantic messages,
and testable invariants in Functors-owned code. Treat Floem as a candidate
platform/view layer behind an adapter until the following questions are
answered:

- Can Functors' current editor model drive Floem without duplicating state?
- Can the editor's shaped text, cursor, selection, diagnostics, and scrolling
  remain deterministic and testable?
- Can Floem's Vello backend be pinned compatibly with Functors' WGPU and winit
  versions without accepting an unmaintainable dependency fork?
- Does Floem provide enough control over notebook output surfaces, focus,
  accessibility, and custom chrome for the actual Functors vision?

The likely outcome is either a Floem-backed application shell with Functors-owned
editor/domain logic, or a decision to keep the existing custom renderer. A full
replacement should follow evidence from the proof of concept, not the breadth
of Floem's widget API alone.

## What Floem Provides

The inspected Floem workspace is organized around these useful boundaries:

- `floem` provides the application, view tree, widgets, styling, events, focus,
  windows, and reactive integration.
- `floem_reactive` provides fine-grained signals and effects.
- `floem_renderer` defines a backend-neutral renderer trait for fills, strokes,
  layers, text, images, SVGs, transforms, clipping, and presentation.
- `floem_vello_renderer` implements the renderer trait with Vello and WGPU.
- `floem_renderer::text` and Floem's text module use Parley vocabulary,
  Fontique font metadata, and Swash-backed font operations for shaped glyph
  runs, cursor positions, selections, and visual lines.
- `floem-editor-core` contains editor-oriented rope, cursor, selection, command,
  movement, and document primitives based on `lapce-xi-rope`.
- Taffy supplies Flexbox and Grid layout.
- The project includes themes, transitions, keyframe animations, localization,
  virtual-list examples, an inspector, file dialogs, menus, and native window
  helpers.

Floem's README states that its rendering options include Vello, Vger, AnyRender
Skia, and tiny-skia. In the inspected workspace, Vello is the default feature,
while the other backends remain selectable. Floem is MIT-licensed, but the
repository is still pre-1.0 and explicitly warns that breaking changes are
expected.

## Fit With the Functors Vision

### Code editor

**Strong fit.** Floem already has editor-specific concepts rather than only
basic text labels. Its text layout wraps Parley and exposes cursor, selection,
visual-line, tab-expansion, fallback-font, and glyph-run concerns. Its editor
core includes rope-backed documents and command/movement behavior that can
inform Functors' own model design.

The risk is ownership overlap. Functors already owns document invariants,
selection semantics, MVU messages, and viewport state. Importing Floem's editor
view or editor core wholesale could create two authorities for editing and
cursor behavior. Reuse patterns and rendering integration first; adopt domain
code only after comparing invariants and tests.

### Reactive notebooks

**Promising but incomplete.** Floem's view tree, signals, layout, virtual lists,
transitions, and custom drawing are a good foundation for notebook chrome,
cell navigation, execution badges, and output sizing. Vello supports the vector
composition needed for overlays and cell decoration.

Floem does not provide Functors' reactive notebook semantics, dependency graph,
execution cache, multi-language executors, or HTML trust boundary. Those must
remain Functors subsystems. HTML output also cannot be assumed to become a
native Floem view; it needs an explicit webview or output-surface boundary.

### Narrative and visual work

**Good fit for native surfaces.** Taffy layout, custom painting, SVG and image
support, animations, and Vello composition can support panels, diagrams, inline
outputs, and themed visualizations. A renderer trait also makes a CPU fallback
possible for tests or unsupported hardware.

**Still requires Functors work.** Floem does not define the semantic data model,
serialization, execution protocol, visualization grammar, or plugin contract
needed by the Functors vision.

### Agentic workspace

**Neutral to positive.** Floem supplies windows, menus, dialogs, focus, events,
and reactive presentation. It does not constrain how agent messages, tools,
streams, permissions, or execution effects are modeled. The main requirement is
to keep agent effects outside view closures and connect them through an explicit
Functors message/effect adapter.

### Cross-platform and future WebGPU

**Positive with dependency caution.** Floem targets Windows, macOS, Linux, and
WASM-related paths, and its GPU resource acquisition explicitly handles
asynchronous device setup for WebGPU. Its Vello backend demonstrates a practical
surface strategy: render Vello's compute-oriented output into a storage texture,
then blit that texture to the swapchain surface.

The current Floem workspace uses `wgpu` 27.0 and a git-pinned winit fork,
whereas Functors currently uses WGPU 30.0.1 and winit 0.30.13. This is not a
minor version detail: surface lifetimes, device descriptors, event APIs, and
Vello integration can all be affected. Compatibility must be proved with a
small build before considering a dependency merge.

## Architectural Comparison

| Concern | Functors today | Floem | Evaluation |
|---|---|---|---|
| State model | Pure MVU transitions | Fine-grained signals and effects | Floem is more ergonomic for UI updates; Functors is easier to replay and test as a whole |
| View model | Pure `Model -> FrameDescription` | Long-lived reactive view tree | Do not let both own the same state |
| Layout | Hand-built deterministic geometry | Taffy Flexbox/Grid | Floem is substantially stronger for app chrome and responsive panels |
| Text | Transitional pixel bridge with Skia/fontdue code | Parley, Fontique, Swash vocabulary, Vello glyph runs | Floem is valuable evidence for the Vello text spike |
| Rendering | Direct WGPU geometry pipeline | Renderer trait with Vello, Vger, Skia, and tiny-skia backends | Floem reduces backend plumbing but adds a framework boundary |
| Editor core | Functors model and rope invariants | `floem-editor-core` and Lapce-derived editor primitives | Compare semantics before reuse |
| Widgets | Hand-built frame geometry | Mature composable widgets and styling | Major Floem advantage |
| Notebook semantics | Planned Functors subsystem | Not provided | Remains Functors-owned |
| HTML output | Planned explicit boundary | Not provided by the core UI stack | No decisive advantage |
| Windowing | Direct winit integration | Managed application/window layer | Floem reduces shell work but may reduce lifecycle control |
| Stability | Small, local Phase 1 codebase | Pre-1.0 framework with many dependencies | Floem's productivity gain carries upgrade and coupling cost |

## Main Advantages

### 1. It validates the desired Vello stack in real application code

Floem's Vello backend is more useful to Functors than a minimal rectangle
example. It demonstrates:

- a retained Vello renderer and per-frame scene;
- a WGPU storage target plus swapchain blit;
- surface reconfiguration and resize handling;
- draw-glyphs integration with font data, transforms, brushes, and normalized
  coordinates;
- SVG scene caching and compositing layers;
- optional off-screen capture for inspection.

This should substantially reduce the unknowns in Functors' Vello implementation
plan. It does not eliminate the need to validate the exact dependency versions.

### 2. It solves the application-chrome problem at the right level

Functors' current blockers include menus, command bars, settings, scrollbars,
focus, themes, and future panels. Floem supplies a coherent framework for
these concerns instead of requiring Functors to grow a custom widget system
inside the renderer.

### 3. It has an editor-shaped text path

The Parley-based `TextLayout` wrapper is particularly relevant. It connects
shaping to visual lines, cursor points, selections, tab expansion, and glyph
painting. That is closer to Functors' requirements than treating text as a
collection of independent strings.

### 4. It offers renderer substitution and inspection

The renderer trait and inspector are valuable engineering patterns. Functors
could use the same conceptual split to keep a backend-neutral scene contract,
provide a headless or software path, and capture frames for visual regression
or diagnostics.

### 5. It accelerates non-editor UI work

Themes, transitions, animations, localization, layouts, menus, dialogs, and
virtualized lists are expensive to build correctly. Floem would let Functors
spend more time on documents, execution, notebooks, and agents.

## Main Costs and Risks

### 1. It conflicts with the pure view contract if adopted directly

Floem constructs a view tree once and updates it through signals and effects.
Functors currently emphasizes pure updates and deterministic
`Model -> FrameDescription` generation. A direct rewrite would risk:

- duplicating state in both the Functors model and Floem signals;
- moving document invariants into UI closures;
- making transitions dependent on subscription order or runtime effects;
- losing simple headless tests for layout and rendering inputs.

The safe integration is an adapter boundary: Functors remains authoritative for
domain state and messages, while Floem owns presentation-local state such as
hover, focus, widget measurement, and animation progress.

### 2. It is a large dependency and version commitment

The inspected workspace includes multiple renderer crates, reactive runtime,
Taffy, Parley, Fontique, Swash, UI events, winit integration, several
`understory` git dependencies, and a winit git fork. Functors would trade a
small prototype for a substantial framework dependency graph.

The WGPU and winit versions do not currently match Functors. Floem also tracks
its own release cadence and is pre-1.0. Upgrades could affect application code,
rendering behavior, and platform support at once.

### 3. It may be too much framework for the first migration

The immediate Functors goal is to replace the renderer while preserving
`FrameDescription`. Taking the full Floem application stack before proving that
boundary would make failures difficult to localize. A Vello renderer spike is
smaller and remains valuable even if Floem is later adopted.

### 4. Editor behavior still needs ownership decisions

Floem's editor core is attractive, but Functors has its own rope, selection,
viewport, workspace, file, and MVU contracts. Combining the two editor models
would create subtle differences in Unicode positions, selections, commands,
IME preedit, scrolling, and diagnostics. The first proof of concept should use
one model and adapt it to the other, not merge both.

### 5. Full custom chrome and accessibility are not automatic

Floem provides window helpers, drag/resize views, focus navigation metadata,
menus, and platform integration. That is useful, but Functors should verify
native accessibility behavior, system window operations, IME behavior, and
custom-chrome requirements on each target. A custom-painted title bar remains a
platform feature, not just another rectangle in a Vello scene.

## Recommended Proof of Concept

Build a separate `floem_poc` example or temporary branch with this shape:

```text
Functors Model + Message + update
                 |
                 v
       adapter-owned presentation state
                 |
                 v
        Floem root view and widgets
                 |
                 v
       Floem Vello renderer backend
                 |
                 v
               WGPU
```

The proof of concept should contain:

1. The existing Functors document model and one read-only or minimally editable
   buffer.
2. A Floem window with menu, command surface, editor viewport, scrollbar, and
   status line.
3. Functors messages translated into Floem actions, with no document state
   duplicated in `RwSignal`s.
4. A shaped text line using Floem's Parley-based layout and Vello glyph path.
5. Cursor, selection, scrolling, resize, DPI change, and IME smoke coverage.
6. A comparison of frame time, memory, binary size, and dependency build cost
   against the current prototype.
7. A decision record that names which layer owns layout, focus, diagnostics,
   theme tokens, and window lifecycle.

The POC should not include notebooks, HTML cells, agents, plugins, or custom
frameless chrome. Those features would obscure whether the basic integration is
sound.

## Decision Criteria

Prefer Floem as the Functors application layer if the POC demonstrates all of
the following:

- Functors' model remains the single authority for document and workspace
  semantics.
- Floem's view adapter can update without rebuilding the entire UI tree for
  every editor change.
- Parley/Floem text layout keeps cursor, selection, fallback, emoji, and IME
  behavior correct for the supported targets.
- The Vello backend builds with a dependency set acceptable for Functors and
  presents reliably on the primary Windows GPU path.
- Widget focus, keyboard navigation, menus, dialogs, scrolling, and theme
  updates are simpler than maintaining equivalent Functors code.
- Visual output, diagnostics, and notebook output-surface boundaries remain
  controllable and testable.
- The maintenance cost of Floem upgrades is acceptable for the project's
  release and platform plans.

Keep the current custom Functors renderer path if the POC requires duplicated
state, cannot preserve deterministic editor behavior, or makes WGPU/winit
upgrades and platform debugging materially harder than the UI productivity
benefit justifies.

## Suggested Direction

Treat Floem as a candidate **application and widget layer on top of Functors'
domain and MVU core**, with its Vello backend as the first rendering
implementation to evaluate. Do not treat Floem as a replacement for the
Functors model, notebook engine, execution layer, plugin system, or HTML
security boundary.

The immediate next step is the small POC above. In parallel, keep the existing
Vello migration plan and use Floem's renderer and text implementation as a
reference implementation for surface targets, glyph runs, cache lifetimes,
clipping, and frame capture. This gives Functors a reversible path: the project
can adopt Floem's UI layer if it proves its value, while the Vello renderer
knowledge remains useful either way.

## References

- Floem workspace inspected at `D:\dev\downloads\floem-main`
- [Functors rendering architecture](../rendering_architecture.md)
- [Functors roadmap](../roadmap.md)
- [Vello rendering decision](../decisions/0006-vello-rendering-and-composition.md)
- [Functors architecture](../architecture.md)
