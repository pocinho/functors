### High-level architecture for a Vello-based Functors editor

| Layer | Tech | Role in Functors |
|------|------|------------------|
| GPU backend | **wgpu** | Device, queue, swapchain/surface |
| 2D engine | **Vello** | Vector/text rendering, composition |
| Scene model | **Vello Scene + your MVU model** | UI + document + decorations |
| Text stack | **Swash + Vello glyph cache** | Shaping, glyph atlases, crisp text |
| App shell | **winit (or similar)** | Window, input, resize, DPI |

---

### 1. Rendering core: wgpu + Vello

**Renderer bootstrap**

- **Create wgpu device/queue/surface** once at startup.  
- **Create `vello::Renderer`** bound to that device:

```rust
let mut renderer = vello::Renderer::new(
    &device,
    vello::RendererOptions {
        use_cpu: false,
        antialiasing_support: vello::AaSupport::all(),
        num_init_threads: NonZeroUsize::new(1),
    },
)?;
```

You keep this `renderer` alive for the whole app and render into a wgpu texture or directly into the swapchain.   [Docs.rs](https://docs.rs/vello/latest/index.html)  

---

### 2. Scene as the bridge between MVU and GPU

**Core idea:**  
Your MVU `Model` describes the editor state; a **Scene builder** turns that into a `vello::Scene`.

- **MVU Model**: buffers, cursors, selections, gutters, minimap, notebook cells, overlays.
- **View function**: pure function `Model -> SceneDescription` (your own intermediate).
- **Scene builder**: `SceneDescription -> vello::Scene`.

In practice:

```rust
let mut scene = vello::Scene::new();

// background
scene.fill(
    vello::peniko::Fill::NonZero,
    vello::Affine::IDENTITY,
    vello::Color::from_rgb8(18, 18, 18),
    None,
    &vello::Rect::new(0.0, 0.0, width as f64, height as f64),
);

// more: gutters, text lines, carets, highlights…
```

You treat Vello’s `Scene` as the **render target** for each frame.   [deepwiki.com](https://deepwiki.com/linebender/vello/1.1-architecture)  [Docs.rs](https://docs.rs/vello/latest/index.html)  

---

### 3. Vello pipeline: how your scene becomes pixels

Vello internally runs a **compute-centric pipeline**:

- **Encoding**: your `Scene` is converted into a compact GPU-friendly binary format (paths, layers, glyphs, gradients).   [deepwiki.com](https://deepwiki.com/linebender/vello/1.1-architecture)  
- **Coarse rasterization**: tiles, binning, visibility, clipping.
- **Fine rasterization**: per-pixel coverage, AA, blending.

You don’t manage these stages directly—your job is to:

1. Build a good `Scene`.
2. Call `renderer.render_to_texture` (or surface) with width/height + AA config.   [Github](https://github.com/linebender/vello/blob/main/ARCHITECTURE.md)  [Docs.rs](https://docs.rs/vello/latest/index.html)  

---

### 4. Text rendering architecture for Functors

**Font + shaping**

- Use **Swash** (via Vello) for font loading and shaping.
- Maintain a **glyph cache** keyed by `(font, size, glyph_id)`—Vello already has infrastructure for glyph atlases and caches.   [deepwiki.com](https://deepwiki.com/linebender/vello/1.1-architecture)  

**Layout**

- Your editor’s layout engine computes:
  - line positions  
  - glyph advances  
  - caret positions  
  - selection rectangles  

- For each line:
  - Compute baseline `y`.
  - For each glyph: `x` = accumulated advance (fractional, not snapped).
  - Emit text as either:
    - Vello text primitives (when available), or
    - per-glyph quads using Vello’s scene API.

**Crispness**

- Fractional positioning + GPU AA gives you:
  - smooth scrolling  
  - crisp zoom  
  - stable spacing  

You get “Lapce-like” sharpness but with a cleaner, higher-level engine.

---

### 5. Integrating with MVU and notebook cells

**Update loop**

1. **Input → Msg** (keyboard, mouse, IME, scroll, resize).
2. **Msg → Model** (pure update).
3. **Model → SceneDescription** (your view).
4. **SceneDescription → `vello::Scene`**.
5. **Render** via `renderer.render_to_texture` or directly to surface.

Notebook cells fit naturally:

- Each cell is a **layer** or **group** in the Scene.
- You can:
  - clip per cell  
  - apply transforms (zoom, pan, rotation)  
  - highlight active cell  
  - overlay execution status, outputs, inline plots.

Vello’s layering and clipping model is designed for this kind of composited 2D UI.   [deepwiki.com](https://deepwiki.com/linebender/vello/1.1-architecture)  

---

### 6. Minimal “frame” pseudocode for Functors

```rust
fn render_frame(model: &Model, device: &wgpu::Device, queue: &wgpu::Queue,
                renderer: &mut vello::Renderer, target_tex: &wgpu::Texture,
                width: u32, height: u32) {

    // 1. Build scene from MVU model
    let scene_desc = view(model);          // your own type
    let mut scene = vello::Scene::new();
    build_scene(&scene_desc, &mut scene);  // emits fills, strokes, text

    // 2. Render via Vello
    renderer.render_to_texture(
        device,
        queue,
        &scene,
        target_tex,
        &vello::RenderParams {
            base_color: palette::css::BLACK,
            width,
            height,
            antialiasing_method: vello::AaConfig::Msaa16,
        },
    ).expect("render failed");
}
```
