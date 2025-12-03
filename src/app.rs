use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use crate::renderer::Renderer;

pub struct App {
    pub renderer: Renderer,
    event_loop: EventLoop<()>,
    window: winit::window::Window,
}

impl App {
    pub async fn new() -> Self {
        env_logger::init();
        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new().build(&event_loop).unwrap();

        let renderer = Renderer::new(&window).await;

        Self {
            renderer,
            event_loop,
            window,
        }
    }

    pub fn run(mut self) {
        self.event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape),
                                ..
                            },
                        ..
                    } => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(*physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        match self.renderer.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => self.renderer.resize(self.renderer.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        }).unwrap();
    }
}
