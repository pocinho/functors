# 0001: Preserve Explicit Domain and Effect Boundaries

- **Status:** Accepted
- **Date:** 2026-10-02

## Context

Functor combines a deterministic editor core with platform lifecycle,
GPU rendering, future workspace persistence, notebook execution, agents, and
plugins. These concerns have different failure modes, dependencies, and
lifecycle requirements.

Allowing platform, GPU, filesystem, or runtime concerns to enter the model and
view would make core behavior harder to test and would couple future features
to the desktop shell.

## Decision

Maintain explicit boundaries between:

```text
platform -> input adapter -> semantic Message -> pure update -> Model -> view/layout
                                                                  -> FrameDescription -> renderer
```

The MVU update computes deterministic model transitions and declared commands.
Commands and platform code perform external effects. The view produces a
backend-neutral frame description, and the renderer consumes that description
without inspecting the domain model.

As subsystems grow, use folder-based modules with narrow public facades when
multiple ownership or dependency boundaries justify the split. Do not create
folders only to match a planned architecture.

## Alternatives Considered

### Let the renderer inspect the model directly

Rejected because it couples domain state to WGPU and makes headless rendering
and layout tests less useful.

### Put file, network, and GPU work inside update

Rejected because it makes transitions nondeterministic, difficult to test, and
hard to schedule or cancel consistently.

### Create the full future directory tree immediately

Rejected because empty or artificial boundaries add indirection before the
subsystems have stable responsibilities.

## Consequences

- Core model, MVU, input mapping, and layout behavior can be tested headlessly.
- Platform and GPU lifecycle code remains effectful and requires targeted smoke
  or integration tests.
- New features need an explicit message, command, or subsystem boundary.
- Some data may be moved or cloned at boundaries when that is clearer than
  exposing implementation details.
- Module extraction is an intentional architectural change, not a formatting
  exercise.

## Follow-up Triggers

Review this decision when:

- the application gains a second platform integration;
- workspace, notebook, agent, or plugin runtimes become active;
- renderer and layout ownership require independent release or test cycles;
- performance measurements show that the current transition or frame boundary
  is no longer adequate.
