use std::{cell::RefCell, collections::HashSet, rc::Rc, sync::Arc};

use acro_ecs::{Application, World};
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
    inner: Option<Arc<winit::window::Window>>,
    state: Rc<RefCell<Option<RendererHandle>>>,
    application: Rc<RefCell<Option<Application>>>,
}

#[derive(Debug, Default)]
pub struct WindowState {
    pub mouse_position: Vec2,
    pub keys_pressed: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<winit::event::MouseButton>,
    pub ui_processed_click: bool,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::builder().build().unwrap();

        Self {
            event_loop: Some(event_loop),
            inner: None,
            state: Rc::new(RefCell::new(None)),
            application: Rc::new(RefCell::new(None)),
        }
    }

    pub fn run(mut self, application: Application) {
        *self.application.borrow_mut() = Some(application);
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

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{prelude::Closure, JsCast};
            use web_sys::Node;
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas().expect("canvas not found");
            let html_window = web_sys::window().expect("window not found");

            // TODO: Let people config their canvas element/size instead of defaulting to fullscreen
            let update_size = {
                let canvas = canvas.clone();
                let html_window = html_window.clone();
                let state = self.state.clone();

                move || {
                    let width = html_window
                        .inner_width()
                        .map_or(None, |v| v.as_f64())
                        .expect("unable to set canvas width")
                        as u32;
                    let height = html_window
                        .inner_height()
                        .map_or(None, |v| v.as_f64())
                        .expect("unable to set canvas height")
                        as u32;

                    canvas.set_width(width);
                    canvas.set_height(height);

                    state
                        .borrow_mut()
                        .as_ref()
                        .map(|state| state.resize(PhysicalSize::new(width, height)));
                }
            };

            update_size();

            let event_listener =
                gloo_events::EventListener::new(&html_window, "resize", move |_| update_size());
            event_listener.forget();

            // let resize_callback = Closure::<dyn FnMut()>::once_into_js(update_size);
            // html_window
            //     .add_event_listener_with_callback("resize", resize_callback)
            //     .expect("unable to attach resize callback");
            // resize_callback.forget();

            html_window
                .document()
                .and_then(|document| document.body())
                .and_then(|body| body.append_child(&Node::from(canvas)).ok())
                .expect("unable to append canvas to body");

            info!("added canvas to DOM");
        }

        window
    }

    async fn init_renderer_if_none(
        window: Arc<winit::window::Window>,
        state: Rc<RefCell<Option<RendererHandle>>>,
        application: Rc<RefCell<Option<Application>>>,
    ) {
        if state.borrow().is_some() {
            return;
        }

        let new_state = RendererState::new(window.clone()).await;

        application
            .borrow_mut()
            .as_mut()
            .expect("application not created")
            .world()
            .insert_resource(new_state.clone());

        *state.borrow_mut() = Some(new_state);

        info!("init_renderer_if_none done");
    }
}

impl ApplicationHandler for Window {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.inner.is_none() {
            let window = self.create_window(event_loop);
            window.set_visible(true);

            self.application
                .borrow_mut()
                .as_mut()
                .expect("application not created")
                .world()
                .insert_resource(WindowState::default());

            // On web platform, the renderer should not be created until a resize event has been
            // emitted. This is because the window's inner size is not set until a resize event happens
            // (and will cause the renderer to error if it tries to initialize with a size of 0x0).
            #[cfg(not(target_arch = "wasm32"))]
            pollster::block_on(Self::init_renderer_if_none(
                window.clone(),
                self.state.clone(),
                self.application.clone(),
            ));

            self.inner = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: event::WindowEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        if let &WindowEvent::Resized(_) = &event {
            let window = self.inner.clone().expect("window not created");
            wasm_bindgen_futures::spawn_local(Self::init_renderer_if_none(
                window.clone(),
                self.state.clone(),
                self.application.clone(),
            ));
            return;
        }

        let window = self.inner.as_ref().expect("window not created");

        let state = self.state.borrow();
        // If the renderer hasn't been created yet, keep waiting until the future is ready
        let state = match state.as_ref() {
            Some(state) => state,
            None => {
                if let WindowEvent::RedrawRequested = event {
                    window.request_redraw();
                }

                return;
            }
        };

        let mut application = self.application.borrow_mut();
        let application = application.as_mut().expect("application not created");

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
