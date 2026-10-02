# Future Documentation

This file is a planning register for documentation that will become important
as Functors gains new capabilities. It is not a requirement to create empty
files now and it is not a substitute for the roadmap.

Create a document when the related subsystem, public contract, or operational
risk becomes real enough to explain. Until then, keep the topic here with its
trigger and intended scope.

## Near Future

### Text and Document Model

**Suggested file:** `docs/text_model.md`

**Create when:** text editing moves beyond the current basic document model or
when storage, Unicode, shaping, or undo/redo becomes an implementation choice.

**Cover:** positions and coordinate units, ranges, selections, line endings,
Unicode and grapheme behavior, editing transactions, undo/redo, storage
representation, and migration constraints.

**Current owners:** the document and editor invariants are described in
`docs/architecture.md`, `docs/module_ownership.md`, and the Rust practices
document.

### Rendering Pipeline

**Suggested file:** `docs/rendering_pipeline.md`

**Create when:** layout, text shaping, resource caching, multiple render
backends, or more complex GPU lifecycle behavior is introduced.

**Cover:** model-to-frame data flow, layout ownership, `FrameDescription`, text
and font resources, surface lifecycle, resize and recovery, caching, and
headless versus GPU testing.

**Current owners:** `docs/architecture.md`, `docs/testing.md`, and the renderer
ownership guidance in `docs/module_ownership.md`.

### Workspace and Persistence

**Suggested file:** `docs/workspace_model.md`

**Create when:** projects, files, tabs, persistence, or workspace metadata are
implemented.

**Cover:** workspace identity, project roots, file ownership, metadata format,
serialization, migrations, file watching, failure recovery, and compatibility.

**Current owners:** the workspace sections of `docs/architecture.md` and the
product roadmap.

## Later Feature Boundaries

### Notebook Runtime

**Suggested file:** `docs/notebook_runtime.md`

**Create when:** notebook cells can execute or produce managed outputs.

**Cover:** cell identity, dependency graphs, invalidation, scheduling,
cancellation, execution environments, caching, outputs, logs, and errors.

### Agent Runtime

**Suggested file:** `docs/agent_runtime.md`

**Create when:** agents can perform real tool calls or participate in editor or
notebook workflows.

**Cover:** sessions, messages, streaming, tool calls, permissions, capability
boundaries, cancellation, retries, diagnostics, and secret handling.

### Plugin API

**Suggested file:** `docs/plugin_api.md`

**Create when:** third-party or separately versioned extensions become a real
supported integration point.

**Cover:** extension points, capabilities, lifecycle, versioning, compatibility,
resource limits, failure isolation, and shutdown.

### Security Model

**Suggested file:** `docs/security_model.md`

**Create before:** enabling untrusted plugins, arbitrary agent tools, process
execution, workspace automation, or access to user secrets.

**Cover:** trust boundaries, filesystem and process access, network access,
secrets, untrusted documents, plugin and agent permissions, auditing, and
recovery from unsafe operations.

## Operational Documentation

### Release and Compatibility Guide

**Suggested files:** `CHANGELOG.md` and `docs/compatibility.md`

**Create when:** Functors has user-facing releases or supported compatibility
contracts.

**Cover:** release notes, upgrade guidance, supported Rust versions, operating
systems, GPU backends, workspace formats, and plugin or API compatibility.

### Documentation Governance

**Suggested file:** `docs/documentation.md`

**Create when:** multiple contributors or several documentation surfaces make
ownership and review difficult to infer.

**Cover:** source-of-truth rules, audience labels, status labels, review
triggers, mdBook publishing, API documentation generation, and link checking.

## Decision Rule

Create future documentation when either condition is met:

1. A subsystem has a stable contract that another contributor must understand.
2. A wrong assumption would create a costly compatibility, security, or data
   migration problem.

Record the architectural choice in `docs/decisions/` as soon as it becomes
expensive to reverse. Keep this file updated by replacing a future entry with a
link to the completed document once the document exists.
