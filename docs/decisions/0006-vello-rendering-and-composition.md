# 0006: Prefer Vello for Rendering and Composition

- Status: Accepted
- Date: 2026-10-03
- Supersedes: 0005

## Context

Functor needs a single cross-platform rendering path for editor geometry,
diagnostics, notebook surfaces, and future custom application chrome. The
current renderer keeps `FrameDescription` backend-neutral, but its text path
uses CPU rasterization and pixel quads. Vello is a better long-term fit for
the existing WGPU boundary: the view can remain deterministic while a
renderer-owned scene builder composes the complete frame.

## Decision

Prefer Vello over WGPU as the production vector composition and text-rendering
backend. Keep the ownership boundary:

```text
Model -> backend-neutral view -> FrameDescription -> Vello scene builder -> WGPU
```

The migration is staged:

1. Pin a Vello version and prove surface rendering with the current WGPU and
   winit versions.
2. Replace the existing geometry pass while keeping `FrameDescription`
   inspectable and deterministic.
3. Replace the pixel-quad text bridge with Vello's validated text path and
   shape-aware layout data.
4. Add scene layers for diagnostics, panels, notebook cells, and overlays as
   their backend-neutral contracts are defined.

## Boundaries

Vello is a rendering and composition engine, not the application's complete
widget, accessibility, HTML, or window-management solution. Keyboard focus,
hit testing, semantic controls, webview isolation, platform window behavior,
and custom chrome remain explicit Functors boundaries. A single Vello scene
must not cause those concerns to leak into `Model` or `view` prematurely.

Vello text API details, font fallback, color fonts, clipping, render-target
formats, and performance must be verified against the pinned crate version on
Windows before removing the transitional Skia and fontdue dependencies.

## Consequences

- `FrameDescription` remains the headless-testable contract between view and
  renderer.
- The renderer gains a scene-building phase and Vello GPU resources.
- Theme tokens and shape-aware text metrics can be introduced without putting
  Vello types in domain or MVU code.
- The dependency and build footprint changes, and Vello's compute-oriented
  pipeline must be validated on supported GPU backends.
- The old Skia decision remains useful as comparison history but no longer
  defines the production direction.

## Review Triggers

Revisit this decision if Vello cannot provide the required text quality,
platform coverage, color-font behavior, or frame-time budget, or if a supported
platform requires a separate rendering path.