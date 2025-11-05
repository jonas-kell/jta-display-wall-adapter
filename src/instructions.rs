use std::time::Duration;

use async_channel::{Receiver, RecvError, SendError, Sender};
use tokio::time::{self, error::Elapsed};

use crate::args::Args;

#[derive(Debug)]
pub enum InstructionFromTimingClient {
    ClientInfo,
    Freetext(String),
    Advertisements,
    Clear,
    StartList,
}

#[derive(Debug)]
pub enum InstructionToTimingClient {
    SendBeforeFrameSetupInstruction,
    SendFrame, // TODO needs to store the frame data
}

#[derive(Clone)]
pub struct InstructionCommunicationChannel {
    args: Args,
    inbound_sender: Sender<InstructionFromTimingClient>,
    inbound_receiver: Receiver<InstructionFromTimingClient>,
    outbound_sender: Sender<InstructionToTimingClient>,
    outbound_receiver: Receiver<InstructionToTimingClient>,
}
impl InstructionCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::unbounded::<InstructionFromTimingClient>();
        let (os, or) = async_channel::unbounded::<InstructionToTimingClient>();

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender: os,
            outbound_receiver: or,
        }
    }

    pub async fn take_in_command(
        &self,
        inst: InstructionFromTimingClient,
    ) -> Result<(), SendError<InstructionFromTimingClient>> {
        self.inbound_sender.send(inst).await
    }

    pub async fn wait_for_incomming_command(
        &self,
    ) -> Result<Result<InstructionFromTimingClient, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.inbound_receiver.recv(),
        )
        .await
    }

    pub async fn send_out_command(
        &self,
        inst: InstructionToTimingClient,
    ) -> Result<(), SendError<InstructionToTimingClient>> {
        self.outbound_sender.send(inst).await
    }

    pub async fn wait_for_command_to_send(
        &self,
    ) -> Result<Result<InstructionToTimingClient, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver.recv(),
        )
        .await
    }
}
