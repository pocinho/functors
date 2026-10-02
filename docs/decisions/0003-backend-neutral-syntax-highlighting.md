# 0003: Keep Syntax Highlighting Backend-Neutral

- **Status:** Accepted
- **Date:** 2026-10-02

## Context

Phase 1 requires useful initial syntax highlighting for Rust and Markdown, but
syntax recognition and style decisions should remain testable without a window
or GPU. The renderer currently uses a temporary bitmap font path and should not
become the owner of language rules.

The application does not yet have file or workspace metadata from which to
select a language. The initial implementation therefore uses conservative
line-level auto-detection when no extension is available and allows the
document language to be selected explicitly by future file loading and
workspace code.

## Decision

Implement syntax highlighting as a pure module that maps a line of text and a
`SyntaxLanguage` to semantic highlight spans. The view converts those spans
into backend-neutral styled text runs, and the renderer only consumes the run
text and color.

Rust and Markdown support is intentionally minimal. Incomplete or unrecognized
text falls back to plain rendering, and highlighting must never block editing
or make malformed source uneditable.

## Alternatives Considered

### Put token recognition in the renderer

Rejected because it would couple language behavior to WGPU and make headless
tests difficult.

### Add a full parser or LSP immediately

Deferred because Phase 1 only needs basic visual guidance. Parser and LSP
integration can replace the implementation behind the span contract later.

### Infer the language from arbitrary text

Rejected because heuristics are surprising. File and workspace metadata should
select the language when those boundaries exist.

## Consequences

- Highlighting spans can be tested deterministically without a GPU.
- The renderer remains a consumer of frame data rather than a language engine.
- The initial lexer is intentionally incomplete and is not a correctness tool.
- Auto-detection is intentionally conservative and may leave ambiguous text
  unstyled until file/workspace integration adds reliable metadata.
- A future parser, tree-sitter integration, or LSP can replace the lexer while
  preserving the view and renderer boundary.

## Review Triggers

Revisit this decision when:

- file extensions and workspace metadata become available;
- syntax highlighting must handle multiline constructs;
- diagnostics or semantic tokens are introduced;
- production text shaping requires richer style and font information.
