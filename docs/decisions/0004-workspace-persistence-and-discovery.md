# ADR 0004: Workspace Persistence and Discovery

## Status

Accepted for Phase 1.

## Decision

Phase 1 keeps the active workspace in memory as a validated root path and a
sorted, shallow directory listing. The filesystem boundary performs discovery;
the MVU loop receives either a workspace value or a structured error.

Workspace configuration persistence, recursive indexing, and file watching are
deferred. The application can explicitly rediscover a workspace when the user
opens a folder again.

## Rationale

This keeps filesystem access out of the document, update, and view layers while
leaving room for later notebook and plugin metadata. A shallow snapshot is
deterministic and sufficient for the first editor workflow; watching and
persistence would introduce lifecycle, conflict, and format decisions before
they are required.

## Consequences

- directory contents can become stale until rediscovery;
- external file changes are reported only when a later file operation observes
  them;
- future persistence and watching features need explicit effect contracts;
- workspace data remains straightforward to test without a physical folder.