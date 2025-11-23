use crate::{
    args::Args,
    database::{get_log_limited, DatabaseManager, DatabaseSerializable},
    file::read_image_files,
    instructions::{
        IncomingInstruction, InstructionCommunicationChannel, InstructionFromCameraProgram,
        InstructionFromTimingProgram, InstructionToTimingProgram,
    },
    server::xml_types::HeatStart,
    webserver::{DisplayClientState, MessageFromWebControl, MessageToWebControl},
};
use images_core::images::{AnimationPlayer, ImageMeta, ImagesStorage};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerImposedSettings {
    position: (u32, u32, u32, u32),
    slideshow_duration_in_ms: u32,
    slideshow_transition_duration_nr_ms: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServerToClient {
    DisplayText(String),
    RequestVersion,
    ServerImposedSettings(ServerImposedSettings),
    Clear,
    DisplayExternalFrame(Vec<u8>),
    AdvertisementImages(Vec<(String, Vec<u8>)>),
    Advertisements,
    Timing,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClientToServer {
    Version(String),
    CurrentWindow(Vec<u8>),
}

#[derive(PartialEq, Eq, Clone)]
pub enum ServerState {
    PassthroughClient,
    PassthroughDisplayProgram,
}

macro_rules! store_to_database {
    ($value:expr, $self_val:expr) => {
        match $value.store_to_database(&$self_val.database_manager) {
            Ok(()) => {
                trace!("Success, we stored an instruction into the database");
            }
            Err(e) => {
                error!("Database storage error: {}", e);
            }
        }
        $self_val.send_out_latest_n_logs_to_webclient(1);
    };
}

pub struct ServerStateMachine {
    args: Args,
    pub state: ServerState,
    comm_channel: InstructionCommunicationChannel,
    display_connected: bool,
    database_manager: DatabaseManager,
}
impl ServerStateMachine {
    pub fn new(
        args: &Args,
        comm_channel: InstructionCommunicationChannel,
        database_manager: DatabaseManager,
    ) -> Self {
        Self {
            args: args.clone(),
            state: ServerState::PassthroughClient,
            comm_channel, // only used to send instructions outwards. Rest is done via incoming commands (there is a handler that continously takes them out of the channel and forwards them into us)
            display_connected: false,
            database_manager,
        }
    }

    pub async fn parse_client_command(&mut self, msg: MessageFromClientToServer) {
        // handle all messages
        match msg {
            MessageFromClientToServer::Version(version) => {
                error!("Client reported to have version: '{}'", version); // TODO compare and error if not compatible

                // report the timing program our fake version
                self.send_message_to_timing_program(InstructionToTimingProgram::SendServerInfo);

                // get client to respect window position and size
                debug!(
                    "Requesting window change on client: {} {} {} {}",
                    self.args.dp_pos_x, self.args.dp_pos_y, self.args.dp_width, self.args.dp_height,
                );
                self.send_message_to_client(MessageFromServerToClient::ServerImposedSettings(
                    ServerImposedSettings {
                        position: (
                            self.args.dp_pos_x,
                            self.args.dp_pos_y,
                            self.args.dp_width,
                            self.args.dp_height,
                        ),
                        slideshow_duration_in_ms: self.args.slideshow_duration_nr_ms,
                        slideshow_transition_duration_nr_ms: self
                            .args
                            .slideshow_transition_duration_nr_ms,
                    },
                ));

                // send client advertisement images
                let folder_path = Path::new("advertisement_container");
                let images_data = match read_image_files(folder_path) {
                    Err(e) => {
                        error!("Could not read advertisement files: {}", e);
                        Vec::new()
                    }
                    Ok(data) => data,
                };
                self.send_message_to_client(MessageFromServerToClient::AdvertisementImages(
                    images_data,
                ));
            }
            MessageFromClientToServer::CurrentWindow(data) => {
                if self.state == ServerState::PassthroughClient {
                    self.send_message_to_timing_program(InstructionToTimingProgram::SendFrame(
                        data,
                    ));
                }
                // ping the state to the web control
                self.send_message_to_web_control(MessageToWebControl::DisplayClientState(
                    self.get_display_client_state(),
                ));
            }
        }
    }

    pub async fn parse_incoming_command(&mut self, msg: IncomingInstruction) {
        // handle all messages
        match msg {
            IncomingInstruction::FromTimingProgram(inst) => match inst {
                InstructionFromTimingProgram::ClientInfo => (),
                InstructionFromTimingProgram::ServerInfo => (),
                InstructionFromTimingProgram::Freetext(text) => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::DisplayText(text))
                    }
                }
                InstructionFromTimingProgram::Clear => {
                    if self.args.passthrough_to_display_program {
                        match self.state {
                            ServerState::PassthroughClient => {
                                info!("Now passing through client");
                                self.state = ServerState::PassthroughDisplayProgram
                            }
                            ServerState::PassthroughDisplayProgram => {
                                info!("Now passing through external display program");
                                self.state = ServerState::PassthroughClient
                            }
                        }
                        self.send_message_to_client(MessageFromServerToClient::Clear);
                    } else {
                        self.state = ServerState::PassthroughClient;
                    }
                }
                InstructionFromTimingProgram::SendFrame(data) => {
                    // this is a bit of a hack, because technically this message comes from the display program
                    trace!("Got command to send a frame inbound (should be from external display program to send back and possibly proxy to our client)");
                    if self.state == ServerState::PassthroughDisplayProgram {
                        self.send_message_to_client(
                            MessageFromServerToClient::DisplayExternalFrame(data),
                        );
                    }
                }
                InstructionFromTimingProgram::Advertisements => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Advertisements);
                    }
                }
                InstructionFromTimingProgram::Timing => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                    }
                }
                inst => error!("Unhandled instruction from timing program: {}", inst),
            },
            IncomingInstruction::FromCameraProgram(inst) => match inst {
                InstructionFromCameraProgram::HeatStartList(list) => {
                    store_to_database!(list, self);
                }
                InstructionFromCameraProgram::HeatStart(start) => {
                    store_to_database!(start, self);
                }
                InstructionFromCameraProgram::HeatFalseStart(false_start) => {
                    store_to_database!(false_start, self);
                }
                inst => error!("Unhandled instruction from camera program: {:?}", inst),
            },
            IncomingInstruction::FromWebControl(inst) => match inst {
                MessageFromWebControl::Advertisements => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Advertisements);
                    }
                }
                MessageFromWebControl::FreeText(text) => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::DisplayText(text))
                    }
                }
                MessageFromWebControl::Idle => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Clear)
                    }
                }
                MessageFromWebControl::RequestDisplayClientState => {
                    self.send_message_to_web_control(MessageToWebControl::DisplayClientState(
                        self.get_display_client_state(),
                    ));
                }
                MessageFromWebControl::SwitchMode => {
                    if self.args.passthrough_to_display_program {
                        match self.state {
                            ServerState::PassthroughClient => {
                                info!("Now passing through client");
                                self.state = ServerState::PassthroughDisplayProgram
                            }
                            ServerState::PassthroughDisplayProgram => {
                                info!("Now passing through external display program");
                                self.state = ServerState::PassthroughClient
                            }
                        }
                        self.send_message_to_client(MessageFromServerToClient::Clear);
                    } else {
                        self.state = ServerState::PassthroughClient;
                    }
                }
                MessageFromWebControl::GetHeatStarts => {
                    trace!("Heat Starts were requested");
                    match HeatStart::get_all_from_database(&self.database_manager) {
                        Ok(data) => {
                            self.send_message_to_web_control(MessageToWebControl::HeatStarts(data));
                        }
                        Err(e) => {
                            error!("Database loading error: {}", e);
                        }
                    }
                }
                MessageFromWebControl::GetLogs(how_many) => {
                    trace!("{} Logs were requested", how_many);
                    self.send_out_latest_n_logs_to_webclient(how_many);
                }
            },
        }
    }

    fn send_out_latest_n_logs_to_webclient(&mut self, n: u32) {
        match get_log_limited(Some(n), &self.database_manager) {
            Ok(data) => {
                self.send_message_to_web_control(MessageToWebControl::Logs(data));
            }
            Err(e) => {
                error!("Database log loading error: {}", e);
            }
        }
    }

    fn get_display_client_state(&self) -> DisplayClientState {
        DisplayClientState {
            alive: self.display_connected,
            external_passthrough_mode: self.state == ServerState::PassthroughDisplayProgram,
            can_switch_mode: self.args.passthrough_to_display_program,
        }
    }

    fn send_message_to_timing_program(&mut self, inst: InstructionToTimingProgram) {
        match self.comm_channel.send_out_command_to_timing_program(inst) {
            Ok(()) => (),
            Err(e) => error!("Failed to send out instruction: {}", e.to_string()),
        }
    }

    pub async fn make_server_request_client_version(&mut self) {
        self.send_message_to_client(MessageFromServerToClient::RequestVersion)
    }

    fn send_message_to_client(&mut self, inst: MessageFromServerToClient) {
        match self.comm_channel.send_out_command_to_client(inst) {
            Ok(()) => (),
            Err(e) => error!(
                "Failed to send out instruction to client: {}",
                e.to_string()
            ),
        }
    }

    fn send_message_to_web_control(&mut self, inst: MessageToWebControl) {
        match self.comm_channel.send_out_command_to_web_control(inst) {
            Ok(()) => (),
            Err(e) => error!(
                "Failed to send out instruction to web control: {}",
                e.to_string()
            ),
        }
    }

    pub fn set_main_display_state(&mut self, state: bool) {
        self.display_connected = state;
    }
}

pub enum ClientState {
    Created,
    Idle,
    DisplayText(String),
    DisplayExternalFrame(ImageMeta),
    Advertisements,
    TestAnimation(AnimationPlayer),
}

static STORAGE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/image_storage.bin"));

pub struct ClientStateMachine {
    pub state: ClientState,
    messages_to_send_out_to_server: Vec<MessageFromClientToServer>,
    pub frame_counter: u64,
    pub window_state_needs_update: Option<(u32, u32, u32, u32)>,
    pub permanent_images_storage: ImagesStorage,
    pub current_frame_dimensions: Option<(u32, u32)>,
    pub slideshow_duration_nr_ms: u32,
    pub slideshow_transition_duration_nr_ms: u32,
}
impl ClientStateMachine {
    pub fn new(args: &Args) -> Self {
        debug!("Start loading Images storage");
        let images_storage = ImagesStorage::from_bytes(STORAGE_BYTES);
        debug!("DONE loading Images storage");

        Self {
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
            window_state_needs_update: None,
            permanent_images_storage: images_storage,
            current_frame_dimensions: None,
            slideshow_duration_nr_ms: args.slideshow_duration_nr_ms,
            slideshow_transition_duration_nr_ms: args.slideshow_transition_duration_nr_ms,
        }
    }

    pub fn parse_server_command(&mut self, msg: MessageFromServerToClient) {
        match msg {
            MessageFromServerToClient::RequestVersion => {
                info!("Version was requested. Communication established!!");
                if matches!(self.state, ClientState::Created) {
                    self.state = ClientState::Idle;
                }
                self.push_new_message(MessageFromClientToServer::Version(String::from(
                    "TODO: THIS SHOULD BE COMPUTED",
                )))
            }
            MessageFromServerToClient::DisplayText(text) => {
                debug!("Server requested display mode to be switched to text");
                self.state = ClientState::DisplayText(text)
            }
            MessageFromServerToClient::ServerImposedSettings(settings) => {
                let (x, y, w, h) = settings.position;
                debug!("Server requested an update of the window position/size");
                self.window_state_needs_update = Some((x, y, w, h));

                debug!(
                    "Server set the slideshow duration to {} ms",
                    settings.slideshow_duration_in_ms
                );
                self.slideshow_duration_nr_ms = settings.slideshow_duration_in_ms;

                // do not forget to do this on size changes -> TODO maybe extract somewhere or do in true window resize handler
                debug!("Cache rescaling Animations");
                self.permanent_images_storage
                    .fireworks_animation
                    .cache_animation_for_size(
                        w,
                        h,
                        &mut self.permanent_images_storage.cached_rescaler,
                        false,
                    );
                debug!("DONE rescaling Animations");
            }
            MessageFromServerToClient::Clear => {
                self.state = ClientState::Idle;
            }
            MessageFromServerToClient::DisplayExternalFrame(data) => {
                // data is bmp file data
                let image = match ImageMeta::from_image_bytes(&data) {
                    Err(e) => {
                        error!("Could not decode external frame: {}", e);
                        return;
                    }
                    Ok(image) => image,
                };

                if let Some((w, h)) = self.current_frame_dimensions {
                    // store rescaled to dynamically cache
                    self.state = ClientState::DisplayExternalFrame(image.get_rescaled(w, h));
                } else {
                    self.state = ClientState::DisplayExternalFrame(image);
                }
            }
            MessageFromServerToClient::AdvertisementImages(new_images) => {
                // clear rescale cache or reconnects accumulate data
                for item_currently_in_cache in &self.permanent_images_storage.advertisement_images {
                    self.permanent_images_storage
                        .cached_rescaler
                        .purge_from_cache(item_currently_in_cache);
                }
                self.permanent_images_storage.advertisement_images = Vec::new();

                // write new image data
                for (new_name, new_data) in new_images {
                    match ImageMeta::from_image_bytes(&new_data) {
                        Err(e) => error!(
                            "Could not transform the advertisement image {}. {}",
                            new_name, e
                        ),
                        Ok(img) => {
                            info!("Loaded advertisement image: {}", new_name);
                            self.permanent_images_storage.advertisement_images.push(img);
                        }
                    }
                }
            }
            MessageFromServerToClient::Advertisements => {
                self.state = ClientState::Advertisements;
            }
            MessageFromServerToClient::Timing => {
                // TODO this is only for testing
                self.state = ClientState::TestAnimation(AnimationPlayer::new(
                    &self.permanent_images_storage.fireworks_animation,
                    self.frame_counter,
                    false,
                ));
            }
        }
    }

    pub fn push_new_message(&mut self, msg: MessageFromClientToServer) {
        self.messages_to_send_out_to_server.push(msg);
    }

    pub fn advance_counters(&mut self) {
        self.frame_counter += 1;
    }

    pub fn get_one_message_to_send(&mut self) -> Option<MessageFromClientToServer> {
        self.messages_to_send_out_to_server.pop()
    }
}
