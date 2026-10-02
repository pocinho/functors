# Testing Strategy

Tests should protect the ownership boundaries that make Functors deterministic,
portable, and maintainable.

## Test the Narrowest Boundary

Prefer tests close to the code that owns the behavior:

- **Model and document:** positions, ranges, selections, editing, cursor validity, and invariants
- **MVU:** semantic messages, transitions, command generation, and state changes
- **Input:** platform event mapping into semantic messages
- **View and layout:** frame descriptions, hit testing, clipping, geometry, and selection rectangles
- **Serialization:** stable workspace, document, notebook, and configuration contracts
- **Effectful subsystems:** error classification, cancellation, shutdown, and lifecycle behavior

Use `tests/` for public contracts or integration paths that are awkward to test
inside a module.

## Headless Tests First

Pure model, update, input, and layout behavior should be testable without a
window, GPU, filesystem, clock, or network. These tests should be the primary
regression suite for editor behavior.

Rendering tests should prefer deterministic frame descriptions and renderer
inputs. Physical GPU pixel snapshots are optional because drivers, fonts,
antialiasing, and hardware can make them brittle.

## Validation Matrix

| Change | Required validation |
| --- | --- |
| Model, MVU, input, or layout | `cargo fmt --check`, `cargo check`, `cargo test` |
| Rendering or visible geometry | Formatting, check, tests, and `cargo run` smoke test |
| Window lifecycle or platform input | Formatting, check, tests, and `cargo run` smoke test on the target platform |
| Dependencies or build configuration | Formatting, check, tests, and relevant platform smoke tests |
| Documentation only | Markdown review and `git diff --check` |

Run Clippy when preparing a pull request or when changing shared Rust APIs:

```text
cargo clippy --all-targets --all-features -- -D warnings
```

## Test Design

Prefer semantic assertions over incidental implementation details. Test both
successful transitions and invalid or boundary inputs. Useful cases include:

- empty and single-line documents;
- Unicode scalar and grapheme-sensitive positions;
- selections spanning lines;
- resize and surface-loss behavior;
- failed file or executor operations;
- cancellation and shutdown of background work.

Use property-based tests when range, cursor, serialization, or editing state
has enough combinations that example tests no longer provide confidence.
