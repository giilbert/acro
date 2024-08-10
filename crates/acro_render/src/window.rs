use std::{collections::HashSet, sync::Arc};

use acro_ecs::Application;
use acro_math::{Float, Vec2};
use tracing::{info, warn};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{self, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};

use crate::state::{RendererHandle, RendererState};

pub struct Window {
    event_loop: Option<EventLoop<()>>,
    window: Option<Arc<winit::window::Window>>,
    state: Option<RendererHandle>,
    application: Option<Application>,
}

#[derive(Debug, Default)]
pub struct WindowState {
    pub mouse_position: Vec2,
    pub keys_pressed: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<winit::event::MouseButton>,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::builder().build().unwrap();

        Self {
            event_loop: Some(event_loop),
            window: None,
            state: None,
            application: None,
        }
    }

    pub fn run(mut self, application: Application) {
        self.application = Some(application);
        let event_loop = self.event_loop.take().unwrap();
        event_loop
            .run_app(&mut self)
            .expect("failed to run event loop");
    }
}

impl Window {
    fn create_window(&self, event_loop: &ActiveEventLoop) -> Arc<winit::window::Window> {
        let window = Arc::new(
            event_loop
                .create_window(
                    winit::window::Window::default_attributes()
                        .with_inner_size(PhysicalSize::new(800, 600))
                        .with_title("acro")
                        .with_visible(false),
                )
                .unwrap(),
        );

        window
    }
}

impl ApplicationHandler for Window {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = self.create_window(event_loop);
            let state = pollster::block_on(RendererState::new(window.clone()));
            window.set_visible(true);

            let mut world = self
                .application
                .as_mut()
                .expect("application not created")
                .world();

            world.insert_resource(state.clone());
            world.insert_resource(WindowState::default());

            self.window = Some(window);
            self.state = Some(state);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: event::WindowEvent,
    ) {
        let window = self.window.as_ref().expect("window not created");
        let state = self.state.as_ref().expect("state not created");
        let application = self.application.as_mut().expect("application not created");

        match event {
            WindowEvent::RedrawRequested => {
                application.run_once();
                window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                state.resize(size);
            }
            WindowEvent::CursorMoved {
                position,
                device_id: _device_id,
            } => {
                let world = &application.world();
                let mut window_state = world.resources().get_mut::<WindowState>();
                window_state.mouse_position = Vec2::new(position.x as Float, position.y as Float);
            }
            WindowEvent::MouseInput {
                device_id: _device_id,
                state,
                button,
            } => {
                let world = &application.world();
                let mut window_state = world.resources().get_mut::<WindowState>();

                if state == ElementState::Pressed {
                    window_state.mouse_buttons_pressed.insert(button);
                } else {
                    window_state.mouse_buttons_pressed.remove(&button);
                }
            }
            WindowEvent::KeyboardInput {
                event,
                device_id: _device_id,
                is_synthetic: _is_synthetic,
            } => {
                let world = &application.world();
                let mut window_state = world.resources().get_mut::<WindowState>();

                match event.physical_key {
                    PhysicalKey::Code(key_code) => {
                        if event.state == ElementState::Pressed {
                            window_state.keys_pressed.insert(key_code);
                        } else {
                            window_state.keys_pressed.remove(&key_code);
                        }
                    }
                    PhysicalKey::Unidentified(unknown_key) => {
                        warn!("unidentified key: {unknown_key:?}");
                    }
                }
            }
            _ => {
                // println!("unhandled event: {:?}", event);
            }
        }
    }
}
