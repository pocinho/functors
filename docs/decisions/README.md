# Architecture Decision Records

This directory records decisions with meaningful long-term consequences for
Functor's architecture, dependencies, public contracts, and operational
behavior.

## When to Add a Decision

Create an ADR when a choice:

- establishes or changes a subsystem boundary;
- introduces a dependency with long-term maintenance cost;
- selects among competing storage, rendering, runtime, or protocol designs;
- affects portability, security, persistence, or plugin compatibility;
- would otherwise be repeatedly reconsidered by future contributors.

Small implementation choices belong in code comments, tests, or normal review
conversation instead.

## Format

Use a sequential filename such as `0001-explicit-effect-boundaries.md`.
Each ADR should include:

- Status
- Context
- Decision
- Alternatives considered
- Consequences
- Follow-up or review triggers

An ADR records why a decision was made. It does not replace current API or
workflow documentation.

## Decisions

- [0001: Preserve Explicit Domain and Effect Boundaries](0001-explicit-domain-effect-boundaries.md)
- [0002: Use a Private Rope-Backed Document](0002-private-rope-backed-document.md)
- [0003: Keep Syntax Highlighting Backend-Neutral](0003-backend-neutral-syntax-highlighting.md)
- [0005: Prefer Skia for Cross-Platform Text Rendering](0005-skia-text-rendering-and-diagnostics.md)
- [0006: Prefer Vello for Rendering and Composition](0006-vello-rendering-and-composition.md)
- [0007: Create Functors-Owned Text and Widget Layers](0007-functors-text-and-widgets.md)
