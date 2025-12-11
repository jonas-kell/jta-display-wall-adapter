use crate::args::Args;
use crate::interface::ServerStateMachine;
use crate::server::comm_channel::InstructionCommunicationChannel;
use std::io::{self, Error, ErrorKind};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::Mutex;
// TODO could refactor ServerState to use Arc<RWLock<...>>. With that, the ServerStateMachine would no longer need to live in an Arc<Mutex<...>> and the main server could avoid that alltogether and properly move to solely message based communication

pub async fn intake_commands(
    args: Args,
    server_state: Arc<Mutex<ServerStateMachine>>,
    comm_channel: InstructionCommunicationChannel,
    shutdown_marker: Arc<AtomicBool>,
) -> io::Result<()> {
    let _ = args;

    loop {
        if shutdown_marker.load(Ordering::SeqCst) {
            info!("Shutdown requested, stopping commands intake");
            break;
        }

        match comm_channel.wait_for_incomming_command().await {
            Ok(command_res) => match command_res {
                Ok(comm) => {
                    let mut guard = server_state.lock().await;
                    guard.parse_incoming_command(comm).await;
                }
                Err(e) => return Err(Error::new(ErrorKind::Other, e.to_string())),
            },
            Err(_) => {
                trace!("No incoming command to process in timeout interval");
                continue;
            }
        };
    }

    Ok(())
}
