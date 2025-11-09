use serde::{Deserialize, Serialize};

use crate::instructions::{
    ClientCommunicationChannelOutbound, IncomingInstruction, InstructionCommunicationChannel,
    InstructionToTimingClient,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServerToClient {
    DisplayText(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClientToServer {
    Version(String),
}

#[derive(PartialEq, Eq)]
enum ServerState {
    Idle,
}

pub struct ServerStateMachine {
    state: ServerState,
    comm_channel: InstructionCommunicationChannel,
    comm_channel_client_outbound: ClientCommunicationChannelOutbound,
}
impl ServerStateMachine {
    pub fn new(
        comm_channel: InstructionCommunicationChannel,
        comm_channel_client_outbound: ClientCommunicationChannelOutbound,
    ) -> Self {
        Self {
            state: ServerState::Idle,
            comm_channel_client_outbound, // per design, this can only be used to send
            comm_channel, // only used to send instructions to the timing client. Rest is done via incoming commands
        }
    }

    pub async fn parse_client_command(&mut self, msg: MessageFromClientToServer) {
        info!("Message received from Client: {:?}", msg);
    }

    pub async fn parse_incoming_command(&mut self, msg: IncomingInstruction) {
        trace!(
            "Incoming message from timing client or camera program: {}",
            msg
        );
    }

    async fn send_message_to_timing_client(&mut self, inst: InstructionToTimingClient) {
        match self.comm_channel.send_out_command(inst).await {
            Ok(()) => (),
            Err(e) => error!("Failed to send out instruction: {}", e.to_string()),
        }
    }

    async fn send_message_to_client(&mut self, inst: MessageFromServerToClient) {
        match self.comm_channel_client_outbound.send_away(inst).await {
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

pub struct ClientStateMachine {
    pub state: ClientState,
    messages_to_send_out_to_server: Vec<MessageFromClientToServer>,
    frame_counter: u64,
}
impl ClientStateMachine {
    pub fn new() -> Self {
        Self {
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
        }
    }

    pub fn parse_server_command(&mut self, msg: MessageFromServerToClient) {
        if self.state == ClientState::Created {
            self.state = ClientState::Idle;
            self.push_new_message(MessageFromClientToServer::Version(String::from(
                "TODO: THIS SHOULD BE COMPUTED",
            )));
        }

        info!("Message received from Server: {:?}", msg);
    }

    fn push_new_message(&mut self, msg: MessageFromClientToServer) {
        self.messages_to_send_out_to_server.push(msg);
    }

    pub fn advance_counters(&mut self) {
        self.frame_counter += 1;
    }

    pub fn get_one_message_to_send(&mut self) -> Option<MessageFromClientToServer> {
        self.messages_to_send_out_to_server.pop()
    }
}
