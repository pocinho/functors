# Module Ownership

This document records which subsystem owns each concern and which dependency
direction future changes must preserve.

## Dependency Direction

```text
platform -> input adapter -> semantic Message -> pure update -> Model -> view/layout
                                                                  -> FrameDescription -> renderer
```

Dependencies should flow toward semantic domain concepts. Domain and
backend-neutral view code must not depend on Winit, WGPU, window handles,
filesystem APIs, clocks, or network clients.

## Current and Future Boundaries

| Boundary | Owns | Must not own |
| --- | --- | --- |
| `main.rs` / app shell | composition, event loop, lifecycle, dispatch, redraw requests | document invariants or layout policy |
| `input` | platform event translation into semantic messages | model mutation or rendering |
| `mvu` | messages, deterministic transitions, declared commands | Winit, WGPU, filesystem, network, or clock access |
| `model` / document | positions, ranges, selections, edits, cursor and document invariants | platform event types or GPU resources |
| `view` / layout | deterministic frame descriptions, hit testing, geometry | GPU calls, window handles, or domain-side mutation |
| `renderer` | WGPU resources, surface lifecycle, buffers, pipelines, presentation | direct inspection of the domain model |
| `workspace` | project roots, files, persistence, workspace metadata | editor rendering details |
| `notebook` | cells, dependency graph, execution state, outputs | platform lifecycle and direct UI mutation |
| `agents` | agent sessions, tool calls, streaming, cancellation, diagnostics | unrestricted access to application internals |
| `plugins` | capability-limited extension contracts and lifecycle | implicit global state or arbitrary platform access |

The future rows describe intended ownership boundaries. They should become
modules only when the corresponding feature has enough internal complexity to
justify the boundary.

## Extraction Triggers

Split a flat module into a folder module when one or more of the following is
true:

- it contains multiple independently testable responsibilities;
- its parts have different dependency requirements;
- it has a stable public facade and private implementation details;
- several features or contributors need an explicit ownership boundary;
- the file is difficult to review or navigate as one unit.

Preserve the public semantic API during an extraction where practical. Add
headless tests before moving behavior across boundaries.

## Boundary Review Questions

Before adding a dependency or moving code, ask:

1. Which subsystem owns this state or invariant?
2. Is this operation a pure transition or an external effect?
3. Can the behavior be tested without a window, GPU, filesystem, clock, or network?
4. Does the new dependency reverse the architecture direction?
5. Should the decision be recorded in `docs/decisions/`?
