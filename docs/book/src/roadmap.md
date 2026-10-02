# Roadmap

## Phase 1 — Foundations (Core Complete, Usability Extension In Progress)
- MVU core
- basic renderer
- text buffer
- workspace loading
- file open/save workflow
- Rust and Markdown highlighting

The MVU/editor foundation is complete. Before Phase 2, the usability extension
must finish scrollbar thumb dragging and complete menu command behavior. Skia
is now the production text backend for cross-platform shaping and emoji
fallback, with visual programming diagnostics built on the same foundation. The
current bitmap/fontdue fallback remains available for degraded rendering.

## Phase 2 — Notebook
- cell model
- execution engine
- visualization cells

## Phase 3 — Agents
- unified agent protocol
- Copilot integration
- local LLM support

## Phase 4 — Plugins
- dynamic plugin loader
- language support modules

## Phase 5 — Polish
- theming
- performance tuning
- documentation
