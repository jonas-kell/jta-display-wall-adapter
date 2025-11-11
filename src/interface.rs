use serde::{Deserialize, Serialize};

use crate::{
    args::Args,
    instructions::{
        ClientCommunicationChannelOutbound, IncomingInstruction, InstructionCommunicationChannel,
        InstructionFromTimingClient, InstructionToTimingClient,
    },
    rasterizing::{CachedImageScaler, ImageMeta},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServerToClient {
    DisplayText(String),
    RequestVersion,
    MoveWindow(u32, u32, u32, u32),
    Clear,
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
            comm_channel, // only used to send instructions to the timing client. Rest is done via incoming commands
        }
    }

    pub async fn parse_client_command(&mut self, msg: MessageFromClientToServer) {
        // handle all messages
        match msg {
            MessageFromClientToServer::Version(version) => {
                error!("Client reported to have version: '{}'", version); // TODO compare and error if not compatible

                // report the timing client our fake version
                self.send_message_to_timing_client(InstructionToTimingClient::SendServerInfo)
                    .await;

                // get client to respect window position and size
                debug!(
                    "Requesting window change on client: {} {} {} {}",
                    self.args.dp_pos_x, self.args.dp_pos_y, self.args.dp_width, self.args.dp_height,
                );
                self.send_message_to_client(MessageFromServerToClient::MoveWindow(
                    self.args.dp_pos_x,
                    self.args.dp_pos_y,
                    self.args.dp_width,
                    self.args.dp_height,
                ))
                .await;
            }
            MessageFromClientToServer::CurrentWindow(data) => {
                // Comment to link to other location where this is relevant COMPAREWITHTHIS1
                if self.state == ServerState::PassthroughClient {
                    self.send_message_to_timing_client(InstructionToTimingClient::SendFrame(data))
                        .await;
                }
            }
        }
    }

    pub async fn parse_incoming_command(&mut self, msg: IncomingInstruction) {
        // handle all messages
        match msg {
            IncomingInstruction::FromTimingClient(inst) => match inst {
                InstructionFromTimingClient::ClientInfo => (),
                InstructionFromTimingClient::ServerInfo => (),
                InstructionFromTimingClient::Freetext(text) => {
                    self.send_message_to_client(MessageFromServerToClient::DisplayText(text))
                        .await
                }
                InstructionFromTimingClient::Clear => {
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
                inst => error!("Unhandled instruction from timing client: {}", inst),
            },
            IncomingInstruction::FromCameraProgram(inst) => match inst {
                inst => error!("Unhandled instruction from camera program: {:?}", inst),
            },
        }
    }

    async fn send_message_to_timing_client(&mut self, inst: InstructionToTimingClient) {
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

#[derive(PartialEq, Eq)]
pub enum ClientState {
    Created,
    Idle,
    DisplayText(String),
}

pub struct ImagesStorage {
    pub jta_logo: ImageMeta,
    // todo other images that are loaded dynamically from server
    pub cached_rescaler: CachedImageScaler,
}

pub struct ClientStateMachine {
    pub state: ClientState,
    messages_to_send_out_to_server: Vec<MessageFromClientToServer>,
    pub frame_counter: u64,
    pub window_state_needs_update: Option<(u32, u32, u32, u32)>,
    pub permanent_images_storage: ImagesStorage,
}
impl ClientStateMachine {
    pub fn new() -> Self {
        let jta_image = ImageMeta::from_image_bytes(include_bytes!("../JTA-Logo.png")).unwrap();

        Self {
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
            window_state_needs_update: None,
            permanent_images_storage: ImagesStorage {
                jta_logo: jta_image,
                cached_rescaler: CachedImageScaler::new(),
            },
        }
    }

    pub fn parse_server_command(&mut self, msg: MessageFromServerToClient) {
        match msg {
            MessageFromServerToClient::RequestVersion => {
                info!("Version was requested. Communication established!!");
                if self.state == ClientState::Created {
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
            MessageFromServerToClient::MoveWindow(x, y, w, h) => {
                debug!("Server requested an update of the window position/size");
                self.window_state_needs_update = Some((x, y, w, h));
            }
            MessageFromServerToClient::Clear => {
                self.state = ClientState::Idle;
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
