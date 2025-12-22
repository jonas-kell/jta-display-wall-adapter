use crate::args::Args;
use crate::client::bitmap::png_to_bmp_bytes;
use crate::client::rasterizing::RasterizerMeta;
use crate::client::rendering::{render_client_frame, RenderCache};
use crate::client::{FRAME_TIME_NS, REPORT_FRAME_LOGS_EVERY_SECONDS, TARGET_FPS};
use crate::file::{create_file_if_not_there_and_write, make_sure_folder_exists};
use crate::interface::{ClientStateMachine, MessageFromClientToServer, MessageFromServerToClient};
use async_broadcast::{Sender as BroadcastSender, TrySendError};
use async_channel::{Receiver, Sender, TryRecvError};
use fontdue::layout::{CoordinateSystem, Layout};
use fontdue::{Font, FontSettings};
use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
#[cfg(target_os = "linux")]
use winit::platform::x11::{WindowAttributesExtX11, WindowType};
use winit::window::WindowAttributes;
use winit::window::{Window, WindowId};

pub fn run_display_task(
    args: Args,
    rx_to_ui: Receiver<MessageFromServerToClient>,
    tx_from_ui: BroadcastSender<MessageFromClientToServer>,
    tx_to_ui: Sender<MessageFromServerToClient>,
    shutdown_marker: Arc<AtomicBool>,
) -> () {
    // setup event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // as fast as possible, wil get overwritten to achieve stable fps

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
        incoming: rx_to_ui,
        outgoing: tx_from_ui,
        shutdown_marker: shutdown_marker,
        state_machine: ClientStateMachine::new(&args, tx_to_ui),
        last_draw_call: Instant::now(),
        draw_cache: RenderCache::new(),
    };
    let _ = event_loop.run_app(&mut app);
}

pub struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    font: Font,
    font_layout: Layout,
    args: Args,
    incoming: Receiver<MessageFromServerToClient>,
    outgoing: BroadcastSender<MessageFromClientToServer>,
    shutdown_marker: Arc<AtomicBool>,
    state_machine: ClientStateMachine,
    last_draw_call: Instant,
    draw_cache: RenderCache,
}

#[cfg(target_os = "linux")]
fn add_linux_specific_properties(window_attributes: WindowAttributes) -> WindowAttributes {
    window_attributes.with_x11_window_type([WindowType::Dialog].into())
}

#[cfg(not(target_os = "linux"))]
fn add_linux_specific_properties(window_attributes: WindowAttributes) -> WindowAttributes {
    window_attributes
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Desired size (pixels) and position (pixels from top-left of screen)
        let attrs = add_linux_specific_properties(
            Window::default_attributes()
                .with_title("JTA Display Window")
                // initial inner size
                .with_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
                // prevent user resizing by setting min and max to same size
                .with_min_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
                .with_max_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
                .with_decorations(false)
                .with_resizable(true)
                .with_position(PhysicalPosition::new(
                    self.args.dp_pos_x,
                    self.args.dp_pos_y,
                )),
        );

        let attrs = if self.args.do_not_set_client_window_always_on_top {
            attrs
        } else {
            attrs.with_window_level(winit::window::WindowLevel::AlwaysOnTop)
        };

        self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));

        // trigger the first re-awakening of the event loop
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(20), // hardcoded first frame delay, that seems to work good until stuff is initialized
        ));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // make sure to fire a redraw event, as that is where the delta time calculation is done
        if let Some(window) = &self.window {
            window.request_redraw();
        }

        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_nanos(FRAME_TIME_NS / 100), // make sure, the application does not go to sleep full as the application will not get mouse events
        ));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                warn!("The close button was pressed; Sadly that is not how this works");
            }
            WindowEvent::Resized(new_size) => {
                // this emits the original size on Wayland with sway window manager ?
                // https://github.com/rust-windowing/winit/issues/3485
                // https://github.com/rust-windowing/winit/pull/3602
                // I could not reproduce this (though I am not sure if it ever emitted the "wrong" dimensions)
                // BUT I added the property .with_resizable(true) above in window creation and at least since then on wayland sway, the resizing instantly returns success
                // (which is what is documented actually, as this event is only fired if display manager handles the resize request asynchronously)

                info!(
                    "The Window was resized to {}x{}",
                    new_size.width, new_size.height
                );

                if let Some((width, height)) = self.state_machine.current_frame_dimensions {
                    if width != new_size.width || height != new_size.height {
                        error!(
                            "The window tells it was resized to {}x{}, but we expected {}x{}",
                            new_size.width, new_size.height, width, height
                        );
                    } else {
                        warn!("The resize dimensions from manager match what was expected");
                    }
                }

                // force the values to what we know internally (as in our application there will never be an external resize (e.g. by mouse) anyway)
                // user the values from state_machine, not from the resize event
                self.re_initialize_pixels(new_size.width, new_size.height);
            }
            WindowEvent::Moved(p) => {
                debug!("The window was moved: {:?}", p);
            }
            WindowEvent::RedrawRequested => {
                // schedule next wakeup after we just finished a redraw session
                let now = Instant::now();
                let nano_since_last_draw_start =
                    now.duration_since(self.last_draw_call).as_nanos() as u64;
                let remaining_nanos = FRAME_TIME_NS.saturating_sub(nano_since_last_draw_start);

                if remaining_nanos == 0 {
                    // track at the start of the renderer and IO process (as there could be more requests for redrawing, than is actually redrawn)
                    self.last_draw_call = Instant::now();

                    // IO and state machine
                    self.process_state(event_loop);

                    if let Some(pixels) = &mut self.pixels {
                        let texture_size = pixels.texture().size();
                        let mut meta = RasterizerMeta {
                            font: &self.font,
                            font_layout: &mut self.font_layout,
                            frame: pixels.frame_mut(),
                            texture_width: texture_size.width as usize,
                            texture_height: texture_size.height as usize,
                            server_imposed_settings: self
                                .state_machine
                                .server_imposed_settings
                                .clone(),
                        };

                        render_client_frame(
                            &mut meta,
                            &mut self.state_machine,
                            &mut self.draw_cache,
                        );

                        let frame_count_to_emit: u64 = std::cmp::max(
                            (self.args.client_emits_frame_every_nr_of_ms * 1000000) / FRAME_TIME_NS,
                            1,
                        );

                        if !matches!(
                            self.state_machine.state,
                            crate::interface::ClientState::DisplayExternalFrame(_)
                        ) {
                            // if the frame is external, why bother sending it back
                            if self.state_machine.frame_counter % frame_count_to_emit == 0 {
                                trace!("Sending back frame to the server");
                                match meta.get_buffer_as_image() {
                                    Ok(img) => {
                                        let bytes = png_to_bmp_bytes(img);
                                        self.state_machine.push_new_message(
                                            MessageFromClientToServer::CurrentWindow(bytes),
                                        );
                                    }
                                    Err(e) => error!("Conversion error: {}", e),
                                }
                            }
                        }

                        // used in a log later (otherwise a mut-borrow problem)
                        let texture_width = meta.texture_width;
                        let texture_height = meta.texture_height;

                        // Render
                        match pixels.render() {
                            Ok(()) => {
                                if (self.state_machine.frame_counter + 111) // shoud not trigger together with the other nth-frame logs
                                    % (TARGET_FPS * REPORT_FRAME_LOGS_EVERY_SECONDS)
                                    == 0
                                {
                                    let percent = (Instant::now()
                                        .duration_since(self.last_draw_call)
                                        .as_nanos()
                                        as u64
                                        * 100)
                                        / FRAME_TIME_NS;
                                    trace!(
                                        "Pixels were re-rendered (reports all {}s as per frame count)",
                                        REPORT_FRAME_LOGS_EVERY_SECONDS
                                    );
                                    if let Some((sm_x, sm_y)) =
                                        self.state_machine.current_frame_dimensions
                                    {
                                        trace!(
                                            "Rendered a size of {}x{} - texture: {}x{}",
                                            sm_x,
                                            sm_y,
                                            texture_width,
                                            texture_height
                                        );
                                    }
                                    trace!(
                                        "Rendering a frame takes {}% of the max time to reach {}fps",
                                        percent,
                                        TARGET_FPS
                                    );
                                }
                            }
                            Err(e) => error!("Error while rendering: {}", e.to_string()),
                        }
                    } else {
                        if self.state_machine.frame_counter % TARGET_FPS == 0 {
                            warn!("The pixels element of the App context is not initialized");
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

impl App {
    fn process_state(&mut self, event_loop: &ActiveEventLoop) {
        // check for shutdown
        if self.shutdown_marker.load(Ordering::SeqCst) {
            info!("Shutdown requested, stopping display app");
            event_loop.exit();
        }

        // handle the frame counter of the state machine
        self.state_machine.advance_counters();

        // write out file to reposition window externally on wayland
        if let Some((x, y, w, h)) = self.state_machine.window_state_needs_update {
            if let Some(window) = &self.window {
                info!("Repositioning window: {} {}", x, y);
                window.set_outer_position(PhysicalPosition::new(x, y));
                if self.args.emit_file_on_location_update {
                    let path_folder = Path::new("move_container/");
                    let path_file = Path::new("move_container/coords.txt");
                    match make_sure_folder_exists(path_folder) {
                        Ok(_) => {
                            match create_file_if_not_there_and_write(
                                path_file,
                                &format!("{} {}", x, y),
                            ) {
                                Err(e) => error!("{}", e),
                                Ok(_) => {
                                    debug!("Position written to file");
                                }
                            }
                        }
                        Err(e) => {
                            error!("{}", e)
                        }
                    }
                }
                info!("Setting window size: {} {}", w, h);
                window.set_max_inner_size(Some(PhysicalSize::new(w, h)));
                window.set_min_inner_size(Some(PhysicalSize::new(w, h)));
                match window.request_inner_size(PhysicalSize::new(w, h)) {
                    None => debug!("Window resizing request went to the display system"), // this triggers the WindowEvent::Resized above
                    Some(size) => {
                        // if this is the same as before, it failed, if it is a different one, we were successful
                        info!(
                            "Window resizing request was answered with size: {}x{}",
                            size.width, size.height
                        );
                        self.re_initialize_pixels(size.width, size.height);
                    }
                }
                // update the state we think we have in the state machine
                self.state_machine.window_state_needs_update = None;
            }
        }

        // update connection status (avoid pushing out messages if we know they will be trashed)
        self.state_machine
            .set_outbound_connection_open(self.outgoing.receiver_count() > 0);

        // read incoming messages (we do not need to loop, as this is running at 60 fps anyway)
        match self.incoming.try_recv() {
            Ok(msg) => {
                self.state_machine.parse_server_command(msg);
            }
            Err(e) => match e {
                TryRecvError::Empty => (),
                e => {
                    error!(
                        "Error in inbound client internal communication: {}",
                        e.to_string()
                    );
                    event_loop.exit();
                }
            },
        };
        // send away outgoing messages (we do not need to loop, as this is running at 60 fps anyway)
        match self.state_machine.get_one_message_to_send() {
            Some(msg) => {
                match self.outgoing.try_broadcast(msg) {
                    Ok(Some(_)) => {
                        trace!("Thrown away old message in outgoing internal communication")
                    }
                    Ok(None) => (),
                    Err(TrySendError::Inactive(_)) => {
                        // to never spam log, none of these should be sent while no listeners there if possible (at least no polling)
                        warn!("Outbound internal channel not open, no active receivers");
                    }
                    Err(TrySendError::Full(_)) => {
                        error!("Receivers are there, but outbound internal channel full. This should not happen!");
                    }
                    Err(TrySendError::Closed(_)) => {
                        error!("Outbound internal channel went away unexpectedly");
                        event_loop.exit();
                    }
                };
            }
            None => (),
        };

        if self.state_machine.frame_counter % (TARGET_FPS * REPORT_FRAME_LOGS_EVERY_SECONDS) == 0 {
            trace!(
                "State was processed (reports all {}s as per frame count)",
                REPORT_FRAME_LOGS_EVERY_SECONDS
            );
        }
    }

    fn re_initialize_pixels(&mut self, width_to_use: u32, height_to_use: u32) {
        // Create every time (defer until window mapped) - resizing was not deemed successfull
        if let Some(window) = &self.window {
            let surface_texture = SurfaceTexture::new(width_to_use, height_to_use, window.clone());
            self.pixels = Some(Pixels::new(width_to_use, height_to_use, surface_texture).unwrap());
            debug!("Pixels were (re)-initialized");
        } else {
            error!("Window should be mapped by now. This is not possible...");
        }
        debug!("Setting state machine's knowledge about the window size");
        self.state_machine.current_frame_dimensions = Some((width_to_use, height_to_use));
    }
}
