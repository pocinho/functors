use std::error::Error;
use std::sync::Arc;

mod input;
mod model;
mod mvu;
mod renderer;
mod view;

use winit::{
    application::ApplicationHandler,
    event::{ElementState, Ime, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

struct FunctorsApp {
    window: Option<Arc<Window>>,
    model: model::Model,
    renderer: Option<renderer::Renderer>,
    pointer_position: Option<(f32, f32)>,
    pointer_down: bool,
}

impl Default for FunctorsApp {
    fn default() -> Self {
        Self {
            window: None,
            model: model::Model::initial(),
            renderer: None,
            pointer_position: None,
            pointer_down: false,
        }
    }
}

impl FunctorsApp {
    fn dispatch(&mut self, event_loop: &ActiveEventLoop, message: mvu::Message) {
        let transition = mvu::update(self.model.clone(), message);
        self.model = transition.model;

        for command in transition.commands {
            match command {
                mvu::Command::RequestRedraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                mvu::Command::Exit => event_loop.exit(),
            }
        }
    }
}

impl ApplicationHandler for FunctorsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("Functors"))
                .expect("creating the Functors window should succeed"),
        );

        window.set_ime_allowed(true);
        let renderer = match pollster::block_on(renderer::Renderer::new(window.clone())) {
            Ok(renderer) => renderer,
            Err(error) => {
                eprintln!("failed to initialize WGPU: {error}");
                event_loop.exit();
                return;
            }
        };

        self.renderer = Some(renderer);
        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.dispatch(event_loop, mvu::Message::CloseRequested),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                self.dispatch(
                    event_loop,
                    mvu::Message::WindowResized {
                        width: size.width,
                        height: size.height,
                    },
                );
            }
            WindowEvent::Focused(false) => {
                self.pointer_down = false;
                self.pointer_position = None;
                self.dispatch(event_loop, mvu::Message::ModifiersChanged { shift: false });
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.dispatch(
                    event_loop,
                    mvu::Message::ModifiersChanged {
                        shift: modifiers.state().shift_key(),
                    },
                );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(message) = input::key_message(&event) {
                    self.dispatch(event_loop, message);
                }
                if let Some(message) = input::key_text_message(&event) {
                    self.dispatch(event_loop, message);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_position = Some((position.x as f32, position.y as f32));
                if self.pointer_down {
                    let position = view::position_at_point(
                        &self.model,
                        view::ViewConfig::default(),
                        position.x as f32,
                        position.y as f32,
                    );
                    self.dispatch(event_loop, mvu::Message::PointerDragged { position });
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                self.pointer_down = true;
                if let Some((x, y)) = self.pointer_position {
                    let position =
                        view::position_at_point(&self.model, view::ViewConfig::default(), x, y);
                    self.dispatch(event_loop, mvu::Message::PointerPressed { position });
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => self.pointer_down = false,
            WindowEvent::Ime(Ime::Commit(text)) => {
                if let Some(message) = input::text_message(text) {
                    self.dispatch(event_loop, message);
                }
            }
            WindowEvent::RedrawRequested => {
                let frame = view::build_frame(&self.model, view::ViewConfig::default());

                if let Some(renderer) = &mut self.renderer {
                    match renderer.render(&frame) {
                        renderer::RenderStatus::Presented | renderer::RenderStatus::Skipped => {}
                        renderer::RenderStatus::Reconfigure => {
                            if let Some(window) = &self.window {
                                let size = window.inner_size();
                                renderer.resize(size.width, size.height);
                            }
                        }
                    }
                }

                self.dispatch(event_loop, mvu::Message::RedrawRequested);
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop.run_app(&mut FunctorsApp::default())?;

    Ok(())
}
