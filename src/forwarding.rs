use std::time::Duration;

use async_channel::{Receiver, RecvError, SendError, Sender};
use tokio::time::{self, error::Elapsed};

use crate::args::Args;

pub type PacketData = Vec<u8>;

#[derive(Clone)]
pub struct PacketCommunicationChannel {
    args: Args,
    inbound_sender: Sender<PacketData>,
    inbound_receiver: Receiver<PacketData>,
    outbound_sender: Sender<PacketData>,
    outbound_receiver: Receiver<PacketData>,
}
impl PacketCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::unbounded::<PacketData>();
        let (os, or) = async_channel::unbounded::<PacketData>();

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender: os,
            outbound_receiver: or,
        }
    }

    pub async fn inbound_take_in(&self, pack: PacketData) -> Result<(), SendError<PacketData>> {
        self.inbound_sender.send(pack).await
    }

    pub async fn inbound_coming_in(&self) -> Result<Result<PacketData, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.inbound_receiver.recv(),
        )
        .await
    }

    pub async fn outbound_take_in(&self, pack: PacketData) -> Result<(), SendError<PacketData>> {
        self.outbound_sender.send(pack).await
    }

    pub async fn outbound_coming_out(&self) -> Result<Result<PacketData, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver.recv(),
        )
        .await
    }
}
