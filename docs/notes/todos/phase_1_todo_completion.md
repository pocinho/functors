# Phase 1 Completion TODO: Core Editor Engine

This note closes the gap between the completed first MVU window and the broader
Phase 1 definition in `docs/ROADMAP.md`.

The first-window implementation is complete in
`docs/notes/todos/phase_1_todo_mvu_window.md`. The roadmap's Phase 1 milestone
is not complete yet because it also requires a production-oriented text buffer,
basic syntax highlighting, workspace discovery, and file open/save behavior.

## Phase 1 Completion Definition

Phase 1 is complete when Functors can:

- run the deterministic MVU editor loop;
- render and edit a document in a desktop window;
- use the selected text-buffer representation behind a document API;
- provide basic Rust and Markdown syntax highlighting;
- open a folder as a workspace and list its files;
- open and save a file with clear dirty-state and error behavior;
- preserve the platform/domain/view ownership boundaries;
- pass headless tests and the required desktop smoke test;
- document the implemented contracts and known limitations.

Notebook execution, multi-language runtimes, HTML output, agents, plugins, and
advanced workspace UX remain later roadmap phases.

## Baseline Already Complete

These items are inherited from the first-window milestone and should not be
reimplemented unless a focused defect is found:

- [x] Winit window lifecycle and clean shutdown.
- [x] WGPU surface, device, queue, resize, redraw, and surface recovery path.
- [x] Semantic input messages separated from platform events.
- [x] Pure deterministic MVU transitions and declarative redraw commands.
- [x] Document editing for insertion, newline, backspace, delete, and selection replacement.
- [x] Cursor movement, pointer hit testing, pointer drag selection, and Shift selection.
- [x] Backend-neutral frame description and deterministic layout tests.
- [x] Gutter, line numbers, text, selection, and cursor rendering.
- [x] Unicode scalar-column contract and first-pass tests for Unicode and tabs.
- [x] Formatting, compilation, headless tests, and primary-platform GUI smoke testing.

The current implementation remains intentionally compact and uses a flat module
layout. Split modules only when the ownership boundaries described in
`docs/module_ownership.md` have grown enough to justify the change.

## Ordered Remaining Work

### 1. Stabilize the Document Storage Boundary

Replace the provisional line storage with the selected text-buffer
representation required by the roadmap, likely a rope or another structure
chosen through measurement.

- [x] Choose Rope as the initial representation and record the tradeoffs in an
      ADR.
- [x] Add the dependency only after the document API and coordinate semantics
      are clear.
- [x] Keep storage details behind the document API.
- [x] Preserve positions, ranges, selections, insertion, deletion, line joins,
      and cursor clamping.
- [x] Define Unicode scalar columns as the current coordinate unit and keep
      grapheme-aware movement as a future boundary.
- [x] Add tests for large documents, multiline edits, empty documents, Unicode,
      tabs, and edits at document boundaries.
- [x] Measure common editing and layout operations before optimizing further;
      see `docs/notes/performance_baseline.md`.

**Completion check:** the model, view, and input layers do not depend on the
chosen storage type, and document invariants remain centralized.

### 2. Add Minimal Syntax Highlighting

Introduce syntax highlighting as a pure document-to-view concern. The first
implementation should be deliberately small and useful rather than a complete
language-server system.

- [x] Define a backend-neutral syntax span and text-style representation.
- [x] Keep highlighting independent of WGPU and platform event types.
- [x] Add minimal Rust highlighting for common keywords, strings, comments,
      numbers, and punctuation.
- [x] Add minimal Markdown highlighting for headings, emphasis, code spans,
      links, and fenced code markers.
- [x] Define behavior for incomplete or invalid source text.
- [x] Keep highlighting failures non-fatal and preserve plain-text rendering.
- [x] Add deterministic tests for spans and plain-text fallback.
- [x] Render styles through `FrameDescription` without coupling the model to
      renderer-specific resources.
- [x] Select Rust or Markdown from file/workspace metadata when file loading is
      implemented.

**Completion check:** Rust and Markdown documents have useful basic styling,
and unrecognized text remains readable and editable.

### 3. Introduce the Workspace Boundary

Add a workspace model and filesystem adapter without allowing filesystem access
to enter the document, MVU, or view layers.

- [x] Define workspace identity and root-directory validation.
- [x] Define semantic messages and commands for opening a workspace.
- [x] Provide a native folder picker action through `Ctrl+O`.
- [x] Implement directory listing through a platform or filesystem boundary.
- [x] Represent files and directories with stable, testable domain data.
- [x] Handle inaccessible paths, invalid roots, and changing directory contents.
- [x] Add a deterministic workspace fixture for headless tests.
- [x] Decide that file watching remains deferred from Phase 1.
- [x] Record workspace ownership and persistence assumptions in an ADR if they
      affect future notebook or plugin features.

**Completion check:** a user can open a folder and obtain a model-driven file
listing without the view reading the filesystem directly.

### 4. Add File Open and Save

Connect the document model to files through explicit effects and semantic
results.

- [x] Define open and save messages and commands.
- [x] Define success and failure messages; picker cancellation is a no-op.
- [x] Decode and encode text with an explicit UTF-8 initial encoding policy.
- [x] Preserve the document dirty state across edits, successful saves, and
      failed saves.
- [x] Handle missing files, permission failures, invalid data, and external
      modifications without corrupting the in-memory document.
- [x] Use a temporary-file replacement strategy appropriate to the supported
      platforms.
- [x] Keep file I/O out of `update`, `model`, `view`, and `renderer`.
- [x] Add tests using temporary directories or an injected filesystem boundary.
- [x] Add a manual smoke test for open, edit, save, close, and reopen.

**Completion check:** file operations are recoverable effects, errors are
visible to the user, and a failed save never silently clears dirty state.

### 5. Integrate the Minimal Editor Workflow

Connect workspace and file operations to the existing editor shell without
turning the composition root into a second domain model.

- [x] Add a semantic way to select or activate a file through `Ctrl+P`.
- [x] Show the active file name or path in the window title.
- [x] Display a clear dirty indicator in the window title.
- [x] Route workspace and file results back through the MVU message loop.
- [x] Keep transient dialogs, platform handles, and filesystem errors at the
      application/platform boundary.
- [x] Add a minimal empty, loading, error, and active-document state.
- [x] Ensure redraw requests occur after workspace, open, save, and highlight
      state changes.

**Completion check:** the app can open a workspace, choose a file, edit it,
save it, and visibly reflect the current state.

### 6. Complete Regression and Smoke Coverage

- [x] Run `cargo fmt --check`.
- [x] Run `cargo check`.
- [x] Run `cargo test`.
- [x] Run `cargo clippy --all-targets --all-features -- -D warnings`.
- [x] Test model, update, input, layout, highlighting, workspace, and file
      boundaries without a window or physical GPU.
- [x] Run `cargo run` on the primary development platform.
- [x] Verify startup, resize, minimize/restore, text editing, selection,
      workspace open, file open, save, error handling, and clean shutdown.
- [x] Confirm that no new domain or view dependency points to Winit, WGPU,
      filesystem APIs, clocks, or platform handles.

## Documentation Completion

- [x] Update `docs/architecture.md` with the implemented workspace and file
      boundaries.
- [x] Update `docs/module_ownership.md` when new modules become real.
- [x] Add ADRs for text storage, syntax highlighting, workspace persistence,
      or encoding choices when they become expensive to reverse.
- [x] Update `docs/testing.md` with new boundary and integration coverage.
- [x] Update the mdBook only for user-facing workflows and capabilities.
- [x] Update `docs/future.md` by replacing completed deferred entries with links
      to the new subsystem documents.
- [x] Update `docs/ROADMAP.md` to mark Phase 1 complete after all required
      roadmap items and the validation checklist pass.

## Usability Extension Before Phase 2

The core checklist above is complete, but the editor is not yet being treated
as Phase 2-ready. The following usability work is the current gate:

- [x] Add semantic vertical and horizontal scrolling.
- [x] Render visible scrollbar tracks and thumbs.
- [x] Add Unicode bitmap coverage and visible unsupported-glyph fallback.
- [x] Add a clickable menu strip and `Ctrl+K` command bar shell.
- [x] Add a settings command and visible settings surface.
- [x] Replace bitmap text with Consolas rasterization and Segoe UI Emoji fallback.
- [ ] Add scrollbar thumb dragging and finish menu command behavior.

## Explicitly Outside Phase 1 Completion

Do not block Phase 1 on these later capabilities:

- notebook cells or reactive execution;
- multi-language executors;
- HTML rendering and interactive outputs;
- agent workflows or Copilot integration;
- third-party plugin loading;
- tabs, split views, command palette, or advanced theming;
- production-grade LSP diagnostics;
- cross-platform release certification.

Those capabilities should receive their own completion notes when their
respective roadmap phases begin.
