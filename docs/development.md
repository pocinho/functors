# Development Guide

This guide describes the local workflow for developing Functors.

## Prerequisites

Install:

- Rust stable with the `rustfmt` and `clippy` components
- A platform-supported graphics environment for WGPU
- Git

The project is a Rust 2024 binary crate. The exact dependency versions are
recorded in `Cargo.toml` and should be updated there rather than duplicated in
this document.

## Common Commands

Run these checks before opening a pull request:

```text
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Format the workspace when needed:

```text
cargo fmt
```

Run the desktop application:

```text
cargo run
```

A change involving window lifecycle, input routing, rendering, or visible
geometry also needs a live smoke test. Confirm that the window starts, accepts
input, redraws after edits, resizes correctly, and closes cleanly.

## Development Loop

1. Read `docs/architecture.md` and the relevant ownership guidance.
2. Identify the semantic message, model invariant, or effect boundary involved.
3. Add or update a headless test for deterministic behavior.
4. Implement the smallest change within the owning module.
5. Run formatting, compilation, tests, and any required GUI smoke test.
6. Update the owning documentation when behavior or boundaries change.

## Documentation Changes

Use `docs/README.md` to find the owning reference. Update architecture or
module ownership documentation when introducing a new dependency direction,
subsystem, or lifecycle boundary. Record significant alternatives and
tradeoffs in `docs/decisions/`.

## Troubleshooting

### Compilation or dependency problems

Run `cargo check` first. Inspect the compiler error at the owning module before
changing architecture or adding dependencies.

### Formatting or lint failures

Run `cargo fmt` for formatting. Use the specific Clippy diagnostic to decide
whether code should be simplified, annotated, or intentionally allowed.

### Window or renderer failures

Run `cargo run` from the repository root and check the platform and WGPU
initialization path. Keep platform handles and GPU types out of model, MVU, and
backend-neutral view code.

### Stale build artifacts

Use `cargo clean` only when dependency or incremental-build state is suspected;
it is not part of the normal development loop.
