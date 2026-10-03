use std::error::Error;
use std::sync::Arc;

mod file_io;
mod input;
mod model;
mod mvu;
#[allow(dead_code)]
mod parley_text;
mod renderer;
mod syntax;
#[allow(dead_code)]
mod text;
#[allow(dead_code)]
mod vello_scene;
mod view;
#[allow(dead_code)]
mod widgets;
mod workspace;

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
    scrollbar_drag: Option<ScrollbarDrag>,
    control_down: bool,
}

struct ScrollbarDrag {
    axis: input::ScrollbarAxis,
    last_position: f32,
    remainder: f32,
}

fn clamp_scroll_delta(current: usize, requested: i32, maximum: usize) -> i32 {
    let current = current.min(maximum);
    if requested >= 0 {
        requested.min(maximum.saturating_sub(current).min(i32::MAX as usize) as i32)
    } else {
        -((-requested as i64).min(current as i64) as i32)
    }
}

fn bounded_scroll_message(model: &model::Model, vertical: i32, horizontal: i32) -> mvu::Message {
    let limits = view::scroll_limits(model, view::ViewConfig::default());
    mvu::Message::Scrolled {
        vertical: clamp_scroll_delta(model.viewport.vertical_offset, vertical, limits.0),
        horizontal: clamp_scroll_delta(model.viewport.horizontal_offset, horizontal, limits.1),
    }
}

impl Default for FunctorsApp {
    fn default() -> Self {
        Self {
            window: None,
            model: model::Model::initial(),
            renderer: None,
            pointer_position: None,
            pointer_down: false,
            scrollbar_drag: None,
            control_down: false,
        }
    }
}

impl FunctorsApp {
    fn dispatch_input_message(&mut self, event_loop: &ActiveEventLoop, message: mvu::Message) {
        match message {
            mvu::Message::OpenWorkspacePickerRequested => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open Workspace")
                    .pick_folder()
                {
                    self.dispatch(event_loop, mvu::Message::OpenWorkspaceRequested(path));
                }
            }
            mvu::Message::OpenFilePickerRequested => {
                if let Some(path) = rfd::FileDialog::new().set_title("Open File").pick_file() {
                    self.dispatch(event_loop, mvu::Message::OpenFileRequested(path));
                }
            }
            message => self.dispatch(event_loop, message),
        }
    }

    fn dispatch(&mut self, event_loop: &ActiveEventLoop, message: mvu::Message) {
        let transition = mvu::update(self.model.clone(), message);
        self.model = transition.model;
        self.update_window_title();

        for command in transition.commands {
            match command {
                mvu::Command::RequestRedraw => {
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
                mvu::Command::Exit => event_loop.exit(),
                mvu::Command::OpenWorkspace(path) => {
                    let result = workspace::discover(path);
                    self.dispatch(event_loop, mvu::Message::WorkspaceOpened(result));
                }
                mvu::Command::OpenFile(path) => {
                    let result = file_io::load(&path);
                    self.dispatch(event_loop, mvu::Message::FileOpened(result));
                }
                mvu::Command::SaveFile {
                    path,
                    text,
                    expected_stamp,
                } => {
                    let result = file_io::save_if_unchanged(&path, &text, expected_stamp.as_ref());
                    self.dispatch(event_loop, mvu::Message::FileSaved(result));
                }
                mvu::Command::OpenWorkspacePicker => {
                    self.dispatch(event_loop, mvu::Message::OpenWorkspacePickerRequested);
                }
                mvu::Command::OpenFilePicker => {
                    self.dispatch(event_loop, mvu::Message::OpenFilePickerRequested);
                }
            }
        }
    }

    fn update_window_title(&self) {
        if let Some(window) = &self.window {
            window.set_title(&window_title(&self.model));
        }
    }
}

fn window_title(model: &model::Model) -> String {
    let state = match model.state {
        model::EditorState::Error => "Error".to_owned(),
        model::EditorState::Loading => "Loading...".to_owned(),
        model::EditorState::Active => model
            .file_path
            .as_ref()
            .map(|path| {
                format!(
                    "{}{}",
                    if model.document.dirty { "* " } else { "" },
                    path.display()
                )
            })
            .unwrap_or_else(|| "Untitled".to_owned()),
        model::EditorState::Empty => model
            .workspace
            .as_ref()
            .map(|workspace| workspace.root.display().to_string())
            .unwrap_or_else(|| "Untitled".to_owned()),
    };

    format!("Functors - {state}")
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
        if let Ok(root) = std::env::current_dir() {
            self.dispatch(event_loop, mvu::Message::OpenWorkspaceRequested(root));
        }
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
                self.scrollbar_drag = None;
                self.pointer_position = None;
                self.control_down = false;
                self.dispatch(event_loop, mvu::Message::ModifiersChanged { shift: false });
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                self.control_down = modifiers.state().control_key();
                self.dispatch(
                    event_loop,
                    mvu::Message::ModifiersChanged {
                        shift: modifiers.state().shift_key(),
                    },
                );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(message) = input::key_message(&event, self.control_down) {
                    self.dispatch_input_message(event_loop, message);
                }
                if let Some(message) = input::key_text_message(&event, self.control_down) {
                    self.dispatch_input_message(event_loop, message);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_position = Some((position.x as f32, position.y as f32));
                let message = if let Some(drag) = &mut self.scrollbar_drag {
                    let current_position = match drag.axis {
                        input::ScrollbarAxis::Vertical => position.y as f32,
                        input::ScrollbarAxis::Horizontal => position.x as f32,
                    };
                    drag.remainder += current_position - drag.last_position;
                    drag.last_position = current_position;
                    let units = (drag.remainder / 10.0).trunc() as i32;
                    drag.remainder -= units as f32 * 10.0;
                    let limits = view::scroll_limits(&self.model, view::ViewConfig::default());
                    let delta = match drag.axis {
                        input::ScrollbarAxis::Vertical => {
                            clamp_scroll_delta(self.model.viewport.vertical_offset, units, limits.0)
                        }
                        input::ScrollbarAxis::Horizontal => clamp_scroll_delta(
                            self.model.viewport.horizontal_offset,
                            units,
                            limits.1,
                        ),
                    };
                    (delta != 0).then(|| match drag.axis {
                        input::ScrollbarAxis::Vertical => mvu::Message::Scrolled {
                            vertical: delta,
                            horizontal: 0,
                        },
                        input::ScrollbarAxis::Horizontal => mvu::Message::Scrolled {
                            vertical: 0,
                            horizontal: delta,
                        },
                    })
                } else if self.pointer_down {
                    let position = view::position_at_point(
                        &self.model,
                        view::ViewConfig::default(),
                        position.x as f32,
                        position.y as f32,
                    );
                    Some(mvu::Message::PointerDragged { position })
                } else {
                    None
                };
                if let Some(message) = message {
                    self.dispatch(event_loop, message);
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (vertical, horizontal) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => {
                        (-(y.round() as i32), -(x.round() as i32))
                    }
                    winit::event::MouseScrollDelta::PixelDelta(position) => (
                        -(position.y / 20.0).round() as i32,
                        -(position.x / 10.0).round() as i32,
                    ),
                };
                self.dispatch(
                    event_loop,
                    bounded_scroll_message(&self.model, vertical, horizontal),
                );
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if let Some((x, y)) = self.pointer_position {
                    let frame = view::build_frame(&self.model, view::ViewConfig::default());
                    if let Some(axis) = input::scrollbar_thumb_axis(&frame, x, y) {
                        self.pointer_down = false;
                        self.scrollbar_drag = Some(ScrollbarDrag {
                            axis,
                            last_position: match axis {
                                input::ScrollbarAxis::Vertical => y,
                                input::ScrollbarAxis::Horizontal => x,
                            },
                            remainder: 0.0,
                        });
                    } else if y < 24.0 {
                        if let Some(message) = input::menu_message(x, y) {
                            self.dispatch_input_message(event_loop, message);
                        }
                    } else if let Some(message) = input::scrollbar_message(&frame, x, y) {
                        self.dispatch(event_loop, message);
                    } else {
                        self.pointer_down = true;
                        let position =
                            view::position_at_point(&self.model, view::ViewConfig::default(), x, y);
                        self.dispatch(event_loop, mvu::Message::PointerPressed { position });
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                self.pointer_down = false;
                self.scrollbar_drag = None;
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                if let Some(message) = input::text_message(text) {
                    self.dispatch(event_loop, message);
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
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
                                window.request_redraw();
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

#[cfg(test)]
mod tests {
    use super::{clamp_scroll_delta, window_title};
    use crate::file_io::FileError;
    use crate::model::{EditorState, Model};
    use crate::vello_scene::build_scene;
    use crate::view::{ViewConfig, build_frame};
    use std::path::PathBuf;
    use std::time::Instant;

    #[test]
    fn title_shows_active_file_and_dirty_indicator() {
        let mut model = Model {
            file_path: Some(PathBuf::from("src/main.rs")),
            state: EditorState::Active,
            ..Default::default()
        };
        model.document.insert_text(&mut model.cursor, "changed");

        assert_eq!(window_title(&model), "Functors - * src/main.rs");
    }

    #[test]
    fn title_shows_error_state_before_other_context() {
        let model = Model {
            file_path: Some(PathBuf::from("src/main.rs")),
            state: EditorState::Error,
            file_error: Some(FileError::InvalidUtf8 {
                path: PathBuf::from("src/main.rs"),
            }),
            ..Default::default()
        };

        assert_eq!(window_title(&model), "Functors - Error");
    }

    #[test]
    #[ignore = "run explicitly to refresh the local performance baseline"]
    fn reports_document_edit_and_layout_baseline() {
        let source = (0..10_000)
            .map(|line| format!("line {line:05} with representative editor text\n"))
            .collect::<String>();
        let mut model = Model::default();
        model.document.insert_text(&mut model.cursor, &source);
        model.viewport.width = 1200;
        model.viewport.height = 800;

        let edit_start = Instant::now();
        for _ in 0..100 {
            let mut edited = model.clone();
            let mut position = crate::model::Position {
                line: 5_000,
                column: 10,
            };
            edited.document.insert_text(&mut position, "x");
            std::hint::black_box(edited);
        }
        let edit_elapsed = edit_start.elapsed();

        let layout_start = Instant::now();
        for _ in 0..100 {
            std::hint::black_box(build_frame(&model, ViewConfig::default()));
        }
        let layout_elapsed = layout_start.elapsed();

        let frame = build_frame(&model, ViewConfig::default());
        let scene_start = Instant::now();
        for _ in 0..100 {
            std::hint::black_box(build_scene(&frame));
        }
        let scene_elapsed = scene_start.elapsed();

        println!(
            "performance baseline: 100 edits={edit_elapsed:?}, 100 layouts={layout_elapsed:?}, 100 vello scenes={scene_elapsed:?}"
        );
    }

    #[test]
    fn scrollbar_drag_deltas_are_clamped_to_document_limits() {
        assert_eq!(clamp_scroll_delta(0, -4, 10), 0);
        assert_eq!(clamp_scroll_delta(8, 5, 10), 2);
        assert_eq!(clamp_scroll_delta(8, -5, 10), -5);
    }
}
