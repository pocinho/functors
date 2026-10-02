# Functors Documentation

This directory contains the design, development, architecture, and contributor
references for Functors.

## Start Here

- [Development Guide](development.md) - local setup, commands, and smoke tests
- [Testing Strategy](testing.md) - test boundaries and validation expectations
- [Module Ownership](module_ownership.md) - dependency direction and subsystem responsibilities
- [Rust Best Practices](rust_best_practices.md) - Rust and architecture standards
- [Architecture](architecture.md) - product vision and current architecture
- [Architecture Decisions](decisions/README.md) - decisions that should remain explicit over time
- [Future Documentation](future.md) - documentation to add when future boundaries become real
- [Roadmap](roadmap.md) - planned capabilities and sequencing
- [Platforms](platforms.md) - platform constraints and support notes

## Documentation Areas

### Contributor References

These documents describe how to work on the repository:

- `development.md`
- `testing.md`
- `module_ownership.md`
- `rust_best_practices.md`
- `decisions/`

### Product and System Design

These documents describe what Functors is intended to become and how its major
systems fit together:

- `architecture.md`
- `roadmap.md`
- `milestones/`
- `notes/`

### Published and API Documentation

- `book/` contains the mdBook source and generated book output. It is primarily
	the end-user and project guide, not the canonical developer reference.
- `api/` contains API-oriented documentation.

### Future References

- `future.md` records important documentation that is intentionally deferred
	until the related subsystem or contract exists.

Keep contributor guidance close to the repository root and use the design
folders for deeper subsystem and product documentation. When two documents
cover the same subject, link to the owning document rather than copying large
sections into both.

## Documentation Rules

- Describe current behavior accurately and label future designs as proposed.
- Update the owning document when a subsystem changes.
- Add an architecture decision when a choice has meaningful long-term tradeoffs.
- Update the mdBook when a change affects end users or the project guide.
- Prefer diagrams, contracts, and examples over broad aspirational prose.
- Keep commands and validation steps synchronized with the repository.
