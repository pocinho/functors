# 0005: Prefer Skia for Cross-Platform Text Rendering

- Status: Superseded by 0006
- Date: 2026-10-02

## Context

The current WGPU renderer uses a temporary bitmap fallback and a platform-specific
fontdue path. That is sufficient to prove the MVU and frame contracts, but it
does not provide production text shaping, reliable font fallback, native color
emoji, or the typography quality expected from a programming editor.

Functors must support Windows first while remaining viable on Android, Linux,
macOS, and other future platforms. The editor also needs a visual diagnostics
layer for programming work: errors, warnings, informational hints, and related
source ranges should be rendered without contaminating the document model with
backend-specific resources.

## Decision

Prefer Skia as the production text-rendering and shaping backend. The intended
font policy is platform-provided primary and fallback fonts, with Windows using
Consolas followed by Segoe UI Emoji. Other platforms should use their native
font managers and fallback chains rather than hard-coded Windows paths.

Keep the existing ownership boundary:

```text
Model diagnostics -> backend-neutral FrameDescription -> Skia text/layout -> WGPU composition
```

Diagnostics remain semantic data and frame geometry in the model/view layers.
Skia owns shaping, glyph selection, font fallback, emoji handling, and text
rasterization. WGPU remains responsible for composition and presentation until
there is a reason to replace that boundary.

## Alternatives Considered

### Keep fontdue as the production renderer

Rejected as the primary direction because it does not provide the desired
cross-platform shaping and color-font behavior without substantial surrounding
infrastructure.

### Replace the entire WGPU renderer with Skia

Deferred. Skia can render the full surface, but using it first for text keeps
the current MVU and frame contracts stable and limits the platform migration.

### Use a WGPU-native text stack only

Still viable, especially with a shaping library, but Skia is preferred for the
initial production text path because it has mature platform font integration and
stronger typography and emoji support.

## Consequences

- Text layout must eventually expose shaped advances rather than assuming one
  fixed advance per Unicode scalar.
- Font loading becomes a platform resource boundary, not a view concern.
- Diagnostics can share the same shaped text and geometry pipeline as document
  text while remaining testable without a GPU.
- The current bitmap/fontdue path remains a transitional fallback until Skia is
  integrated and validated on the primary desktop and Android targets.
- Skia introduces native build dependencies and increases package/build size.

## Review Triggers

Revisit this decision when:

- a supported platform cannot provide the required Skia font backend;
- a WGPU-native text stack demonstrably provides equivalent shaping and emoji
  behavior with lower maintenance cost;
- diagnostics require a separate accessibility or GPU overlay architecture;
- Android packaging constraints require a different native integration strategy.
