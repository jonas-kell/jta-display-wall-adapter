use crate::{
    args::{Args, MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS},
    idcapture::format::{IDCaptureMessage, MessageToIdcaptureServer},
    instructions::{
        IncomingInstruction, InstructionFromCameraProgram, InstructionFromExternalDisplayProgram,
        InstructionFromTimingProgram, InstructionToTimingProgram,
    },
    interface::{MessageFromClientToServer, MessageFromServerToClient},
    server::BibMessage,
    webserver::{MessageFromWebControl, MessageToWebControl},
    wind::format::{MessageToWindServer, WindMessageBroadcast},
};
use async_broadcast::{
    InactiveReceiver, Receiver as BroadcastReceiverLibrary, RecvError as BroadcastRecvError,
    Sender as BroadcastSender, TrySendError as BroadcastTrySendError,
};
use async_channel::{Receiver, RecvError, Sender, TrySendError};
use std::time::Duration;
use tokio::time::{self, error::Elapsed};

pub enum ConnectionCheck {
    CameraProgramTimingPort,
    CameraProgramDataPort,
    CameraProgramXMLPort,
    ExternalDisplayProgramPassthrough,
}

#[derive(Clone)]
pub struct InstructionCommunicationChannel {
    args: Args,
    inbound_sender: Sender<IncomingInstruction>,
    inbound_receiver: Receiver<IncomingInstruction>,
    outbound_sender_timing_program: BroadcastSender<InstructionToTimingProgram>,
    outbound_receiver_timing_program: BroadcastReceiverStorage<InstructionToTimingProgram>,
    outbound_sender_client: BroadcastSender<MessageFromServerToClient>,
    outbound_receiver_client: BroadcastReceiverStorage<MessageFromServerToClient>,
    outbound_sender_web_control: BroadcastSender<MessageToWebControl>,
    outbound_receiver_web_control: BroadcastReceiverStorage<MessageToWebControl>,
    outbound_sender_wind_server: BroadcastSender<MessageToWindServer>,
    outbound_receiver_wind_server: BroadcastReceiverStorage<MessageToWindServer>,
    outbound_sender_idcapture_server: BroadcastSender<MessageToIdcaptureServer>,
    outbound_receiver_idcapture_server: BroadcastReceiverStorage<MessageToIdcaptureServer>,
    connection_check_sender_camera_program_timing_port: BroadcastSender<bool>,
    connection_check_receiver_camera_program_timing_port: BroadcastReceiverStorage<bool>,
    connection_check_sender_camera_program_data_port: BroadcastSender<bool>,
    connection_check_receiver_camera_program_data_port: BroadcastReceiverStorage<bool>,
    connection_check_sender_camera_program_xml_port: BroadcastSender<bool>,
    connection_check_receiver_camera_program_xml_port: BroadcastReceiverStorage<bool>,
    connection_check_sender_external_display_passthrough: BroadcastSender<bool>,
    connection_check_receiver_external_display_passthrough: BroadcastReceiverStorage<bool>,
}
impl InstructionCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (is, ir) = async_channel::bounded::<IncomingInstruction>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );

        // outbound channels could have multiple targets (broadcast), but in any case must support, that they can discard messages if there is no active receiver available
        let (mut os, or) = async_broadcast::broadcast::<InstructionToTimingProgram>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        os.set_overflow(true);
        let (mut sc, rc) = async_broadcast::broadcast::<MessageFromServerToClient>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        sc.set_overflow(true);
        let (mut swe, rwe) = async_broadcast::broadcast::<MessageToWebControl>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        swe.set_overflow(true);
        let (mut swi, rwi) = async_broadcast::broadcast::<MessageToWindServer>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        swi.set_overflow(true);
        let (mut sid, rid) = async_broadcast::broadcast::<MessageToIdcaptureServer>(
            MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS,
        );
        sid.set_overflow(true);
        // channels that only check for connection
        let (mut scptp, rcptp) =
            async_broadcast::broadcast::<bool>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        scptp.set_overflow(true);
        let (mut scpdp, rcpdp) =
            async_broadcast::broadcast::<bool>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        scpdp.set_overflow(true);
        let (mut scpxp, rcpxp) =
            async_broadcast::broadcast::<bool>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        scpxp.set_overflow(true);
        let (mut sdpt, rdpt) =
            async_broadcast::broadcast::<bool>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        sdpt.set_overflow(true);

        Self {
            args: args.clone(),
            inbound_sender: is,
            inbound_receiver: ir,
            outbound_sender_timing_program: os,
            outbound_receiver_timing_program: BroadcastReceiverStorage::new(or, args),
            outbound_sender_client: sc,
            outbound_receiver_client: BroadcastReceiverStorage::new(rc, args),
            outbound_sender_web_control: swe,
            outbound_receiver_web_control: BroadcastReceiverStorage::new(rwe, args),
            outbound_sender_wind_server: swi,
            outbound_receiver_wind_server: BroadcastReceiverStorage::new(rwi, args),
            outbound_sender_idcapture_server: sid,
            outbound_receiver_idcapture_server: BroadcastReceiverStorage::new(rid, args),
            connection_check_sender_camera_program_timing_port: scptp,
            connection_check_receiver_camera_program_timing_port: BroadcastReceiverStorage::new(
                rcptp, args,
            ),
            connection_check_sender_camera_program_data_port: scpdp,
            connection_check_receiver_camera_program_data_port: BroadcastReceiverStorage::new(
                rcpdp, args,
            ),
            connection_check_sender_camera_program_xml_port: scpxp,
            connection_check_receiver_camera_program_xml_port: BroadcastReceiverStorage::new(
                rcpxp, args,
            ),
            connection_check_sender_external_display_passthrough: sdpt,
            connection_check_receiver_external_display_passthrough: BroadcastReceiverStorage::new(
                rdpt, args,
            ),
        }
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

    pub fn take_in_command_from_external_display_program(
        &self,
        inst: InstructionFromExternalDisplayProgram,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromExternalDisplayProgram(inst))
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

    pub fn take_in_command_from_idcapture_server(
        &self,
        inst: IDCaptureMessage,
    ) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromIdcaptureServer(inst))
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

    pub fn take_in_command_from_bib_server(&self, inst: BibMessage) -> Result<(), String> {
        match self
            .inbound_sender
            .try_send(IncomingInstruction::FromBibServer(inst))
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
        match self.outbound_sender_timing_program.try_broadcast(inst) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in internal comm channel (to timing program)");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!(
                    "Outbound internal channel not open, no active receivers (to timing program)",
                );
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Timing Program receivers are there, but outbound internal channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Timing program communication channel went away unexpectedly"
            )),
        }
    }

    pub fn timing_program_there_to_receive(&self) -> bool {
        self.outbound_sender_timing_program.receiver_count() > 0
    }

    pub fn timing_program_receiver(&self) -> BroadcastReceiver<InstructionToTimingProgram> {
        self.outbound_receiver_timing_program.get_active_receiver()
    }

    pub fn send_out_command_to_client(
        &self,
        inst: MessageFromServerToClient,
    ) -> Result<(), String> {
        match self.outbound_sender_client.try_broadcast(inst) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in internal comm channel (to client)");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!("Outbound internal channel not open, no active receivers (to client)",);
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Client receivers are there, but outbound internal channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Client communication channel went away unexpectedly"
            )),
        }
    }

    pub fn client_receiver(&self) -> BroadcastReceiver<MessageFromServerToClient> {
        self.outbound_receiver_client.get_active_receiver()
    }

    pub fn send_out_command_to_web_control(&self, inst: MessageToWebControl) -> Result<(), String> {
        match self.outbound_sender_web_control.try_broadcast(inst) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in internal comm channel (to web control)");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!("Outbound internal channel not open, no active receivers (to web control)",);
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Web control receivers are there, but outbound internal channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Web control communication channel went away unexpectedly"
            )),
        }
    }

    pub fn web_control_there_to_receive(&self) -> bool {
        self.outbound_sender_web_control.receiver_count() > 0
    }

    pub fn web_control_receiver(&self) -> BroadcastReceiver<MessageToWebControl> {
        self.outbound_receiver_web_control.get_active_receiver()
    }

    pub fn send_out_command_to_wind_server(&self, inst: MessageToWindServer) -> Result<(), String> {
        match self.outbound_sender_wind_server.try_broadcast(inst) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in internal comm channel (to wind server)");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!("Outbound internal channel not open, no active receivers (to wind server)",);
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Wind server receivers are there, but outbound internal channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Wind server communication channel went away unexpectedly"
            )),
        }
    }

    #[allow(dead_code)]
    pub fn send_out_command_to_idcapture_server(
        &self,
        inst: MessageToIdcaptureServer,
    ) -> Result<(), String> {
        match self.outbound_sender_idcapture_server.try_broadcast(inst) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in internal comm channel (to idcapture server)");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!(
                    "Outbound internal channel not open, no active receivers (to idcapture server)",
                );
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Wind server receivers are there, but outbound internal channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Wind server communication channel went away unexpectedly"
            )),
        }
    }

    pub fn wind_server_receiver(&self) -> BroadcastReceiver<MessageToWindServer> {
        self.outbound_receiver_wind_server.get_active_receiver()
    }

    pub fn wind_server_there_to_receive(&self) -> bool {
        self.outbound_sender_wind_server.receiver_count() > 0
    }

    pub fn idcapture_server_receiver(&self) -> BroadcastReceiver<MessageToIdcaptureServer> {
        self.outbound_receiver_idcapture_server
            .get_active_receiver()
    }

    pub fn idcapture_server_there_to_receive(&self) -> bool {
        self.outbound_sender_idcapture_server.receiver_count() > 0
    }

    pub fn connection_check(&self, tpe: ConnectionCheck) -> bool {
        match tpe {
            ConnectionCheck::CameraProgramTimingPort => {
                self.connection_check_sender_camera_program_timing_port
                    .receiver_count()
                    > 0
            }
            ConnectionCheck::CameraProgramDataPort => {
                self.connection_check_sender_camera_program_data_port
                    .receiver_count()
                    > 0
            }
            ConnectionCheck::CameraProgramXMLPort => {
                self.connection_check_sender_camera_program_xml_port
                    .receiver_count()
                    > 0
            }
            ConnectionCheck::ExternalDisplayProgramPassthrough => {
                self.connection_check_sender_external_display_passthrough
                    .receiver_count()
                    > 0
            }
        }
    }

    pub fn get_connection_check_marker(&self, tpe: ConnectionCheck) -> BroadcastReceiver<bool> {
        match tpe {
            ConnectionCheck::CameraProgramTimingPort => self
                .connection_check_receiver_camera_program_timing_port
                .get_active_receiver(),
            ConnectionCheck::CameraProgramDataPort => self
                .connection_check_receiver_camera_program_data_port
                .get_active_receiver(),
            ConnectionCheck::CameraProgramXMLPort => self
                .connection_check_receiver_camera_program_xml_port
                .get_active_receiver(),
            ConnectionCheck::ExternalDisplayProgramPassthrough => self
                .connection_check_receiver_external_display_passthrough
                .get_active_receiver(),
        }
    }
}

pub type PacketData = Vec<u8>;

#[derive(Clone)]
pub struct PacketCommunicationChannel {
    inbound_sender: BroadcastSender<PacketData>,
    inbound_receiver_ref: BroadcastReceiverStorage<PacketData>,
    outbound_sender: BroadcastSender<PacketData>,
    outbound_receiver_ref: BroadcastReceiverStorage<PacketData>,
}
impl PacketCommunicationChannel {
    pub fn new(args: &Args) -> Self {
        let (mut is, ir) =
            async_broadcast::broadcast::<PacketData>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        is.set_overflow(true);
        let (mut os, or) =
            async_broadcast::broadcast::<PacketData>(MAX_NUMBER_OF_MESSAGES_IN_INTERNAL_BUFFERS);
        os.set_overflow(true);

        Self {
            inbound_sender: is,
            inbound_receiver_ref: BroadcastReceiverStorage::new(ir, args),
            outbound_sender: os,
            outbound_receiver_ref: BroadcastReceiverStorage::new(or, args),
        }
    }

    pub fn inbound_take_in(&self, pack: PacketData) -> Result<(), String> {
        match self.inbound_sender.try_broadcast(pack) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in inbound packet forwarding channel");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!("Inbound packet forwarding channel not open, no active receivers",);
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Packet forwarding receivers are there, but inbound channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Inbound packet forwarding channel went away unexpectedly"
            )),
        }
    }

    pub fn inbound_receiver(&self) -> BroadcastReceiver<PacketData> {
        self.inbound_receiver_ref.get_active_receiver()
    }

    pub fn outbound_take_in(&self, pack: PacketData) -> Result<(), String> {
        match self.outbound_sender.try_broadcast(pack) {
            Ok(Some(_)) => {
                trace!("Thrown away old message in outbound packet forwarding channel");
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(BroadcastTrySendError::Inactive(_)) => {
                warn!("Outbound packet forwarding channel not open, no active receivers",);
                Ok(())
            }
            Err(BroadcastTrySendError::Full(_)) => {
                error!("Packet forwarding receivers are there, but outbound channel full. This should not happen!");
                Ok(())
            }
            Err(BroadcastTrySendError::Closed(_)) => Err(format!(
                "Outbound packet forwarding channel went away unexpectedly"
            )),
        }
    }

    pub fn outbound_receiver(&self) -> BroadcastReceiver<PacketData> {
        self.outbound_receiver_ref.get_active_receiver()
    }
}

#[derive(Clone)]
struct BroadcastReceiverStorage<T>
where
    T: Clone,
{
    args: Args,
    rec: InactiveReceiver<T>,
}

impl<T> BroadcastReceiverStorage<T>
where
    T: Clone,
{
    fn new(receiver: BroadcastReceiverLibrary<T>, args: &Args) -> Self {
        let deact = receiver.deactivate();

        Self {
            rec: deact,
            args: args.clone(),
        }
    }

    fn get_active_receiver(&self) -> BroadcastReceiver<T> {
        BroadcastReceiver {
            args: self.args.clone(),
            rec: self.rec.activate_cloned(),
        }
    }
}

pub struct BroadcastReceiver<T>
where
    T: Clone,
{
    args: Args,
    rec: BroadcastReceiverLibrary<T>,
}

impl<T> BroadcastReceiver<T>
where
    T: Clone,
{
    pub async fn wait_for_some_data(&mut self) -> Result<Result<T, BroadcastRecvError>, Elapsed> {
        time::timeout(
            Duration::from_millis(self.args.wait_ms_before_testing_for_shutdown),
            self.rec.recv(),
        )
        .await
    }

    pub fn conn_check_usage_end_function(self) -> () {
        let _ = self.rec.deactivate();
    }
}
