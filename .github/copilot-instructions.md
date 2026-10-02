# Functors Repository Instructions

Read `docs/architecture.md` before making structural changes. The current Phase 1 implementation is intentionally compact and flat; do not reorganize it into the aspirational directory tree unless the relevant ownership boundary has grown enough to justify the split.

## Architecture

Preserve this dependency direction:

```text
winit -> input adapter -> semantic Message -> pure update -> Model -> view/layout
      -> FrameDescription -> WGPU renderer -> redraw request
```

- Keep `main.rs` as the composition root and platform event loop.
- Keep `input.rs` responsible for translating winit events into semantic messages.
- Keep `mvu.rs` pure and platform-independent.
- Keep document invariants, cursor positions, selections, ranges, and editing operations in `model.rs`.
- Keep `view.rs` deterministic and backend-neutral; it must not call WGPU or inspect window handles.
- Keep `renderer.rs` consuming `FrameDescription`, not the domain model.
- Do not introduce winit, WGPU, filesystem, clock, or window-handle dependencies into domain or view code.

## Change Discipline

- Prefer the smallest change that preserves the existing ownership boundaries.
- Add headless tests for model, update, input mapping, and layout behavior before GUI changes.
- For changes to window lifecycle, input routing, rendering, or visible geometry, run a live `cargo run` smoke test after `cargo fmt --check`, `cargo check`, and `cargo test`.
- Treat the `font8x8` renderer as temporary. Production text shaping, grapheme-aware movement, fallback fonts, persistence, dialogs, plugins, and diagnostics require explicit future boundaries.
- Keep documentation synchronized with the actual flat module layout and record refactor triggers instead of implying aspirational files already exist.
- Never revert unrelated user changes in the worktree.
