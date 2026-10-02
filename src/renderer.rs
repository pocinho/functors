use std::{fs, sync::Arc};

use bytemuck::{Pod, Zeroable};
use font8x8::{BASIC_FONTS, GREEK_FONTS, HIRAGANA_FONTS, LATIN_FONTS, UnicodeFonts};
use fontdue::{Font, FontSettings};
use skia_safe::surfaces::raster_n32_premul;
use skia_safe::textlayout::{
    FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle as SkiaTextStyle,
};
use skia_safe::{Color as SkiaColor, ColorType, FontMgr, FontStyle, Typeface};
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::view::{Color, FrameDescription};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    fonts: FontSet,
}

pub const PRIMARY_FONT_FAMILY: &str = "Consolas";
pub const EMOJI_FALLBACK_FONT_FAMILY: &str = "Segoe UI Emoji";

#[derive(Default)]
struct FontSet {
    primary: Option<Font>,
    emoji_fallback: Option<Font>,
    skia: Option<SkiaTextRenderer>,
}

struct SkiaTextRenderer {
    font_collection: FontCollection,
    primary_typeface: Option<Typeface>,
}

struct TextStyle {
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
}

const TEXT_RASTER_SCALE: f32 = 2.0;

impl FontSet {
    fn load() -> Self {
        if !cfg!(target_os = "windows") {
            return Self::default();
        }

        Self {
            primary: windows_font_path(PRIMARY_FONT_FAMILY).and_then(load_font),
            emoji_fallback: windows_font_path(EMOJI_FALLBACK_FONT_FAMILY).and_then(load_font),
            skia: SkiaTextRenderer::new(),
        }
    }
}

impl SkiaTextRenderer {
    fn new() -> Option<Self> {
        skia_safe::icu::init();
        let font_manager = FontMgr::default();
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(font_manager.clone(), None);
        Some(Self {
            font_collection,
            primary_typeface: font_manager
                .match_family_style(PRIMARY_FONT_FAMILY, FontStyle::normal()),
        })
    }

    fn append_text(
        &self,
        vertices: &mut Vec<Vertex>,
        frame: &FrameDescription,
        text: &str,
        style: &TextStyle,
        width: f32,
    ) -> bool {
        let surface_width = (width * TEXT_RASTER_SCALE).ceil().max(1.0) as i32;
        let surface_height = (24.0 * TEXT_RASTER_SCALE) as i32;
        let Some(mut surface) = raster_n32_premul((surface_width, surface_height)) else {
            return false;
        };
        let canvas = surface.canvas();
        canvas.clear(SkiaColor::TRANSPARENT);

        let mut paragraph_style = ParagraphStyle::new();
        let mut text_style = SkiaTextStyle::new();
        text_style
            .set_font_families(&[PRIMARY_FONT_FAMILY, EMOJI_FALLBACK_FONT_FAMILY])
            .set_font_size(16.0 * TEXT_RASTER_SCALE)
            .set_color(SkiaColor::from_argb(
                (style.color[3] * 255.0) as u8,
                (style.color[0] * 255.0) as u8,
                (style.color[1] * 255.0) as u8,
                (style.color[2] * 255.0) as u8,
            ));
        if let Some(typeface) = self.primary_typeface.as_ref() {
            text_style.set_typeface(typeface.clone());
        }
        paragraph_style.set_text_style(&text_style);
        let mut builder = ParagraphBuilder::new(&paragraph_style, self.font_collection.clone());
        builder.push_style(&text_style);
        builder.add_text(text);
        let mut paragraph = builder.build();
        paragraph.layout((width * TEXT_RASTER_SCALE).max(1.0));
        let raster_scale = width * TEXT_RASTER_SCALE / paragraph.max_intrinsic_width().max(1.0);
        paragraph.paint(canvas, (0.0, 0.0));
        let Some(pixmap) = canvas.peek_pixels() else {
            return false;
        };
        let row_bytes = pixmap.row_bytes();
        let (red_offset, green_offset, blue_offset) = match pixmap.color_type() {
            ColorType::BGRA8888 => (2, 1, 0),
            _ => (0, 1, 2),
        };
        let pixels = unsafe {
            std::slice::from_raw_parts(
                pixmap.addr().cast::<u8>(),
                row_bytes * surface_height as usize,
            )
        };
        let mut rendered_any = false;
        for row in 0..surface_height as usize {
            for column in 0..surface_width as usize {
                let pixel_offset = row * row_bytes + column * 4;
                let alpha = pixels[pixel_offset + 3];
                if alpha > 5 {
                    rendered_any = true;
                    let alpha_f = f32::from(alpha) / 255.0;
                    let unpremultiply = |offset: usize| {
                        (f32::from(pixels[pixel_offset + offset]) / f32::from(alpha)).min(1.0)
                    };
                    let mut color = [
                        unpremultiply(red_offset),
                        unpremultiply(green_offset),
                        unpremultiply(blue_offset),
                        0.0,
                    ];
                    color[3] = alpha_f;
                    append_rect(
                        vertices,
                        frame,
                        style.x + column as f32 / TEXT_RASTER_SCALE * raster_scale,
                        style.y + row as f32 / TEXT_RASTER_SCALE,
                        raster_scale / TEXT_RASTER_SCALE,
                        1.0 / TEXT_RASTER_SCALE,
                        color,
                    );
                }
            }
        }
        rendered_any
    }
}

fn windows_font_path(family: &str) -> Option<&'static str> {
    match family {
        PRIMARY_FONT_FAMILY => Some("C:\\Windows\\Fonts\\consola.ttf"),
        EMOJI_FALLBACK_FONT_FAMILY => Some("C:\\Windows\\Fonts\\seguiemj.ttf"),
        _ => None,
    }
}

fn load_font(path: &str) -> Option<Font> {
    let bytes = fs::read(path).ok()?;
    Font::from_bytes(bytes, FontSettings::default()).ok()
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

const REPLACEMENT_GLYPH: [u8; 8] = [
    0b11111111, 0b10000001, 0b10111101, 0b10100101, 0b10100101, 0b10111101, 0b10000001, 0b11111111,
];

impl Vertex {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: std::mem::size_of::<[f32; 2]>() as u64,
                    shader_location: 1,
                },
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStatus {
    Presented,
    Skipped,
    Reconfigure,
}

impl Renderer {
    pub async fn new(
        window: Arc<Window>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Functors Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let alpha_mode = capabilities.alpha_modes[0];
        let config = surface_config(format, present_mode, alpha_mode, size.width, size.height);
        surface.configure(&device, &config);
        let pipeline = create_pipeline(&device, format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            fonts: FontSet::load(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn render(&mut self, frame: &FrameDescription) -> RenderStatus {
        if self.config.width == 0 || self.config.height == 0 {
            return RenderStatus::Skipped;
        }

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => return RenderStatus::Skipped,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return RenderStatus::Reconfigure;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Functors Clear Pass"),
            });
        let vertices = build_vertices(frame, &self.fonts);
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Functors Frame Geometry"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Functors Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(to_wgpu_color(frame.background)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        self.queue.present(output);
        RenderStatus::Presented
    }
}

fn create_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Functors Solid Geometry Shader"),
        source: wgpu::ShaderSource::Wgsl(
            "\n                struct VertexInput {\n                    @location(0) position: vec2<f32>,\n                    @location(1) color: vec4<f32>,\n                };\n                struct VertexOutput {\n                    @builtin(position) position: vec4<f32>,\n                    @location(0) color: vec4<f32>,\n                };\n                @vertex\n                fn vs_main(input: VertexInput) -> VertexOutput {\n                    var output: VertexOutput;\n                    output.position = vec4<f32>(input.position, 0.0, 1.0);\n                    output.color = input.color;\n                    return output;\n                }\n                @fragment\n                fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {\n                    return input.color;\n                }\n            ".into(),
        ),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Functors Solid Geometry Pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(Vertex::layout())],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

fn build_vertices(frame: &FrameDescription, fonts: &FontSet) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let scale = 1.5;
    let line_color = [0.48, 0.52, 0.58, 1.0];
    let cursor_color = [0.95, 0.75, 0.30, 1.0];
    let gutter_color = [0.12, 0.13, 0.15, 1.0];
    let scrollbar_track_color = [0.10, 0.11, 0.13, 1.0];
    let scrollbar_thumb_color = [0.35, 0.38, 0.43, 1.0];
    let menu_color = [0.08, 0.09, 0.11, 1.0];
    let panel_color = [0.14, 0.15, 0.18, 0.98];

    append_rect(
        &mut vertices,
        frame,
        frame.menu_bar.x,
        frame.menu_bar.y,
        frame.menu_bar.width,
        frame.menu_bar.height,
        menu_color,
    );
    if let Some(bar) = frame.command_bar {
        append_rect(
            &mut vertices,
            frame,
            bar.x,
            bar.y,
            bar.width,
            bar.height,
            panel_color,
        );
    }
    if let Some(panel) = frame.settings_panel {
        append_rect(
            &mut vertices,
            frame,
            panel.x,
            panel.y,
            panel.width,
            panel.height,
            panel_color,
        );
    }

    append_rect(
        &mut vertices,
        frame,
        frame.gutter.x,
        frame.gutter.y,
        frame.gutter.width,
        frame.gutter.height,
        gutter_color,
    );

    for line_number in &frame.line_numbers {
        append_text(
            &mut vertices,
            frame,
            fonts,
            &line_number.text,
            TextStyle {
                x: line_number.x,
                y: line_number.y,
                scale,
                color: line_color,
            },
        );
    }
    for text_run in &frame.overlay_text {
        append_text(
            &mut vertices,
            frame,
            fonts,
            &text_run.text,
            TextStyle {
                x: text_run.x,
                y: text_run.y,
                scale,
                color: to_vertex_color(text_run.color),
            },
        );
    }
    for selection in &frame.selections {
        append_rect(
            &mut vertices,
            frame,
            selection.x,
            selection.y,
            selection.width,
            selection.height,
            [0.20, 0.30, 0.45, 0.65],
        );
    }
    for text_run in &frame.text_runs {
        append_text(
            &mut vertices,
            frame,
            fonts,
            &text_run.text,
            TextStyle {
                x: text_run.x,
                y: text_run.y,
                scale,
                color: to_vertex_color(text_run.color),
            },
        );
    }
    for cursor in &frame.cursors {
        append_rect(
            &mut vertices,
            frame,
            cursor.x,
            cursor.y,
            cursor.width,
            cursor.height,
            cursor_color,
        );
    }
    for scrollbar in [
        frame.vertical_scrollbar.as_ref(),
        frame.horizontal_scrollbar.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        append_rect(
            &mut vertices,
            frame,
            scrollbar.track.x,
            scrollbar.track.y,
            scrollbar.track.width,
            scrollbar.track.height,
            scrollbar_track_color,
        );
        append_rect(
            &mut vertices,
            frame,
            scrollbar.thumb.x,
            scrollbar.thumb.y,
            scrollbar.thumb.width,
            scrollbar.thumb.height,
            scrollbar_thumb_color,
        );
    }

    vertices
}

fn append_text(
    vertices: &mut Vec<Vertex>,
    frame: &FrameDescription,
    fonts: &FontSet,
    text: &str,
    style: TextStyle,
) {
    if let Some(skia) = fonts.skia.as_ref()
        && skia.append_text(
            vertices,
            frame,
            text,
            &style,
            (text.chars().count() as f32 * 10.0).max(10.0),
        )
    {
        return;
    }

    for (character_index, character) in text.chars().enumerate() {
        if let Some(font) = fonts
            .primary
            .as_ref()
            .filter(|font| font.lookup_glyph_index(character) != 0)
            .or_else(|| {
                fonts
                    .emoji_fallback
                    .as_ref()
                    .filter(|font| font.lookup_glyph_index(character) != 0)
            })
        {
            let (metrics, bitmap) = font.rasterize(character, 16.0);
            let cell_x = style.x + character_index as f32 * 10.0;
            let glyph_width = metrics.width as f32;
            let glyph_scale = if glyph_width > 0.0 {
                (9.0 / glyph_width).min(1.0)
            } else {
                1.0
            };
            let glyph_x = cell_x
                + ((10.0 - glyph_width * glyph_scale) / 2.0).max(0.0)
                + metrics.xmin as f32 * glyph_scale;
            let glyph_height = metrics.height as f32 * glyph_scale;
            let glyph_y = style.y + ((16.0 - glyph_height) / 2.0).max(0.0)
                - metrics.ymin as f32 * glyph_scale;
            for row in 0..metrics.height {
                for column in 0..metrics.width {
                    let alpha = bitmap[row * metrics.width + column] as f32 / 255.0;
                    if alpha > 0.05 {
                        let mut glyph_color = style.color;
                        glyph_color[3] *= alpha;
                        append_rect(
                            vertices,
                            frame,
                            glyph_x + column as f32 * glyph_scale,
                            glyph_y + row as f32 * glyph_scale,
                            glyph_scale,
                            glyph_scale,
                            glyph_color,
                        );
                    }
                }
            }
            continue;
        }

        let glyph = glyph_for(character);
        for (row, bits) in glyph.iter().enumerate() {
            for column in 0..8 {
                if bits & (1 << column) != 0 {
                    append_rect(
                        vertices,
                        frame,
                        style.x + character_index as f32 * 10.0 + column as f32 * style.scale,
                        style.y + 2.0 + row as f32 * style.scale,
                        style.scale,
                        style.scale,
                        style.color,
                    );
                }
            }
        }
    }
}

fn glyph_for(character: char) -> [u8; 8] {
    BASIC_FONTS
        .get(character)
        .or_else(|| LATIN_FONTS.get(character))
        .or_else(|| GREEK_FONTS.get(character))
        .or_else(|| HIRAGANA_FONTS.get(character))
        .unwrap_or(REPLACEMENT_GLYPH)
}

fn append_rect(
    vertices: &mut Vec<Vertex>,
    frame: &FrameDescription,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
) {
    let left = x / frame.viewport.width * 2.0 - 1.0;
    let right = (x + width) / frame.viewport.width * 2.0 - 1.0;
    let top = 1.0 - y / frame.viewport.height * 2.0;
    let bottom = 1.0 - (y + height) / frame.viewport.height * 2.0;
    vertices.extend([
        Vertex {
            position: [left, top],
            color,
        },
        Vertex {
            position: [right, top],
            color,
        },
        Vertex {
            position: [right, bottom],
            color,
        },
        Vertex {
            position: [left, top],
            color,
        },
        Vertex {
            position: [right, bottom],
            color,
        },
        Vertex {
            position: [left, bottom],
            color,
        },
    ]);
}

fn surface_config(
    format: wgpu::TextureFormat,
    present_mode: wgpu::PresentMode,
    alpha_mode: wgpu::CompositeAlphaMode,
    width: u32,
    height: u32,
) -> wgpu::SurfaceConfiguration {
    wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        color_space: wgpu::SurfaceColorSpace::Auto,
        width: width.max(1),
        height: height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    }
}

fn to_wgpu_color(color: Color) -> wgpu::Color {
    wgpu::Color {
        r: f64::from(color.0) / 255.0,
        g: f64::from(color.1) / 255.0,
        b: f64::from(color.2) / 255.0,
        a: f64::from(color.3) / 255.0,
    }
}

fn to_vertex_color(color: Color) -> [f32; 4] {
    [
        f32::from(color.0) / 255.0,
        f32::from(color.1) / 255.0,
        f32::from(color.2) / 255.0,
        f32::from(color.3) / 255.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::{FontSet, REPLACEMENT_GLYPH, TextStyle, Vertex, append_text, glyph_for};
    use crate::view::{Color, FrameDescription, Rect, Size};

    #[test]
    fn renders_supported_unicode_and_marks_unknown_glyphs() {
        assert_ne!(glyph_for('A'), REPLACEMENT_GLYPH);
        assert_ne!(glyph_for('λ'), REPLACEMENT_GLYPH);
        assert_eq!(glyph_for('😀'), REPLACEMENT_GLYPH);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn skia_produces_pixels_for_letters_and_emoji_without_following_input() {
        let frame = FrameDescription {
            viewport: Size {
                width: 800.0,
                height: 600.0,
            },
            background: Color(0, 0, 0, 255),
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
        let fonts = FontSet::load();
        let mut vertices = Vec::<Vertex>::new();

        append_text(
            &mut vertices,
            &frame,
            &fonts,
            "A😀",
            TextStyle {
                x: 10.0,
                y: 10.0,
                scale: 1.5,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        );

        assert!(!vertices.is_empty());
        vertices.clear();
        append_text(
            &mut vertices,
            &frame,
            &fonts,
            "😀",
            TextStyle {
                x: 10.0,
                y: 10.0,
                scale: 1.5,
                color: [1.0, 1.0, 1.0, 1.0],
            },
        );
        assert!(!vertices.is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn loads_the_configured_windows_editor_fonts() {
        let fonts = FontSet::load();

        assert!(fonts.primary.is_some());
        assert!(fonts.emoji_fallback.is_some());
    }
}
