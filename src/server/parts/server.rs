use crate::args::Args;
use crate::interface::{ServerStateMachine, ServerStateMachineServerStateReader};
use crate::server::comm_channel::{InstructionCommunicationChannel, PacketCommunicationChannel};
use crate::server::parts::client_communicator::client_communicator;
use crate::server::parts::database::create_database_manager;
use crate::server::parts::export::{test_data, write_to_xml_output_file};
use crate::server::parts::intake_commands::intake_commands;
use crate::server::parts::tcp_client_camera_program::tcp_client_camera_program;
use crate::server::parts::tcp_forwarder_display_program::tcp_forwarder_display_program;
use crate::server::parts::tcp_listener_timing_program::tcp_listener_timing_program;
use crate::server::parts::tcp_listener_wind_server::tcp_listener_wind_server;
use crate::webserver::{get_local_ip, webserver, HttpServerStateManager, Server};
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Start server
pub async fn run_server(args: &Args) -> () {
    // TODO remove/move to proper location
    write_to_xml_output_file(args, "test.meetxml", test_data());
    // TODO end

    let passthrough_address_display_program: SocketAddr = format!(
        "{}:{}",
        args.passthrough_address_display_program, args.passthrough_port_display_program
    )
    .parse()
    .expect("Invalid display program passthrough address");

    let own_addr_timing: SocketAddr = format!("0.0.0.0:{}", args.listen_port)
        .parse()
        .expect("Invalid listen address");

    if args.passthrough_to_display_program {
        info!(
            "Talking to {} as display program",
            passthrough_address_display_program
        );
    }
    if args.listen_to_timing_program {
        info!(
            "Listening self to the timing program on {}",
            own_addr_timing
        );
    }

    let (camera_program_timing_address, camera_program_data_address, camera_program_xml_address) =
        if let Some(camera_program_ip) = &args.address_camera_program {
            let camera_program_timing_address: SocketAddr =
                format!("{}:{}", camera_program_ip, args.camera_exchange_timing_port)
                    .parse()
                    .expect("Invalid camera program address for timing");
            let camera_program_data_address: SocketAddr =
                format!("{}:{}", camera_program_ip, args.camera_exchange_data_port)
                    .parse()
                    .expect("Invalid camera program address for data");
            let camera_program_xml_address: SocketAddr =
                format!("{}:{}", camera_program_ip, args.camera_exchange_xml_port)
                    .parse()
                    .expect("Invalid camera program address for xml");
            info!(
                "Talking to the camera program on {}, {} and {}",
                camera_program_timing_address,
                camera_program_data_address,
                camera_program_xml_address
            );

            (
                Some(camera_program_timing_address),
                Some(camera_program_data_address),
                Some(camera_program_xml_address),
            )
        } else {
            info!("Not configured to connect to a camera program");
            (None, None, None)
        };

    let internal_communication_address =
        if let Some(display_client_ip) = &args.address_display_client {
            let internal_communication_address: SocketAddr = format!(
                "{}:{}",
                display_client_ip, args.display_client_communication_port
            )
            .parse()
            .expect("Invalid internal address");

            info!(
                "Talking to {} for internal communication to display client",
                internal_communication_address
            );

            Some(internal_communication_address)
        } else {
            info!("Not configured to connect to a display client");
            None
        };

    let own_addr_webcontrol: SocketAddr = format!("0.0.0.0:{}", args.internal_webcontrol_port)
        .parse()
        .expect("Invalid webcontrol address");

    info!(
        "LAN access from http://{}:{}/",
        get_local_ip(),
        args.internal_webcontrol_port
    );

    // check settings allowed / make sense
    if args.passthrough_to_display_program && !args.listen_to_timing_program {
        error!("Can not passthrough to display program if not listening to timing program");
        return;
    }
    // avoid ports doubling and inform about where to run external display program
    if args.passthrough_to_display_program && args.listen_to_timing_program {
        if args.passthrough_address_display_program == "127.0.0.1"
            && args.passthrough_port_display_program == args.listen_port
        {
            error!("Can not passthrough to display program that should run on the same machine (127.0.0.1)");
            error!("The port {} can only be used by one program at a time (or if no other program is running, this server would connect to itself)", args.listen_port );
            error!("If you wish to passthrough to external display software, you need to run it on a separate machine");
            return;
        }
    }

    let wind_server_address = if let Some(wind_server_ip) = &args.address_wind_server {
        let own_addr_wind_server: SocketAddr =
            format!("{}:{}", wind_server_ip, args.wind_exchange_port)
                .parse()
                .expect("Invalid wind server address");

        Some(own_addr_wind_server)
    } else {
        info!("Not configured to connect to a wind server");
        None
    };

    let comm_channel = InstructionCommunicationChannel::new(&args);
    let comm_channel_packets = PacketCommunicationChannel::new(&args);
    let database_manager = match create_database_manager(args.clone()) {
        Err(e) => {
            error!("Database initialization problem: {}", e);
            return ();
        }
        Ok(man) => man,
    };
    let (server_state, server_state_reader) = ServerStateMachineServerStateReader::build(
        ServerStateMachine::new(&args, comm_channel.clone(), database_manager),
    );
    let shutdown_marker = Arc::new(AtomicBool::new(false));

    let tcp_listener_server_instance = tcp_listener_timing_program(
        args.clone(),
        server_state_reader.clone(),
        comm_channel.clone(),
        comm_channel_packets.clone(),
        Arc::clone(&shutdown_marker),
        own_addr_timing,
    );

    let tcp_forwarder_display_program_instance = tcp_forwarder_display_program(
        args.clone(),
        server_state_reader.clone(),
        comm_channel.clone(),
        comm_channel_packets.clone(),
        Arc::clone(&shutdown_marker),
        passthrough_address_display_program,
    );

    let client_communicator_instance = client_communicator(
        args.clone(),
        server_state_reader.clone(),
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
        internal_communication_address,
    );

    let intake_commands_instance = intake_commands(
        args.clone(),
        server_state,
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
    );

    let tcp_client_camera_program_instance = tcp_client_camera_program(
        args.clone(),
        server_state_reader.clone(),
        comm_channel.clone(),
        Arc::clone(&shutdown_marker),
        camera_program_timing_address,
        camera_program_data_address,
        camera_program_xml_address,
    );

    let web_server_task = webserver(own_addr_webcontrol, comm_channel.clone());
    let (web_server_manager, http_server): (HttpServerStateManager, Server) = match web_server_task
    {
        Ok(res) => res,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let tcp_client_wind_server_instance = tcp_listener_wind_server(
        args.clone(),
        server_state_reader,
        comm_channel.clone(),
        shutdown_marker.clone(),
        wind_server_address,
    );

    // spawn the async runtimes in parallel
    let client_communicator_task = tokio::spawn(client_communicator_instance);
    let intake_commands_task = tokio::spawn(intake_commands_instance);
    let tcp_listener_server_task = tokio::spawn(tcp_listener_server_instance);
    let tcp_forwarder_display_program_task = tokio::spawn(tcp_forwarder_display_program_instance);
    let tcp_client_camera_program_task = tokio::spawn(tcp_client_camera_program_instance);
    let tcp_client_wind_server_task = tokio::spawn(tcp_client_wind_server_instance);
    let webserver_task = tokio::spawn(http_server);
    let shutdown_task = tokio::spawn(async move {
        // listen for ctrl-c
        tokio::signal::ctrl_c().await?;

        shutdown_marker.store(true, Ordering::SeqCst);
        web_server_manager.stop_gracefully().await;

        Ok::<_, Error>(())
    });

    // Wait for all tasks to complete
    match tokio::try_join!(
        client_communicator_task,
        intake_commands_task,
        tcp_listener_server_task,
        tcp_forwarder_display_program_task,
        tcp_client_camera_program_task,
        tcp_client_wind_server_task,
        webserver_task,
        shutdown_task,
    ) {
        Err(_) => error!("Error in at least one listening task"),
        Ok(_) => info!("All listeners closed successfully"),
    };
}
