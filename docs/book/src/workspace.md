# Workspace & Projects

In the current desktop editor, a workspace is a validated folder opened with
`Ctrl+O`. Functors performs a shallow directory discovery and keeps the sorted
file and directory entries in the MVU model.

The current workflow is:

- `Ctrl+O` opens a native folder picker and activates a workspace;
- `Ctrl+P` opens a native file picker for a file in or outside that workspace;
- `Ctrl+S` saves the active UTF-8 document;
- the window title shows the workspace or active file, dirty state, loading
	state, and errors.

Workspace discovery is a filesystem effect at the application boundary. The
view does not read directories directly, and the model stores a snapshot that
can become stale until the folder is opened again.

Workspace persistence, recursive file trees, tabs, project metadata, and file
watching are deferred to later workspace and UX work. See the
[workspace persistence decision](../../decisions/0004-workspace-persistence-and-discovery.md)
for the current ownership and lifecycle assumptions.
