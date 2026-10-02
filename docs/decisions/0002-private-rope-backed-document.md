# 0002: Use a Private Rope-Backed Document

- **Status:** Accepted
- **Date:** 2026-10-02

## Context

The initial editor used `Vec<String>` directly as the document representation.
That was sufficient for the first window, but it exposed storage details to
layout and tests and made future storage changes part of the surrounding API.
The Phase 1 roadmap requires a production-oriented text-buffer boundary.

The editor currently defines columns as Unicode scalar offsets. Grapheme-aware
movement and production text shaping remain future work.

## Decision

Use `ropey::Rope` as the private storage representation inside `Document`.
Expose document operations and line-oriented accessors rather than the Rope or
its internal collections. Preserve the current scalar-column contract for
positions, editing, hit testing, and layout.

Document mutations continue to own cursor clamping, line joins, range deletion,
and dirty-state changes. The MVU, input, view, and renderer layers do not depend
on Rope APIs.

## Alternatives Considered

### Keep `Vec<String>` as the public representation

Rejected because callers would depend on storage details and future changes
would require broad API changes.

### Introduce a custom text buffer immediately

Rejected because it would add maintenance cost before the project has measured a
need for custom storage behavior.

### Use a piece table

Deferred. It may become appropriate if workload measurements or persistence and
undo requirements make its tradeoffs preferable to Rope.

## Consequences

- Document storage can evolve without changing MVU, layout, or renderer APIs.
- Multiline edits and line-oriented reads remain available through `Document`.
- Rope introduces an external dependency and should be covered by boundary
  tests rather than leaking into application code.
- The current scalar-column behavior is not a complete grapheme-aware text
  editing model.
- Large-document and editing-workload measurements remain future work.

## Review Triggers

Revisit this decision when:

- grapheme-aware editing and shaping are implemented;
- undo/redo transaction storage is designed;
- large-document profiling identifies a storage bottleneck;
- persistence or concurrent views require different text-buffer semantics.
