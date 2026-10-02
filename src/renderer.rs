use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use font8x8::UnicodeFonts;
use wgpu::util::DeviceExt;
use winit::window::Window;

use crate::view::{Color, FrameDescription};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

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
        let vertices = build_vertices(frame);
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

fn build_vertices(frame: &FrameDescription) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    let scale = 1.5;
    let line_color = [0.48, 0.52, 0.58, 1.0];
    let text_color = [0.86, 0.88, 0.90, 1.0];
    let cursor_color = [0.95, 0.75, 0.30, 1.0];
    let gutter_color = [0.12, 0.13, 0.15, 1.0];

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
            &line_number.text,
            line_number.x,
            line_number.y,
            scale,
            line_color,
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
            &text_run.text,
            text_run.x,
            text_run.y,
            scale,
            text_color,
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

    vertices
}

fn append_text(
    vertices: &mut Vec<Vertex>,
    frame: &FrameDescription,
    text: &str,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
) {
    for (character_index, character) in text.chars().enumerate() {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(character) {
            for (row, bits) in glyph.iter().enumerate() {
                for column in 0..8 {
                    if bits & (1 << column) != 0 {
                        append_rect(
                            vertices,
                            frame,
                            x + character_index as f32 * 10.0 + column as f32 * scale,
                            y + 2.0 + row as f32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
    }
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
