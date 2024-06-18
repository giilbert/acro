use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
};

use crate::state::RendererState;

pub struct Window {
    event_loop: Option<EventLoop<()>>,
    window: Option<Arc<winit::window::Window>>,
    state: Option<RendererState>,
    update: Box<dyn FnMut()>,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::builder().build().unwrap();

        Self {
            event_loop: Some(event_loop),
            window: None,
            state: None,
            update: Box::new(|| panic!("no update set!")),
        }
    }

    pub fn run(mut self, update: impl FnMut() + 'static) {
        self.update = Box::new(update);
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
                .create_window(winit::window::Window::default_attributes().with_title("acro"))
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
        let state = self.state.as_mut().expect("state not created");

        match event {
            WindowEvent::RedrawRequested => {
                (self.update)();
                state.clear();
                window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => println!("unhandled event: {:?}", event),
        }
    }
}
