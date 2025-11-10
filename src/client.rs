use crate::args::Args;
use crate::interface::{ClientStateMachine, MessageFromClientToServer, MessageFromServerToClient};
use crate::rasterizing::RasterizerMeta;
use crate::rendering::render_client_frame;
use async_channel::{Receiver, Sender, TryRecvError};
use fontdue::layout::{CoordinateSystem, Layout};
use fontdue::{Font, FontSettings};
use futures::prelude::*;
use pixels::{Pixels, SurfaceTexture};
use std::io::Error;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::time;
use tokio_serde::formats::*;
use tokio_serde::Framed;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

pub async fn run_client(args: &Args) -> () {
    let (tx_to_ui, rx_to_ui) = async_channel::unbounded::<MessageFromServerToClient>();
    let (tx_from_ui, rx_from_ui) = async_channel::unbounded::<MessageFromClientToServer>();

    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let network_task = tokio::spawn(run_network_task(
        args.clone(),
        tx_to_ui,
        rx_from_ui,
        Arc::clone(&shutdown_marker),
    ));
    let shutdown_marker_sdt = Arc::clone(&shutdown_marker);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker_sdt.store(true, Ordering::SeqCst);

        Ok::<_, Error>(())
    });

    // async runtime stuff started, the display task doesn't like being inside tokio, so it comes after and takes shutdown orders via Arc
    tokio::spawn(async move {
        match tokio::try_join!(network_task, shutdown_task) {
            Err(_) => error!("Error in at least one client task"),
            Ok(_) => info!("All client tasks closed successfully"),
        };
    });

    run_display_task(
        args.clone(),
        rx_to_ui,
        tx_from_ui,
        Arc::clone(&shutdown_marker),
    );
}

pub async fn run_network_task(
    args: Args,
    tx_to_ui: Sender<MessageFromServerToClient>,
    rx_from_ui: Receiver<MessageFromClientToServer>,
    shutdown_marker: Arc<AtomicBool>,
) -> Result<(), Error> {
    let listen_addr: SocketAddr = format!("0.0.0.0:{}", args.internal_communication_port)
        .parse()
        .expect("Invalid internal communication address");

    let listener = TcpListener::bind(listen_addr).await?;
    debug!("TCP listener started on {}", listen_addr);

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            debug!("Shutdown requested, stopping listener on {}", listen_addr);
            break;
        }

        // Wait for new connection with timeout so we can check shutdown flag periodically
        match time::timeout(
            Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
            listener.accept(),
        )
        .await
        {
            Ok(Ok((inbound, client_addr))) => {
                debug!("Accepted connection from {}", client_addr);

                let (read_half, write_half) = inbound.into_split();
                let mut deserializer: Framed<
                    _,
                    MessageFromServerToClient,
                    MessageFromClientToServer,
                    _,
                > = Framed::new(
                    FramedRead::new(read_half, LengthDelimitedCodec::new()),
                    Bincode::<MessageFromServerToClient, MessageFromClientToServer>::default(),
                );
                let mut serializer: Framed<
                    _,
                    MessageFromServerToClient,
                    MessageFromClientToServer,
                    _,
                > = Framed::new(
                    FramedWrite::new(write_half, LengthDelimitedCodec::new()),
                    Bincode::<MessageFromServerToClient, MessageFromClientToServer>::default(),
                );

                // Connection is accepted. Handle all further in own task

                let shutdown_marker = shutdown_marker.clone();
                let tx_to_ui = tx_to_ui.clone();
                let rx_from_ui = rx_from_ui.clone();

                tokio::spawn(async move {
                    let shutdown_marker_read = shutdown_marker.clone();

                    let read_handler = async move {
                        loop {
                            if shutdown_marker_read.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main external -> self transfer"
                                );
                                break;
                            }

                            match time::timeout(
                                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                                deserializer.next(),
                            )
                            .await
                            {
                                Err(_) => {
                                    trace!("No new TCP traffic within timeout interval");
                                    continue;
                                }
                                Ok(None) => return Err("TCP stream went away".into()),
                                Ok(Some(Err(e))) => return Err(e.to_string()),
                                Ok(Some(Ok(mes))) => match tx_to_ui.send(mes).await {
                                    Ok(()) => trace!(
                                        "Message taken in and forwarded into internal comm channel"
                                    ),
                                    Err(e) => return Err(e.to_string()),
                                },
                            }
                        }
                        Ok::<_, String>(())
                    };

                    let shutdown_marker_write = shutdown_marker;

                    let write_handler = async move {
                        loop {
                            if shutdown_marker_write.load(Ordering::SeqCst) {
                                debug!(
                                    "Shutdown marker set, breaking main self -> external transfer"
                                );
                                break;
                            }

                            match time::timeout(
                                Duration::from_millis(args.wait_ms_before_testing_for_shutdown),
                                rx_from_ui.recv(),
                            )
                            .await
                            {
                                Err(_) => {
                                    trace!("No new Messages to send out within timeout interval");
                                    continue;
                                }
                                Ok(Err(e)) => return Err(e.to_string()),
                                Ok(Ok(mes)) => match serializer.send(mes).await {
                                    Ok(()) => trace!(
                                        "TCP sender forwarded message from internal comm channel"
                                    ),
                                    Err(e) => return Err(e.to_string()),
                                },
                            }
                        }

                        Ok::<_, String>(())
                    };

                    tokio::try_join!(read_handler, write_handler)?;

                    Ok::<_, String>(())
                });
            }
            Ok(Err(e)) => error!("Accept error: {}", e),
            Err(_) => {
                // expected on timeout, just loop
                trace!("No new TCP connection within timeout interval");
            }
        }
    }

    Ok(())
}

pub fn run_display_task(
    args: Args,
    rx_to_ui: Receiver<MessageFromServerToClient>,
    tx_from_ui: Sender<MessageFromClientToServer>,
    shutdown_marker: Arc<AtomicBool>,
) -> () {
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
        args: args,
        font: font,
        font_layout: font_layout,
        pixels: None,
        window: None,
        incoming: rx_to_ui,
        outgoing: tx_from_ui,
        shutdown_marker: shutdown_marker,
        state_machine: ClientStateMachine::new(),
    };
    let _ = event_loop.run_app(&mut app);
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    font: Font,
    font_layout: Layout,
    args: Args,
    incoming: Receiver<MessageFromServerToClient>,
    outgoing: Sender<MessageFromClientToServer>,
    shutdown_marker: Arc<AtomicBool>,
    state_machine: ClientStateMachine,
}

const TARGET_FPS_DELAY_MS: u64 = 16;

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Desired size (pixels) and position (pixels from top-left of screen)
        let attrs = Window::default_attributes()
            .with_title("JTA Display Window")
            // initial inner size
            .with_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
            // prevent user resizing by setting min and max to same size
            .with_min_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
            .with_max_inner_size(PhysicalSize::new(self.args.dp_width, self.args.dp_height))
            .with_decorations(false)
            .with_position(PhysicalPosition::new(
                self.args.dp_pos_x,
                self.args.dp_pos_y,
            ));

        self.window = Some(Arc::new(event_loop.create_window(attrs).unwrap()));

        // trigger the first re-awakening of the event loop
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(TARGET_FPS_DELAY_MS),
        ));
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // check for shutdown
        if self.shutdown_marker.load(Ordering::SeqCst) {
            debug!("Shutdown requested, stopping display app");
            event_loop.exit();
        }

        if let Some((x, y, w, h)) = self.state_machine.window_state_needs_update {
            if let Some(window) = &self.window {
                info!("Repositioning window: {} {}", x, y);
                window.set_outer_position(PhysicalPosition::new(x, y));
                info!("Setting window size: {} {}", w, h);
                window.set_max_inner_size(Some(PhysicalSize::new(w, h)));
                window.set_min_inner_size(Some(PhysicalSize::new(w, h)));
                match window.request_inner_size(PhysicalSize::new(w, h)) {
                    None => info!("Window resizing request went to the display system"),
                    Some(size) => info!("Window resizing request: {:?}", size),
                }
                self.state_machine.window_state_needs_update = None;
            }
        }

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
                match self.outgoing.try_send(msg) {
                    Ok(()) => (),
                    Err(e) => {
                        error!(
                            "Error in outbound client internal communication: {}",
                            e.to_string()
                        );
                        event_loop.exit();
                    }
                };
            }
            None => (),
        };

        // called just before the event loop sleeps
        if let Some(window) = &self.window {
            window.request_redraw();
        }

        self.state_machine.advance_counters();

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
            WindowEvent::Resized(new_size) => {
                info!("The Window was resized: {:?}", new_size);

                // pixels setup needs to be redone (only here!!)
                if let Some(pixels) = &mut self.pixels {
                    match pixels.resize_surface(new_size.width, new_size.height) {
                        Ok(()) => debug!("Pixels were resized"),
                        Err(e) => error!("Failed to resize pixels surface: {}", e),
                    }
                } else {
                    // first-time creation (defer until window mapped)
                    if let Some(window) = &self.window {
                        let surface_texture =
                            SurfaceTexture::new(new_size.width, new_size.height, window.clone());
                        self.pixels = Some(
                            Pixels::new(new_size.width, new_size.height, surface_texture).unwrap(),
                        );
                        debug!("Pixels were initialized");
                    } else {
                        error!("Window should be mapped by now. This is not possible...");
                    }
                }
            }
            WindowEvent::Moved(p) => {
                info!("The window was moved: {:?}", p);
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

                    render_client_frame(&mut meta, &self.state_machine);

                    // TODO send frame back (uses counter)

                    // Render
                    match pixels.render() {
                        Ok(()) => (),
                        Err(e) => error!("Error while rendering: {}", e.to_string()),
                    }
                } else {
                    if self.state_machine.log_pixels_not_initialized {
                        warn!("The pixels element of the App context is not initialized");
                        self.state_machine.log_pixels_not_initialized = false;
                    }
                }
            }
            _ => (),
        }
    }
}
