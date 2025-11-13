use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    args::Args,
    client::images_tools::{CachedImageScaler, ImageMeta},
    file::read_image_files,
    instructions::{
        ClientCommunicationChannelOutbound, IncomingInstruction, InstructionCommunicationChannel,
        InstructionFromTimingProgram, InstructionToTimingProgram,
    },
};

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

pub struct ServerStateMachine {
    args: Args,
    pub state: ServerState,
    comm_channel: InstructionCommunicationChannel,
    comm_channel_client_outbound: ClientCommunicationChannelOutbound,
}
impl ServerStateMachine {
    pub fn new(
        args: &Args,
        comm_channel: InstructionCommunicationChannel,
        comm_channel_client_outbound: ClientCommunicationChannelOutbound,
    ) -> Self {
        Self {
            args: args.clone(),
            state: ServerState::PassthroughClient,
            comm_channel_client_outbound, // per design, this can only be used to send
            comm_channel, // only used to send instructions to the timing program. Rest is done via incoming commands
        }
    }

    pub async fn parse_client_command(&mut self, msg: MessageFromClientToServer) {
        // handle all messages
        match msg {
            MessageFromClientToServer::Version(version) => {
                error!("Client reported to have version: '{}'", version); // TODO compare and error if not compatible

                // report the timing program our fake version
                self.send_message_to_timing_program(InstructionToTimingProgram::SendServerInfo)
                    .await;

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
                ))
                .await;

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
                ))
                .await;
            }
            MessageFromClientToServer::CurrentWindow(data) => {
                if self.state == ServerState::PassthroughClient {
                    self.send_message_to_timing_program(InstructionToTimingProgram::SendFrame(
                        data,
                    ))
                    .await;
                }
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
                    self.send_message_to_client(MessageFromServerToClient::DisplayText(text))
                        .await
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
                        self.send_message_to_client(MessageFromServerToClient::Clear)
                            .await;
                    } else {
                        self.state = ServerState::PassthroughClient;
                    }
                }
                InstructionFromTimingProgram::SendFrame(data) => {
                    trace!("Got command to send a frame inbound (should be from external display program to send back and possibly proxy to our client)");
                    if self.state == ServerState::PassthroughDisplayProgram {
                        self.send_message_to_client(
                            MessageFromServerToClient::DisplayExternalFrame(data),
                        )
                        .await;
                    }
                }
                InstructionFromTimingProgram::Advertisements => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Advertisements)
                            .await;
                    }
                }
                inst => error!("Unhandled instruction from timing program: {}", inst),
            },
            IncomingInstruction::FromCameraProgram(inst) => match inst {
                inst => error!("Unhandled instruction from camera program: {:?}", inst),
            },
        }
    }

    async fn send_message_to_timing_program(&mut self, inst: InstructionToTimingProgram) {
        match self.comm_channel.send_out_command(inst) {
            Ok(()) => (),
            Err(e) => error!("Failed to send out instruction: {}", e.to_string()),
        }
    }

    pub async fn make_server_request_client_version(&mut self) {
        self.send_message_to_client(MessageFromServerToClient::RequestVersion)
            .await
    }

    async fn send_message_to_client(&mut self, inst: MessageFromServerToClient) {
        match self.comm_channel_client_outbound.send_away(inst) {
            Ok(()) => (),
            Err(e) => error!(
                "Failed to send out instruction to client: {}",
                e.to_string()
            ),
        }
    }
}

pub enum ClientState {
    Created,
    Idle,
    DisplayText(String),
    DisplayExternalFrame(ImageMeta),
    Advertisements,
}

pub struct ImagesStorage {
    pub jta_logo: ImageMeta,
    pub advertisement_images: Vec<ImageMeta>,
    pub cached_rescaler: CachedImageScaler,
}

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
        let jta_image = ImageMeta::from_image_bytes(include_bytes!("../JTA-Logo.png")).unwrap();

        Self {
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
            window_state_needs_update: None,
            permanent_images_storage: ImagesStorage {
                jta_logo: jta_image,
                cached_rescaler: CachedImageScaler::new(),
                advertisement_images: Vec::new(),
            },
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
