use crate::{
    args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS},
    instructions::{
        IncomingInstruction, InstructionFromCameraProgram, InstructionFromTimingProgram,
        InstructionToTimingProgram,
    },
    interface::{MessageFromClientToServer, MessageFromServerToClient},
    webserver::{MessageFromWebControl, MessageToWebControl},
    wind::format::{MessageToWindServer, WindMessageBroadcast},
};
use async_channel::{Receiver, RecvError, Sender, TrySendError};
use std::time::Duration;
use tokio::time::{self, error::Elapsed};

#[derive(Clone)]
pub struct InstructionCommunicationChannel {
    args: Args,
    inbound_sender: Sender<IncomingInstruction>,
    inbound_receiver: Receiver<IncomingInstruction>,
    outbound_sender_timing_program: Sender<InstructionToTimingProgram>,
    outbound_receiver_timing_program: Receiver<InstructionToTimingProgram>,
    outbound_sender_client: Sender<MessageFromServerToClient>,
    outbound_receiver_client: Receiver<MessageFromServerToClient>,
    outbound_sender_web_control: Sender<MessageToWebControl>,
    outbound_receiver_web_control: Receiver<MessageToWebControl>,
    outbound_sender_wind_server: Sender<MessageToWindServer>,
    outbound_receiver_wind_server: Receiver<MessageToWindServer>,
}
impl InstructionCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::bounded::<IncomingInstruction>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (os, or) = async_channel::bounded::<InstructionToTimingProgram>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (sc, rc) = async_channel::bounded::<MessageFromServerToClient>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (swe, rwe) = async_channel::bounded::<MessageToWebControl>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        let (swi, rwi) = async_channel::bounded::<MessageToWindServer>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender_timing_program: os,
            outbound_receiver_timing_program: or,
            outbound_sender_client: sc,
            outbound_receiver_client: rc,
            outbound_sender_web_control: swe,
            outbound_receiver_web_control: rwe,
            outbound_sender_wind_server: swi,
            outbound_receiver_wind_server: rwi,
        }

        // TODO all outbound communication that could hit multiple endpoints (definitely websockets) should use a broadcast based 1 -> all communication
    }

    pub fn take_in_command_from_timing_program(
        &self,
        inst: InstructionFromTimingProgram,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromTimingProgram(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub fn take_in_command_from_camera_program(
        &self,
        inst: InstructionFromCameraProgram,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromCameraProgram(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub fn take_in_command_from_web_control(
        &self,
        inst: MessageFromWebControl,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromWebControl(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub fn take_in_command_from_client(
        &self,
        inst: MessageFromClientToServer,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromClient(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }
    pub fn take_in_command_from_wind_server(
        &self,
        inst: WindMessageBroadcast,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromWindServer(inst))
        {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Internal communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Internal communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_incomming_command(
        &self,
    ) -> Result<Result<IncomingInstruction, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.inbound_receiver.recv(),
        )
        .await
    }

    pub fn send_out_command_to_timing_program(
        &self,
        inst: InstructionToTimingProgram,
    ) -> Result<(), String> {
        match self.outbound_sender_timing_program.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Timing program communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Timing program communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_command_to_send_to_timing_program(
        &self,
    ) -> Result<Result<InstructionToTimingProgram, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver_timing_program.recv(),
        )
        .await
    }

    pub fn send_out_command_to_client(
        &self,
        inst: MessageFromServerToClient,
    ) -> Result<(), String> {
        match self.outbound_sender_client.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => Err(format!("Client communication channel closed...")),
            Err(TrySendError::Full(_)) => {
                trace!("Client communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_command_to_send_to_client(
        &self,
    ) -> Result<Result<MessageFromServerToClient, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver_client.recv(),
        )
        .await
    }

    pub fn send_out_command_to_web_control(&self, inst: MessageToWebControl) -> Result<(), String> {
        match self.outbound_sender_web_control.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Web control communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Web control communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_command_to_send_to_web_control(
        &self,
    ) -> Result<Result<MessageToWebControl, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver_web_control.recv(),
        )
        .await
    }

    pub fn send_out_command_to_wind_server(&self, inst: MessageToWindServer) -> Result<(), String> {
        match self.outbound_sender_wind_server.try_send(inst) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(format!("Wind server communication channel closed..."))
            }
            Err(TrySendError::Full(_)) => {
                trace!("Wind server communication channel is full. Seems like there is no source to consume");
                Ok(())
            }
        }
    }

    pub async fn wait_for_command_to_send_to_wind_server(
        &self,
    ) -> Result<Result<MessageToWindServer, RecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.outbound_receiver_wind_server.recv(),
        )
        .await
    }
}
