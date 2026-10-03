use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

use vello::{
    Glyph, Scene,
    kurbo::{Affine, Rect, RoundedRect, Stroke},
    peniko::{Blob, Color, Fill, FontData},
};

use crate::parley_text::ParleyTextEngine;
use crate::text::GlyphRun;
use crate::view::{Color as FrameColor, FrameDescription, Rect as FrameRect};
use crate::widgets::{Bounds, PaintDescription, Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneLayer {
    Background,
    Panels,
    Gutter,
    Text,
    Selections,
    Cursors,
    Scrollbars,
}

pub const fn scene_layer_order() -> [SceneLayer; 7] {
    [
        SceneLayer::Background,
        SceneLayer::Panels,
        SceneLayer::Gutter,
        SceneLayer::Text,
        SceneLayer::Selections,
        SceneLayer::Cursors,
        SceneLayer::Scrollbars,
    ]
}

pub struct VelloGpuRenderer {
    renderer: vello::Renderer,
    text_engine: ParleyTextEngine,
    font_cache: HashMap<(usize, usize, u32), Arc<Vec<u8>>>,
}

impl VelloGpuRenderer {
    pub fn new(device: &wgpu::Device) -> Result<Self, vello::Error> {
        let renderer = vello::Renderer::new(
            device,
            vello::RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::area_only(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )?;
        Ok(Self {
            renderer,
            text_engine: ParleyTextEngine::new(),
            font_cache: HashMap::new(),
        })
    }

    pub fn append_frame_text(&mut self, scene: &mut Scene, frame: &FrameDescription) {
        append_frame_text_with_engine(scene, frame, &mut self.text_engine, &mut self.font_cache);
    }

    pub fn build_frame_scene(&mut self, frame: &FrameDescription) -> Scene {
        let mut scene = build_scene_background(frame);
        self.append_frame_text(&mut scene, frame);
        append_frame_decorations(&mut scene, frame);
        scene
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        target: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), vello::Error> {
        self.renderer.render_to_texture(
            device,
            queue,
            scene,
            target,
            &vello::RenderParams {
                base_color: Color::from_rgba8(0, 0, 0, 0),
                width,
                height,
                antialiasing_method: vello::AaConfig::Area,
            },
        )
    }
}

pub fn build_scene(frame: &FrameDescription) -> Scene {
    let mut scene = build_scene_background(frame);
    append_frame_decorations(&mut scene, frame);
    scene
}

fn build_scene_background(frame: &FrameDescription) -> Scene {
    let mut scene = Scene::new();
    fill_rect(&mut scene, frame.viewport_rect(), frame.background);
    fill_rect(&mut scene, frame.menu_bar, FrameColor(20, 23, 28, 255));
    fill_rect(&mut scene, frame.gutter, FrameColor(30, 33, 38, 255));

    if let Some(command_bar) = frame.command_bar {
        fill_rect(&mut scene, command_bar, FrameColor(45, 48, 55, 250));
    }
    if let Some(settings_panel) = frame.settings_panel {
        fill_rect(&mut scene, settings_panel, FrameColor(45, 48, 55, 250));
    }
    scene
}

fn append_frame_decorations(scene: &mut Scene, frame: &FrameDescription) {
    for selection in &frame.selections {
        fill_rect(scene, *selection, FrameColor(51, 77, 115, 165));
    }
    for cursor in &frame.cursors {
        fill_rect(scene, *cursor, FrameColor(245, 210, 120, 255));
    }
    for scrollbar in [
        frame.vertical_scrollbar.as_ref(),
        frame.horizontal_scrollbar.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        fill_rect(scene, scrollbar.track, FrameColor(26, 28, 33, 255));
        fill_rect(scene, scrollbar.thumb, FrameColor(90, 97, 110, 255));
    }
}

pub fn append_frame_text(scene: &mut Scene, frame: &FrameDescription) {
    let mut text_engine = ParleyTextEngine::new();
    let mut font_cache = HashMap::new();
    append_frame_text_with_engine(scene, frame, &mut text_engine, &mut font_cache);
}

fn append_frame_text_with_engine(
    scene: &mut Scene,
    frame: &FrameDescription,
    text_engine: &mut ParleyTextEngine,
    font_cache: &mut HashMap<(usize, usize, u32), Arc<Vec<u8>>>,
) {
    for line_number in &frame.line_numbers {
        append_parley_runs(
            scene,
            text_engine,
            &line_number.text,
            (line_number.x, line_number.y),
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: frame.viewport.width,
                height: frame.viewport.height,
            },
            FrameColor(150, 155, 165, 255),
            font_cache,
        );
    }
    for text_run in frame.text_runs.iter().chain(frame.overlay_text.iter()) {
        append_parley_runs(
            scene,
            text_engine,
            &text_run.text,
            (text_run.x, text_run.y),
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: frame.viewport.width,
                height: frame.viewport.height,
            },
            text_run.color,
            font_cache,
        );
    }
}

fn append_parley_runs(
    scene: &mut Scene,
    text_engine: &mut ParleyTextEngine,
    text: &str,
    origin: (f32, f32),
    clip: FrameRect,
    color: FrameColor,
    font_cache: &mut HashMap<(usize, usize, u32), Arc<Vec<u8>>>,
) {
    let clip_shape = Rect::new(
        f64::from(clip.x),
        f64::from(clip.y),
        f64::from(clip.x + clip.width),
        f64::from(clip.y + clip.height),
    );
    for run in text_engine.layout_single_line_runs(text, 16.0) {
        let font_key = (
            run.font.data.as_ref().as_ptr() as usize,
            run.font.data.as_ref().len(),
            run.font.index,
        );
        let font_bytes = font_cache
            .entry(font_key)
            .or_insert_with(|| Arc::new(run.font.data.as_ref().to_vec()))
            .clone();
        let font = FontData::new(Blob::new(font_bytes), run.font.index);
        let glyphs = run.glyphs.iter().map(|glyph| Glyph {
            id: glyph.glyph_id,
            x: glyph.x,
            y: glyph.y,
        });
        scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip_shape);
        scene
            .draw_glyphs(&font)
            .font_size(16.0)
            .transform(Affine::translate((
                f64::from(origin.0),
                f64::from(origin.1),
            )))
            .brush(Color::from_rgba8(color.0, color.1, color.2, color.3))
            .draw(Fill::NonZero, glyphs);
        scene.pop_layer();
    }
}

pub fn append_paint_description(scene: &mut Scene, paint: &PaintDescription) {
    match *paint {
        PaintDescription::Panel {
            bounds,
            fill,
            border,
        } => {
            fill_bounds(scene, bounds, fill);
            if let Some(border) = border {
                let shape = rounded_bounds(bounds, border.radius);
                scene.stroke(
                    &Stroke::new(f64::from(border.width)),
                    Affine::IDENTITY,
                    peniko_color(border.color),
                    None,
                    &shape,
                );
            }
        }
        PaintDescription::Menu { bounds, fill, .. }
        | PaintDescription::CommandBar { bounds, fill, .. }
        | PaintDescription::StatusSurface { bounds, fill, .. } => {
            fill_bounds(scene, bounds, fill);
        }
        PaintDescription::Scrollbar {
            track,
            thumb,
            track_color,
            thumb_color,
        } => {
            fill_bounds(scene, track, track_color);
            fill_bounds(scene, thumb, thumb_color);
        }
    }
}

pub fn append_shaped_line(
    scene: &mut Scene,
    font_bytes: Arc<Vec<u8>>,
    glyph_run: &GlyphRun,
    font_size: f32,
    origin: (f32, f32),
    color: FrameColor,
) {
    let font = FontData::new(Blob::new(font_bytes), 0);
    let glyphs = glyph_run.glyphs.iter().map(|glyph| Glyph {
        id: glyph.glyph_id,
        x: glyph.x,
        y: glyph.y,
    });
    scene
        .draw_glyphs(&font)
        .font_size(font_size)
        .transform(Affine::translate((
            f64::from(origin.0),
            f64::from(origin.1),
        )))
        .brush(Color::from_rgba8(color.0, color.1, color.2, color.3))
        .draw(Fill::NonZero, glyphs);
}

pub fn append_clipped_shaped_line(
    scene: &mut Scene,
    font_bytes: Arc<Vec<u8>>,
    glyph_run: &GlyphRun,
    font_size: f32,
    origin: (f32, f32),
    clip: FrameRect,
    color: FrameColor,
) {
    let clip_shape = Rect::new(
        f64::from(clip.x),
        f64::from(clip.y),
        f64::from(clip.x + clip.width),
        f64::from(clip.y + clip.height),
    );
    scene.push_clip_layer(Fill::NonZero, Affine::IDENTITY, &clip_shape);
    append_shaped_line(scene, font_bytes, glyph_run, font_size, origin, color);
    scene.pop_layer();
}

fn fill_bounds(scene: &mut Scene, bounds: Bounds, color: Rgba) {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return;
    }
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        peniko_color(color),
        None,
        &Rect::new(
            f64::from(bounds.x),
            f64::from(bounds.y),
            f64::from(bounds.x + bounds.width),
            f64::from(bounds.y + bounds.height),
        ),
    );
}

fn rounded_bounds(bounds: Bounds, radius: f32) -> RoundedRect {
    RoundedRect::new(
        f64::from(bounds.x),
        f64::from(bounds.y),
        f64::from(bounds.x + bounds.width),
        f64::from(bounds.y + bounds.height),
        f64::from(radius.max(0.0)),
    )
}

fn peniko_color(color: Rgba) -> Color {
    Color::from_rgba8(color.0, color.1, color.2, color.3)
}

fn fill_rect(scene: &mut Scene, rect: FrameRect, color: FrameColor) {
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgba8(color.0, color.1, color.2, color.3),
        None,
        &Rect::new(
            f64::from(rect.x),
            f64::from(rect.y),
            f64::from(rect.x + rect.width),
            f64::from(rect.y + rect.height),
        ),
    );
}

trait ViewportRect {
    fn viewport_rect(&self) -> FrameRect;
}

impl ViewportRect for FrameDescription {
    fn viewport_rect(&self) -> FrameRect {
        FrameRect {
            x: 0.0,
            y: 0.0,
            width: self.viewport.width,
            height: self.viewport.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;

    use super::{
        SceneLayer, append_clipped_shaped_line, append_paint_description, append_shaped_line,
        build_scene, scene_layer_order,
    };
    use crate::parley_text::ParleyTextEngine;
    use crate::text::FontId;
    use crate::view::{Color, FrameDescription, Rect, Size};
    use crate::widgets::{BorderPaint, Bounds, PaintDescription, Rgba};

    #[test]
    fn builds_scene_geometry_without_renderer_state() {
        let frame = FrameDescription {
            viewport: Size {
                width: 640.0,
                height: 480.0,
            },
            background: Color(30, 30, 30, 255),
            gutter: Rect {
                x: 0.0,
                y: 24.0,
                width: 40.0,
                height: 456.0,
            },
            line_numbers: Vec::new(),
            text_runs: Vec::new(),
            selections: vec![Rect {
                x: 40.0,
                y: 24.0,
                width: 80.0,
                height: 20.0,
            }],
            cursors: vec![Rect {
                x: 40.0,
                y: 24.0,
                width: 2.0,
                height: 20.0,
            }],
            vertical_scrollbar: None,
            horizontal_scrollbar: None,
            menu_bar: Rect {
                x: 0.0,
                y: 0.0,
                width: 640.0,
                height: 24.0,
            },
            command_bar: None,
            settings_panel: None,
            overlay_text: Vec::new(),
        };

        let _scene = build_scene(&frame);

        let mut resized = frame.clone();
        resized.viewport = Size {
            width: 1280.0,
            height: 720.0,
        };
        let _resized_scene = build_scene(&resized);
    }

    #[test]
    fn builds_empty_scene_without_geometry() {
        let frame = FrameDescription {
            viewport: Size {
                width: 0.0,
                height: 0.0,
            },
            background: Color(30, 30, 30, 255),
            gutter: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            line_numbers: Vec::new(),
            text_runs: Vec::new(),
            selections: Vec::new(),
            cursors: Vec::new(),
            vertical_scrollbar: None,
            horizontal_scrollbar: None,
            menu_bar: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            command_bar: None,
            settings_panel: None,
            overlay_text: Vec::new(),
        };

        let _scene = build_scene(&frame);
    }

    #[test]
    fn scene_layers_have_stable_draw_order() {
        assert_eq!(
            scene_layer_order(),
            [
                SceneLayer::Background,
                SceneLayer::Panels,
                SceneLayer::Gutter,
                SceneLayer::Text,
                SceneLayer::Selections,
                SceneLayer::Cursors,
                SceneLayer::Scrollbars,
            ]
        );
    }

    #[test]
    fn translates_every_widget_paint_description() {
        let bounds = Bounds {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 40.0,
        };
        let border = Some(BorderPaint {
            color: Rgba(100, 110, 120, 255),
            width: 1.0,
            radius: 2.0,
        });
        let paints = [
            PaintDescription::Panel {
                bounds,
                fill: Rgba(30, 30, 30, 255),
                border,
            },
            PaintDescription::Menu {
                bounds,
                fill: Rgba(40, 40, 40, 255),
                text: Rgba(220, 220, 220, 255),
            },
            PaintDescription::CommandBar {
                bounds,
                fill: Rgba(45, 45, 45, 255),
                text: Rgba(220, 220, 220, 255),
            },
            PaintDescription::Scrollbar {
                track: bounds,
                thumb: Bounds {
                    x: 10.0,
                    y: 20.0,
                    width: 20.0,
                    height: 40.0,
                },
                track_color: Rgba(20, 20, 20, 255),
                thumb_color: Rgba(90, 90, 90, 255),
            },
            PaintDescription::StatusSurface {
                bounds,
                fill: Rgba(50, 50, 50, 255),
                text: Rgba(220, 220, 220, 255),
            },
        ];
        let mut scene = vello::Scene::new();
        for paint in &paints {
            append_paint_description(&mut scene, paint);
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn emits_a_parley_shaped_line_to_vello() {
        let bytes = fs::read("C:\\Windows\\Fonts\\consola.ttf").unwrap();
        let mut text = ParleyTextEngine::new();
        let layout = text.layout_single_line("cafe", FontId(0), 16.0);
        let mut scene = vello::Scene::new();

        append_shaped_line(
            &mut scene,
            Arc::new(bytes),
            &layout.run,
            16.0,
            (10.0, 24.0),
            Color(220, 220, 220, 255),
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn emits_fractional_shaped_text_inside_a_clip_layer() {
        let bytes = fs::read("C:\\Windows\\Fonts\\consola.ttf").unwrap();
        let mut text = ParleyTextEngine::new();
        let layout = text.layout_single_line("fractional", FontId(0), 16.0);
        let mut scene = vello::Scene::new();

        append_clipped_shaped_line(
            &mut scene,
            Arc::new(bytes),
            &layout.run,
            16.0,
            (10.5, 24.25),
            crate::view::Rect {
                x: 10.0,
                y: 10.0,
                width: 60.0,
                height: 24.0,
            },
            Color(220, 220, 220, 255),
        );
    }
}
