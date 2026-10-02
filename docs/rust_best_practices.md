# Rust Best Practices for Functor

***Ensuring Functor remains deterministic, modular, and maintainable as it grows.***

This document defines the preferred Rust practices for Functor as the project
expands from a compact desktop editor into a code editor, reactive notebook,
agent workspace, and extensible runtime.

These are target-state guidelines. They describe the architecture that future
features should move toward; they do not require premature abstraction or
module splitting before a clear ownership boundary exists.

When a local decision appears to conflict with this document, prefer the
smallest design that preserves ownership, deterministic domain behavior, and
clear effect boundaries. Record an intentional exception when the tradeoff is
not obvious.

---

## 1. Module Structure and Ownership

### Prefer folder-based modules for real subsystems

As a subsystem grows beyond one cohesive implementation file, prefer a folder
module with a small public facade and focused private modules:

```text
src/
    mvu/
        mod.rs       # public facade and composition of the subsystem
        message.rs
        model.rs
        update.rs
        command.rs
    rendering/
        mod.rs
        frame.rs
        layout.rs
        gpu.rs
    text_buffer/
        mod.rs
        document.rs
        cursor.rs
        edit.rs
```

The exact Rust module layout may use either `module.rs` with a `module/`
subdirectory or `module/mod.rs`. Choose one convention consistently within the
repository. The important property is ownership clarity, not the filename.

Do not create folders merely to mirror an aspirational architecture. A split is
justified when at least one of these is true:

- the module contains multiple independently testable responsibilities;
- different code has different dependency requirements;
- a subsystem has a stable public API and private implementation details;
- multiple contributors or features need a clear ownership boundary;
- the file has become difficult to navigate or review as one unit.

### Keep dependency direction explicit

Functor should preserve a one-way flow from platform events toward semantic
domain transitions and from domain state toward backend rendering:

```text
platform -> input adapter -> Message -> update -> Model -> view/layout
                                            -> FrameDescription -> renderer
```

The dependency direction should be enforced by module APIs and crate
boundaries where useful. Domain modules must not depend on window handles,
WGPU types, platform event types, clocks, or filesystem APIs.

A likely future boundary is:

```text
app/
platform/
input/
mvu/
model/
view/
rendering/
text_buffer/
workspace/
notebook/
agents/
plugins/
```

This is a destination for grown ownership boundaries, not a requirement that
all directories exist immediately.

### Keep public facades narrow

A subsystem's `mod.rs` or `lib.rs` should expose the stable concepts required
by its callers and keep implementation details private. Prefer a small set of
semantic types over re-exporting every internal helper.

Avoid god modules. Each module should answer one architectural question:

- What state and invariants does it own?
- What inputs does it accept?
- What outputs or effects does it produce?
- Which dependencies is it allowed to see?

Use descriptive names such as `cursor_position`, `render_surface`, and
`workspace_root` rather than abbreviated names whose meaning depends on local
context.

---

## 2. Ownership, Borrowing, and State

### Make ownership visible in APIs

Use borrowing when a function only observes data and ownership transfer when a
function consumes or replaces state:

```rust
fn layout(model: &Model, config: &ViewConfig) -> FrameDescription
fn update(model: Model, message: Message) -> Transition
```

Avoid unnecessary cloning, especially for documents, frame data, notebook
outputs, and agent payloads. Prefer moving owned values, borrowing read-only
values, or using purpose-specific shared ownership where the lifetime requires
it.

A clone is acceptable when it makes an ownership boundary explicit, prevents a
borrow from escaping, isolates concurrent work, or is cheaper and clearer than
complex lifetime plumbing. Explain non-obvious cloning decisions in the API
design or surrounding documentation rather than cloning defensively by habit.

### Prefer local mutation and immutable boundaries

Immutable data flow is valuable at subsystem boundaries, but Rust does not
require all implementation code to be immutable. Mutating an owned local
`Model`, document, or builder is idiomatic when the operation remains
encapsulated and its result is deterministic.

Prefer:

- pure transitions at the MVU boundary;
- immutable inputs to layout and rendering descriptions;
- exclusive ownership for mutable document edits;
- explicit synchronization only where data crosses threads or runtimes.

Avoid `Arc<Mutex<T>>` as a default architecture. Use message passing, task
ownership, channels, or immutable snapshots first. `Arc` without a mutex is
appropriate when shared lifetime ownership is required, such as a window or
resource handle held by a WGPU surface.

### Keep invariants with their owner

Cursor positions, selections, ranges, document lengths, workspace identity,
and notebook dependency state should be validated and modified by the module
that owns them. Do not duplicate invariant logic in input adapters, views, or
platform code.

---

## 3. Error Handling and Failure Boundaries

### Classify failures before choosing a response

Use `Result<T, E>` for recoverable failures such as file access, workspace
loading, plugin startup, executor failures, shader or surface creation, and
user-provided configuration.

Use explicit typed errors when callers can respond differently to different
failure classes. `thiserror` is appropriate when a subsystem benefits from
readable structured errors; it is not a mandatory dependency for every small
module.

At application startup, a composition root may use `expect` for a documented
invariant whose failure means the application cannot continue. The message
should identify the invariant. Do not use `unwrap` or `expect` to hide ordinary
runtime failures.

### Preserve error context across boundaries

Add context when crossing from one subsystem to another, while keeping the
underlying cause available for diagnostics. Errors from workspace discovery,
notebook execution, rendering, and agent orchestration should identify the
operation and relevant resource without exposing secrets or unstable internals.

### Separate user-facing failure from programmer error

A malformed document, unavailable GPU adapter, missing file, or failed agent
request should produce a recoverable error state or user-visible diagnostic.
An impossible internal invariant may justify a panic during development, but
such cases should be made explicit and converted to typed failure when the
application can reasonably continue.

---

## 4. MVU and Effect Boundaries

### Keep update deterministic and free of external side effects

The MVU update function should compute a transition from a model and message:

```rust
fn update(model: Model, message: Message) -> Transition
```

It may mutate the owned model internally for efficiency. “Pure” means that the
same model and message produce the same model changes and declared commands;
it does not mean that every local variable must be immutable.

Update code must not directly perform filesystem I/O, GPU work, network calls,
clock reads, process execution, or platform interaction.

### Represent effects explicitly

Commands describe effects that must be performed outside the pure update
function, for example:

- requesting a redraw;
- reading or writing a file;
- starting notebook or agent work;
- scheduling a timer;
- exiting the application;
- loading or saving workspace state.

A command should carry the data needed to perform its effect and should not
silently access global state. Async work should return a semantic message back
to the update loop rather than mutate the model from a background task.

Rendering is split into two stages: the pure view/layout code creates a
`FrameDescription`, and the platform/rendering shell consumes it. Constructing
the frame is not itself an effect; issuing GPU calls is an effect owned by the
renderer and lifecycle shell.

### Treat messages as semantic contracts

Input adapters translate platform events into semantic messages. The update
layer should not need to know whether a message came from Winit, a notebook
runtime, an agent, a plugin, or a test.

Messages should describe intent or completed results, not leak platform event
structures into the domain model.

---

## 5. Rendering and GPU Lifecycle

### Separate domain, layout, and GPU responsibilities

CPU-side code owns:

- document interpretation and editor state;
- text shaping and line layout;
- hit testing and selection geometry;
- deterministic `FrameDescription` generation;
- resource-independent display decisions.

GPU-side code owns:

- device, queue, surface, and pipeline setup;
- uploading buffers and textures;
- issuing draw calls;
- surface acquisition and presentation;
- handling resize, lost, outdated, and timeout conditions.

The renderer consumes a frame description or other explicit render input. It
must not reach into the domain model to discover editor state.

### Make lifecycle ownership explicit

GPU initialization does not need to be generally idempotent. Instead, define
which object owns initialization, when resources are created or recreated, and
how shutdown is handled. Repeated lifecycle events must not leak resources or
create duplicate active surfaces, but `Renderer::new` may remain a one-time
constructor.

Resize and surface-loss behavior should be explicit and testable where
possible. Platform event handling should decide when to recreate resources;
the renderer should report conditions it cannot resolve locally.

### Keep rendering responsive

Do not perform blocking file, network, process, notebook, or agent work on the
render thread or inside a redraw handler. Expensive layout or shaping should
be measured and moved behind an appropriate cache or task boundary when real
workloads require it.

---

## 6. Text, Documents, and Editing

### Keep text representation behind a document API

The document boundary should own storage details, positions, ranges, edits,
line structure, and cursor validity. Callers should not depend on whether the
implementation uses `Vec<String>`, a rope, a piece table, or another structure.

Choose a rope or piece table when document size, edit locality, concurrent
views, or measured performance justify it. Do not introduce one solely because
it is architecturally fashionable.

### Define coordinate semantics precisely

Document positions must specify whether columns represent bytes, Unicode scalar
values, grapheme clusters, or another unit. Cursor movement, selection, hit
testing, and rendering must use compatible semantics or perform explicit
conversion.

When production text editing is introduced, grapheme-aware movement, shaping,
fallback fonts, bidirectional text, tabs, and line endings should be handled
by dedicated boundaries rather than scattered special cases.

### Make editing transitions deterministic

Insertion, deletion, selection replacement, cursor movement, and undo/redo
should be deterministic and covered by focused tests. Undo/redo is required
when the feature is introduced, but it should not constrain the initial model
before its transaction and memory semantics are defined.

---

## 7. Async Work, Concurrency, and Agents

### Make async boundaries explicit

Keep synchronous state transitions synchronous. Spawn tasks only for work that
can block, is long-running, or naturally belongs to an async runtime. Define
how cancellation, shutdown, backpressure, and task failure are represented as
messages.

Do not let background tasks mutate shared application state directly. Return
results through channels or commands and apply them through the update loop.

### Isolate runtimes and plugins

Notebook executors, agent runtimes, and plugins should communicate through
stable traits or message contracts. They must not acquire arbitrary access to
editor internals, renderer state, or platform handles.

Plugin and agent APIs should define capability boundaries, resource limits,
shutdown behavior, and diagnostic propagation before supporting untrusted or
third-party implementations.

---

## 8. Testing Strategy

### Test pure behavior at the narrowest boundary

Prefer unit tests close to the owning module for:

- document invariants and editing operations;
- MVU transitions and command generation;
- input-to-message mapping;
- layout, hit testing, and selection geometry;
- serialization and workspace state transitions.

Use integration tests in `tests/` for public subsystem contracts and lifecycle
behavior that cannot be tested naturally inside a module.

### Prefer deterministic headless rendering tests

Test frame descriptions, layout data, resource decisions, and renderer input
without requiring a physical GPU whenever possible. GPU initialization and
presentation can have platform-specific integration tests or manual smoke
tests.

Snapshots are useful for stable serialized layout or frame-description data,
but physical GPU pixel snapshots are optional and should not be the primary
correctness test. They are sensitive to drivers, fonts, antialiasing, and
hardware.

### Test failure and lifecycle paths

For effectful subsystems, test unavailable files, failed executor requests,
invalid workspace metadata, adapter absence, surface loss, resize, shutdown,
and cancellation. Tests should verify both the error classification and the
state transition visible to the user.

### Keep tests maintainable

Use focused fixtures and semantic assertions. Avoid tests that duplicate
implementation details or depend on incidental field ordering. Property tests
are valuable for range, cursor, serialization, and edit invariants when the
state space becomes large.

---

## 9. Documentation and API Design

### Document public contracts and non-obvious invariants

Use rustdoc for public library APIs, stable subsystem facades, safety
requirements, error behavior, lifecycle assumptions, and non-obvious domain
invariants. A small binary does not need exhaustive documentation on every
private helper.

Keep examples runnable when examples are part of the public API. Cross-link
related concepts with rustdoc links such as [`Model`] and [`Message`].

### Keep architectural documentation current

When a feature introduces a new ownership boundary, update the architecture
documentation and record the dependency direction. Do not document aspirational
modules as though they already exist.

Public APIs should be designed around domain concepts rather than backend
implementation details. Avoid exposing WGPU, Winit, executor, or plugin
internals unless the caller is explicitly part of that boundary.

---

## 10. General Decision Rules

- Prefer composition and explicit data flow over inheritance-like frameworks.
- Prefer small, cohesive APIs over broad convenience facades.
- Prefer measured performance work over speculative optimization.
- Prefer deterministic pure logic before adding runtime effects.
- Prefer explicit lifecycle and shutdown behavior over hidden global state.
- Prefer capability-limited interfaces for plugins, agents, and executors.
- Prefer the simplest representation that satisfies current requirements.
- Prefer a documented exception over a misleading universal rule.

Future work should preserve the distinction between domain state, semantic
messages, pure view descriptions, platform lifecycle, and effectful runtimes.
That distinction is the foundation that lets Functor grow without turning every
new feature into a dependency on the window, renderer, or global application
state.
