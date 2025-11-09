use crate::args::Args;
use fontdue::{Font, FontSettings};
use pixels::{Pixels, SurfaceTexture};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

#[derive(Default)]
struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    font: Option<Font>,
}

const TARGET_FPS_DELAY_MS: u64 = 16;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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

        // font setup
        let font_data =
            include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf") as &[u8];
        let font = Font::from_bytes(font_data, FontSettings::default()).unwrap();

        // store everything
        self.font = Some(font);

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

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(font), Some(pixels)) = (&self.font, &mut self.pixels) {
                    let tex_size = pixels.texture().size();
                    let frame = pixels.frame_mut();
                    frame.fill(0);

                    let text = "Hello, world!";
                    let mut x_cursor = 20;
                    let y0 = 40;
                    let tex_width = tex_size.width as usize;

                    for ch in text.chars() {
                        let (metrics, bitmap) = font.rasterize(ch, 32.0);

                        for y in 0..metrics.height {
                            for x in 0..metrics.width {
                                let px = bitmap[y * metrics.width + x];
                                let i = ((y0 + y) * tex_width + (x_cursor + x)) * 4;
                                if i + 3 < frame.len() {
                                    frame[i] = 255; // R
                                    frame[i + 1] = 255; // G
                                    frame[i + 2] = 255; // B
                                    frame[i + 3] = px; // alpha
                                }
                            }
                        }

                        x_cursor += metrics.advance_width as usize; // move cursor for next char
                    }

                    pixels.render().unwrap();
                }
            }
            _ => (),
        }
    }
}

pub async fn run_client(args: &Args) -> () {
    let _ = args;

    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::WaitUntil(
        Instant::now() + Duration::from_millis(TARGET_FPS_DELAY_MS),
    ));

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}
