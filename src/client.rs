use crate::args::Args;
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run_client(args: &Args) -> () {
    let _ = args;

    let event_loop = EventLoop::new();

    // Desired size (pixels) and position (pixels from top-left of screen)
    let width = 640u32;
    let height = 360u32;
    let pos_x = 800i32;
    let pos_y = 400i32;

    let window = WindowBuilder::new()
        .with_title("Display Window")
        // initial inner size
        .with_inner_size(PhysicalSize::new(width, height))
        // prevent user resizing by setting min and max to same size
        .with_min_inner_size(PhysicalSize::new(width, height))
        .with_max_inner_size(PhysicalSize::new(width, height))
        .with_decorations(false)
        .build(&event_loop)
        .unwrap();

    // Set outer position (some window managers may ignore or alter this)
    window.set_outer_position(PhysicalPosition::new(pos_x, pos_y));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    error!("Tried to close Window -> Cant allow that");
                }
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    // Could re-scale windo here. Probably not necessary though
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                // place any drawing / UI update calls here (none for an empty window)
            }
            _ => {}
        }
    });
}
