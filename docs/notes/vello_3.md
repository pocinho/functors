# Functors Vello Architecture

> GPU‑native, fully themed UI for the Functors editor using Vello + wgpu + MVU.

---

## 1. Goals

- **Single rendering stack:** Vello + wgpu for *all* UI (chrome, panels, editor, notebook cells).
- **MVU‑driven:** Pure `Model → View → Scene` pipeline.
- **Crisp text everywhere:** Swash + Vello glyph pipeline.
- **Infinite canvas ready:** Notebook cells, zoom, transforms.
- **Cross‑platform & WebGPU‑ready:** Same architecture on desktop and browser.

---

## 2. Stack overview

- **GPU backend:** `wgpu`
- **2D engine:** `vello`
- **Windowing/input:** `winit` (or equivalent)
- **Text shaping:** `swash` (via Vello)
- **Architecture:** MVU (Model–View–Update)

```text
winit (events, window, DPI)
        ↓
   MVU (update)
        ↓
   View (AppSceneDescription)
        ↓
Scene builder (AppSceneDescription → vello::Scene)
        ↓
   Vello + wgpu (render)
```

---

## 3. MVU model and UI domains

### 3.1 Core model

```fsharp
type Theme =
  { Palette : Palette
    Typography : Typography
    Radii : Radii
    Shadows : Shadows }

type EditorBuffer =
  { Text : Rope
    Cursor : Cursor
    Selections : Selection list }

type NotebookCell =
  { Id : CellId
    Kind : CellKind // Code, Markdown, Output
    Buffer : EditorBuffer
    State : CellState }

type Layout =
  { Sidebar : SidebarLayout
    Tabs : TabsLayout
    EditorArea : Rect
    StatusBar : Rect
    Overlays : Overlay list }

type Model =
  { Theme : Theme
    Buffers : Map<BufferId, EditorBuffer>
    Notebook : NotebookCell list
    Layout : Layout
    WindowState : WindowState
    InputState : InputState }
```

The **Model** owns *all* UI state: editor, notebook, chrome, panels, overlays, theme.

---

## 4. View: from Model to AppSceneDescription

### 4.1 AppSceneDescription

```rust
struct AppScene {
    chrome: ChromeScene,
    sidebar: SidebarScene,
    tabs: TabsScene,
    editor: EditorScene,
    notebook: NotebookScene,
    status_bar: StatusBarScene,
    overlays: Vec<OverlayScene>,
}
```

```rust
fn view(model: &Model) -> AppScene {
    AppScene {
        chrome: view_chrome(model),
        sidebar: view_sidebar(model),
        tabs: view_tabs(model),
        editor: view_editor(model),
        notebook: view_notebook(model),
        status_bar: view_status_bar(model),
        overlays: view_overlays(model),
    }
}
```

---

## 5. Scene builder: AppScene → `vello::Scene`

### 5.1 Single scene per frame

```rust
fn build_scene(app: &AppScene, scene: &mut vello::Scene) {
    build_chrome(&app.chrome, scene);
    build_sidebar(&app.sidebar, scene);
    build_tabs(&app.tabs, scene);
    build_editor(&app.editor, scene);
    build_notebook(&app.notebook, scene);
    build_status_bar(&app.status_bar, scene);
    for overlay in &app.overlays {
        build_overlay(overlay, scene);
    }
}
```

Each builder:

- uses theme tokens (colors, radii, shadows)
- emits Vello primitives:
  - fills (rects, rounded rects)
  - strokes (borders, separators)
  - paths (icons)
  - text (glyph runs)

---

## 6. Text rendering

### 6.1 Font + glyph cache

- Load fonts via Swash.
- Maintain a glyph cache keyed by `(font, size, glyph_id)`.
- Let Vello manage GPU glyph atlases internally.

### 6.2 Layout

- A text layout module computes:
  - line breaks
  - glyph positions (fractional)
  - caret and selection geometry

```rust
struct GlyphRun {
    font: FontId,
    size: f32,
    glyphs: Vec<PositionedGlyph>, // x, y, glyph_id
}
```

Editor, tabs, status bar, sidebar all use the same text pipeline.

---

## 7. Theming system

### 7.1 Theme tokens

```rust
struct Palette {
    bg: Color;
    bg_alt: Color;
    fg: Color;
    accent: Color;
    accent_soft: Color;
    border: Color;
    error: Color;
    warning: Color;
    success: Color;
}

struct Radii {
    small: f32;
    medium: f32;
    large: f32;
}

struct Shadows {
    soft: Shadow;
    strong: Shadow;
}
```

All builders consume `Theme` instead of hard‑coded colors.

### 7.2 Dynamic theme switching

- `Model.Theme` is updated by a `Msg::SetTheme`.
- Next frame, `view` + `build_scene` render with the new palette.

---

## 8. Layout and hit‑testing

### 8.1 Layout

A layout engine computes `Rect`s for:

- chrome
- sidebar
- tabs
- editor area
- notebook cells
- status bar
- overlays

These rects drive both rendering and hit‑testing.

### 8.2 Hit‑testing

- Input from `winit` → `Msg` with coordinates.
- Use layout rects to route events:
  - tabs
  - editor
  - cells
  - chrome drag/resize
  - panel resizing

---

## 9. Render loop

```rust
fn render_frame(
    model: &Model,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut vello::Renderer,
    target_tex: &wgpu::Texture,
    width: u32,
    height: u32,
) {
    // 1. View
    let app_scene_desc = view(model);

    // 2. Build Vello scene
    let mut scene = vello::Scene::new();
    build_scene(&app_scene_desc, &mut scene);

    // 3. Render
    renderer
        .render_to_texture(
            device,
            queue,
            &scene,
            target_tex,
            &vello::RenderParams {
                base_color: vello::Color::BLACK,
                width,
                height,
                antialiasing_method: vello::AaConfig::Msaa16,
            },
        )
        .expect("render failed");
}
```

---

## 10. Frameless window & custom chrome

- Use a frameless window.
- Draw title bar, buttons, borders via Vello.
- Implement drag/resize via hit‑testing on chrome rects.

Result: fully branded, cross‑platform Functors UI with no OS widgets.

---

## 11. Notebook cells and infinite canvas

- Each notebook cell is a logical layer:
  - rect + clip
  - text + outputs
  - execution state badges
- Apply Vello transforms (scale, translate) per cell:
  - zoom
  - pan
  - focus/defocus animations
- Future: infinite canvas by mapping cells onto a larger coordinate space.

---

## 12. Extension points

- **Animations:** time‑based state in `Model`, interpolated in `view`.
- **Mini‑map:** separate `EditorScene` view over the same buffer.
- **Inline plots:** vector graphics rendered via Vello inside cells.
- **Agent overlays:** status, suggestions, traces as overlay scenes.
- **WebGPU target:** reuse the same scene pipeline in a browser build.
