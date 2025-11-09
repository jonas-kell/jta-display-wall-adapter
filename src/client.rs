use crate::args::Args;
use crate::rasterizing::{clear, draw_text, RasterizerMeta};
use fontdue::layout::{CoordinateSystem, Layout};
use fontdue::{Font, FontSettings};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub async fn run_client(args: &Args) -> () {
    // setup event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::WaitUntil(
        Instant::now() + Duration::from_millis(TARGET_FPS_DELAY_MS),
    ));

    // font setup
    let font_data = include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf") as &[u8];
    let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();
    let font_layout = Layout::new(CoordinateSystem::PositiveYDown);

    // run app
    let mut app = App {
        args: args.clone(),
        font: font,
        font_layout: font_layout,
        pixels: None,
        window: None,
    };
    let _ = event_loop.run_app(&mut app);
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    font: Font,
    font_layout: Layout,
    args: Args,
}

const TARGET_FPS_DELAY_MS: u64 = 16;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let _ = &self.args;

        // Desired size (pixels) and position (pixels from top-left of screen)
        let width = 640u32;
        let height = 360u32;
        let pos_x = 800i32;
        let pos_y = 400i32;

        let attrs = Window::default_attributes()
            .with_title("Display Window")
            // initial inner size
            .with_inner_size(PhysicalSize::new(width, height))
            // prevent user resizing by setting min and max to same size
            .with_min_inner_size(PhysicalSize::new(width, height))
            .with_max_inner_size(PhysicalSize::new(width, height))
            .with_decorations(false)
            .with_position(PhysicalPosition::new(pos_x, pos_y));

        self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));

        // pixels setup
        let pixels = {
            let window = self.window.clone().unwrap();
            let size = window.inner_size();
            let surface_texture = SurfaceTexture::new(size.width, size.height, window);
            Pixels::new(size.width, size.height, surface_texture).unwrap()
        };
        self.pixels = Some(pixels);

        // trigger the first re-awakening of the event loop
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(TARGET_FPS_DELAY_MS),
        ));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // called just before the event loop sleeps
        if let Some(window) = &self.window {
            window.request_redraw();
        }

        // schedule next wakeup after we just finished a redraw session
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(TARGET_FPS_DELAY_MS),
        ));
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; Sadly that is not how this works");
            }
            WindowEvent::RedrawRequested => {
                if let Some(pixels) = &mut self.pixels {
                    let texture_size = pixels.texture().size();
                    let mut meta = RasterizerMeta {
                        font: &self.font,
                        font_layout: &mut self.font_layout,
                        frame: pixels.frame_mut(),
                        texture_width: texture_size.width as usize,
                        texture_height: texture_size.height as usize,
                    };
                    // Draw area

                    clear(&mut meta);
                    draw_text("Hello world", 55.0, 22.0, 20.0, &mut meta);

                    // Render
                    match pixels.render() {
                        Ok(()) => (),
                        Err(e) => error!("Error while rendering: {}", e.to_string()),
                    }
                } else {
                    error!("The pixels element of the App context is not initialized")
                }
            }
            _ => (),
        }
    }
}
