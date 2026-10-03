# Current Blockers

This document records the issues that currently prevent Functors from feeling
like a dependable visual programming environment. It is intentionally more
concrete than the roadmap: each item names the user-visible problem, the
architectural risk, and the condition for considering it resolved.

## Priority

Visual ergonomics is a product requirement for both the editor and the future
cell notebook. Text, cursor placement, menus, widgets, diagnostics, notebook
outputs, and window chrome must feel like parts of one coherent environment.
Phase 2 notebook work should not deepen the current rendering compromises.

## Blockers

### 1. Production text rendering

**Current problem:** The editor text path currently combines Skia CPU paragraph
rasterization with WGPU pixel quads. It demonstrates shaping and fallback, but
it is not yet a crisp, efficient, production text renderer. Fixed-cell editor
geometry also needs to remain synchronized with shaped glyph positions and
emoji advances.

**Required direction:** Keep `FrameDescription` backend-neutral and replace the
pixel-quad bridge with a Vello scene and GPU text path. Validate the selected
shaping and font-fallback stack on Windows first, including color emoji and
mixed-font cursor geometry. Keep a platform-neutral fallback plan while
Vello's text integration is being proven.

**Done when:** Consolas and Segoe UI Emoji render sharply, color emoji work
where the platform supports them, cursor and selection geometry remain aligned
for mixed text and emoji, and text rendering does not dominate frame time.

### 2. Widget and theme system

**Current problem:** Menus, command bar, settings, scrollbars, panels, and
future notebook controls are currently hand-built frame geometry. There is no
shared widget state, focus model, accessibility model, or theme contract.

**Required direction:** Choose a widget layer deliberately. `egui` is the
smallest step toward WGPU integration; `iced` or Slint provide stronger
application-level widget and styling models. The choice must support keyboard
navigation, focus, high-DPI layout, theme tokens, and embedding the editor and
HTML outputs.

**Done when:** Editor and notebook controls share theme tokens for typography,
spacing, colors, borders, focus, selection, diagnostics, and disabled states.
Menus and settings are functional without duplicating ad hoc hit regions.

### 3. Window chrome and application theming

**Current problem:** Native window decorations are controlled by the operating
system, so the application cannot fully reproduce a unified Functors theme.

**Required direction:** Decide between limited native decoration theming and
custom window chrome. Custom chrome enables a themed title bar, borders,
shadows, tabs, and controls, but requires correct dragging, resizing,
minimize/maximize/restore, system menu behavior, and accessibility handling.

**Done when:** The chosen mode has a documented platform implementation and the
window chrome, widgets, editor, and notebook surfaces use one coherent theme.

### 4. Diagnostics and visual programming feedback

**Current problem:** The frame contract can carry overlay text, but diagnostics
are not yet a first-class semantic model. Errors, warnings, hints, ranges, and
notebook execution states need stable visual representations.

**Required direction:** Add backend-neutral diagnostic records to the model/view
contract. Keep glyph shaping and rasterization in the renderer, but let the
view decide ranges, underlines, markers, badges, and inline messages.

**Done when:** Diagnostics are deterministic in headless tests, readable in
both editor and notebook contexts, keyboard navigable, and themeable without
renderer-specific data leaking into the model.

### 5. HTML notebook composition

**Current problem:** Future HTML cells will need to coexist with native editor
text, widgets, diagnostics, and GPU content. A webview or embedded browser has
its own lifecycle, input routing, scaling, theme, and security constraints.

**Required direction:** Define an output-surface boundary before implementing
rich notebook cells. HTML content must be isolated from the document model and
execution effects, with explicit sizing, focus, input, origin, and trust
policies. The native theme should be expressible to HTML through a controlled
theme contract rather than copied CSS fragments.

**Done when:** HTML outputs resize predictably, participate in scrolling and
focus navigation, share theme values, and cannot access workspace or process
resources outside their declared capabilities.

## Recommended Sequence

1. Establish the text-rendering contract and choose the production Windows
   backend, with a platform-neutral fallback plan.
2. Introduce theme tokens and a widget/focus layer for menus, command bar,
   settings, and scrollbars.
3. Decide and implement native or custom window chrome.
4. Add semantic diagnostics to `FrameDescription` and test their layout.
5. Define the HTML output-surface boundary before beginning notebook UI work.

## Constraints

- Preserve the dependency direction:
  `winit -> input -> Message -> update -> Model -> view -> FrameDescription -> renderer`.
- Keep WGPU available as Vello's composition backend for geometry, notebook
  surfaces, and future visualization.
- Do not put window handles, WGPU resources, Direct2D objects, Skia objects, or
  webview handles in model, MVU, or backend-neutral view code.
- Treat the current fontdue/bitmap fallback and Skia pixel-quad path as
  transitional until visual quality and performance are measured on supported
  platforms.

## References

- [Architecture](architecture.md)
- [Skia text-rendering decision](decisions/0005-skia-text-rendering-and-diagnostics.md)
- [Platforms](platforms.md)
- [Roadmap](roadmap.md)
- [Future documentation register](future.md)
