# Phase 1 TODO: First Working MVU Window

This note turns the Phase 1 milestone plan into an executable sequence for the
Rust rewrite. The immediate target is a small desktop window that runs a pure
MVU loop, renders a deterministic frame, accepts text input, and can be tested
without requiring a GPU or a real window.

The older F# implementation at `D:\dev\projects\pp\Functor` is the reference
for boundaries, not an API to copy. Its useful lessons are:

- Keep the domain model and update function independent of Avalonia.
- Translate platform events into semantic editor messages before updating state.
- Keep rendering as `slice -> layout -> RenderingModel`; the backend only draws.
- Keep viewport offsets, cursor positions, selections, and dimensions explicit.
- Test coordinate conversion and Unicode behavior at the layout boundary.

## Phase 1 Definition of Done

Phase 1 is complete when all of these are true:

- `cargo run` opens a desktop window on the primary development platform.
- The window clears to a model-provided background and renders a visible title
  or text line.
- Resize, close, keyboard, text-input, and redraw events reach the MVU layer.
- The model contains a text buffer, cursor, viewport, and dirty state.
- Typing, backspace, enter, and left/right/up/down movement update the model.
- The view produces a backend-neutral frame description; it does not draw.
- A renderer consumes that frame description and draws background, gutter,
  text, selection, and cursor.
- Core update, viewport slicing, layout, and input mapping have deterministic
  unit tests.
- The application remains responsive while no asynchronous command is running.

The first window is deliberately not a complete editor. File dialogs,
workspace discovery, syntax highlighting, notebooks, plugins, and HTML output
remain follow-up work after this loop is observable and testable.

## Target Module Shape

Start with one binary crate and split modules only along ownership boundaries:

```text
src/
  main.rs                 # process entry point and platform shell startup
  app.rs                  # App model, message routing, command queue, redraw policy
  mvu/
    mod.rs
    message.rs            # semantic messages, not winit types
    update.rs             # pure Model + Message -> (Model, Commands)
    command.rs            # effects requested by update; no direct platform calls
  model/
    mod.rs
    document.rs           # minimal text document and dirty state
    cursor.rs             # line/column position and movement rules
    viewport.rs           # size and scroll offsets
  view/
    mod.rs
    frame.rs              # backend-neutral frame primitives and text runs
    layout.rs             # visible-line slicing and pixel geometry
  platform/
    mod.rs
    input.rs              # winit event -> Message mapping
    window.rs             # winit event loop and window lifecycle
  rendering/
    mod.rs
    wgpu_renderer.rs      # GPU resource setup and frame drawing
```

Keep these boundaries stable even if the initial implementation remains in
fewer physical files. The important split is:

```text
winit event -> semantic Message -> pure update -> Model -> pure view/layout
           -> FrameDescription -> renderer -> redraw request
```

## Ordered Implementation Steps

### 1. Establish a buildable shell

- [x] Confirm the supported desktop target and Rust toolchain.
- [x] Add `winit` 0.30.13 as the initial desktop windowing dependency and add
  `wgpu` 30.0.1 with `pollster` for the first clear pass. Add `ropey` when the
  buffer work begins.
- [x] Keep `main.rs` as a thin composition root.
- [x] Run `cargo fmt --check` and `cargo check` before introducing rendering.
- [x] Record the chosen crate version and platform assumptions in the main
  architecture note.

**Checkpoint:** the project compiles with a windowing dependency and has a
single place responsible for creating the window/event loop.

### 2. Define the minimal pure model

Create a model that can drive the first visible frame:

```rust
struct Model {
    document: Document,
    cursor: Cursor,
    viewport: Viewport,
    mode: EditorMode,
    needs_redraw: bool,
}
```

Use a small `Vec<String>` buffer for the first vertical/horizontal layout if it
keeps the code straightforward; replace it with `ropey::Rope` once editing
operations are covered. Do not let the rendering or window code depend on the
buffer representation.

- [x] Define `Position { line, column }` and document bounds invariants.
- [x] Define viewport width/height and vertical/horizontal offsets.
- [x] Define insert-mode behavior as the default first mode.
- [x] Use Unicode scalar offsets for internal columns; convert only at
  platform/API boundaries. Add grapheme-aware movement when the text layout
  backend is introduced.
- [x] Add model constructors with a deterministic initial document.

**Checkpoint:** model construction has no filesystem, window, clock, or GPU
dependency.

### 3. Define messages and pure update

Use semantic messages so `update` can be tested without `winit`:

```rust
enum Message {
    WindowResized { width: u32, height: u32 },
    RedrawRequested,
    KeyPressed(Key),
    TextInput(String),
    PointerPressed { position: Position },
    CloseRequested,
}
```

The exact enum can evolve, but platform event types should not leak into the
domain. `update` should return the next model plus declarative commands, for
example `Command::RequestRedraw` or `Command::Exit`.

- [x] Implement resize with non-negative offsets and stored dimensions.
- [x] Implement text insertion, newline, backspace, and cursor movement.
- [x] Clamp cursor positions after every edit.
- [x] Mark the document dirty only when the text changes.
- [x] Make repeated resize, redraw, and unsupported-key messages harmless.
- [x] Add tests for each transition and for cursor behavior at document edges.

**Checkpoint:** a test can feed messages to `update` and assert the complete
next model without starting a window.

### 4. Build the backend-neutral view and layout

Follow the F# rendering boundary. The view/layout layer computes geometry but
does not call WGPU:

```rust
struct FrameDescription {
    viewport: Size,
    background: Color,
    line_numbers: Vec<LineNumber>,
    text_runs: Vec<TextRun>,
    selections: Vec<Rect>,
    cursors: Vec<Rect>,
}
```

- [x] Slice only visible lines from the document using viewport offsets.
- [x] Define fixed first-pass font metrics (`line_height`, `advance`) behind a
  `TextMeasurer` trait or configuration object.
- [x] Compute gutter width from line count and a minimum width.
- [x] Map positions to rectangles and pointer coordinates back to positions.
- [x] Keep horizontal scroll coordinates consistent for text, selection, and
  hit testing.
- [x] Define explicit draw ordering: background, gutter, highlights,
  selection, text, diagnostics, cursor. The current renderer implements the
  available layers in that order and leaves diagnostics empty when no model
  selection is present.
- [x] Add tests for empty buffers, final-line scrolling, clamping, and
  viewport slicing.
- [x] Add complete first-pass Unicode contract tests before adopting a rope or
  real font shaper: emoji, combining marks, tabs, and a cursor at the end of a
  line. These currently use one fixed advance per Unicode scalar.

**Checkpoint:** `view(model, metrics)` returns the same `FrameDescription` for
the same inputs and the result is inspectable in a unit test.

### 5. Open a clearable WGPU window

Use `winit` only for lifecycle and input, and `wgpu` only for the renderer.
Keep initialization explicit and asynchronous where the APIs require it.

- [x] Create the event loop and a resizable window.
- [x] Create a WGPU instance, surface, adapter, device, and queue.
- [x] Configure the surface from the current window size and handle zero-sized
  surfaces during minimize/resize.
- [x] Add a minimal clear pass that clears the surface with the model’s
  background color.
- [x] Handle WGPU 30 surface loss/outdated statuses by reconfiguring; skip
  timeout, occluded, and validation frames.
- [x] Request redraw after state changes and in `RedrawRequested` render one
  frame.
- [x] Exit cleanly on `CloseRequested`.

**Checkpoint:** `cargo run` opens a stable clearable window, resizes without
panic, and closes without leaving a stuck process. The WGPU clear-pass smoke
run completed cleanly on the primary Windows development platform.

### 6. Render the first visible editor frame

Start with geometry that proves the entire pipeline, not a full text engine:

- [x] Draw the background and line-number gutter.
- [x] Draw line numbers using the temporary `font8x8` fixed-width glyph
  strategy.
- [x] Draw the initial document text with the temporary bitmap glyph strategy.
- [x] Draw a cursor rectangle at the model cursor position.
- [x] Render selection rectangles when the model contains a selection. The
  model stores an optional anchor/focus range and layout emits clipped
  per-line rectangles; pointer drag now updates the anchor/focus range.
- [x] Keep the WGPU renderer consuming only `FrameDescription` and render
  configuration.
- [x] Ensure the first frame is visible even with an empty document through the
  background, gutter, and cursor layers.

The text backend is a deliberate decision point. Evaluate `cosmic-text`,
`glyphon`, Vello, and a simpler first-pass atlas against Unicode shaping,
WGPU compatibility, and maintenance cost. Do not make the domain model depend
on whichever option is selected.

**Checkpoint:** the visible window demonstrates model-driven background, text,
cursor, and resize behavior.

### 7. Connect real input

- [x] Translate `winit` keyboard events into semantic `KeyPressed` messages.
- [x] Use text/IME input events for inserted text instead of treating key codes
  as characters.
- [x] Map arrows, backspace, enter, home, and end first.
- [x] Map pointer coordinates through layout to a cursor position.
- [x] Request redraw after input and resize state transitions.
- [x] Verify that focus loss and modifier changes do not leave stale state.
  Pointer drag state and Shift state are cleared on focus loss; Shift
  navigation extends selections through semantic modifier messages.

The F# adapter is a useful model here: platform-specific keys are normalized
before keymap/editor logic sees them. Preserve that separation in Rust.

**Checkpoint:** typing and navigation visibly change the text/cursor, and a
pointer click moves the cursor to the expected line and column.

### 8. Add focused tests and manual checks

Automated tests should remain platform-independent wherever possible:

- [x] `Model::initial` produces the expected empty/seed document.
- [x] `update` handles insertion, deletion, movement, resize, and dirty state.
- [x] Layout slices only visible lines and clamps offsets.
- [x] Position hit testing clamps outside the document.
- [x] Text runs cover the source line without gaps or overlaps.
- [x] Unicode and tab columns preserve the chosen coordinate contract.
- [x] A frame can be generated without a GPU.

Manual smoke test on the primary desktop target:

1. Run `cargo run`.
2. Confirm the window opens with the expected background and text.
3. Resize and minimize/restore the window.
4. Type text, press enter, delete text, and move in all four directions.
5. Click inside and outside the text bounds.
6. Close the window and confirm the process exits.

Run at each checkpoint:

```text
cargo fmt --check
cargo check
cargo test
```

Add a GPU/integration check only after the headless tests are stable; CI must
not require a physical display or adapter for the pure core tests.

## Explicitly Deferred

Do not expand the first window into these features until the loop above works:

- file open/save and workspace discovery
- syntax highlighting and LSP diagnostics
- tabs, panels, split views, and command palette
- notebook cells or HTML output
- plugin loading and agent integration
- cross-platform mobile/web shells
- production text shaping, theming, and animation systems

These belong after the first working window because they add commands,
asynchronous effects, or rendering layers that are difficult to diagnose while
the core event/update/render path is still unproven.

## Suggested Completion Order

1. Pure model, messages, update, and tests.
2. Pure frame description, slicing, layout, and tests.
3. Winit window with a WGPU clear pass.
4. Frame drawing for text, gutter, and cursor.
5. Input translation and text editing.
6. Unicode/layout hardening and platform smoke test.
7. Documentation update and only then file persistence.
