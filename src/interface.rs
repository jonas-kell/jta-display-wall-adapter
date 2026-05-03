use crate::client::frametime::{FrametimeReport, FrametimeTracker};
use crate::database::{
    create_heat_assignment, delete_athlete, delete_bib_equivalence, delete_evaluation,
    delete_heat_assignment, delete_pdf_setting, get_all_athletes_meta_data,
    get_database_static_state, get_main_heat, init_database_static_state,
    populate_display_from_bib, ApplicationMode, DatabaseStaticState,
};
use crate::idcapture::format::IDCaptureMessage;
use crate::instructions::InstructionFromExternalDisplayProgram::{Frame, ServerInfo};
use crate::open_webcontrol;
use crate::productkey::{dev_mode, product_key_valid};
use crate::productkey::{today, ProductKey};
use crate::server::audio_types::{AudioPlayer, Sound};
use crate::server::bib_detection::{
    generate_bib_data, BibDataPoint, CompetitorEvaluatedBibServer, DisplayEntry,
    MessageToBibServer, RaceHasStartedBibServer, SeekForTimeBibServer,
};
use crate::server::camera_program_types::{
    CompetitorEvaluated, HeatFalseStart, HeatFinish, HeatIntermediate, HeatResult, HeatStart,
    HeatWind,
};
use crate::server::comm_channel::{ConnectionCheck, InstructionCommunicationChannel};
use crate::server::export_functions::{
    fake_main_heat_start_list, generate_meet_data, write_to_xml_output_file,
};
use crate::times::RaceTime;
use crate::webserver::{ConnectionState, PDFConfigurationSetting};
use crate::{
    args::Args,
    client::{ClockState, TimingSettings, TimingStateMachine, TimingUpdate},
    database::{
        get_heat_data, get_log_limited, get_wind_readings, purge_heat_data, DatabaseManager,
        DatabaseSerializable,
    },
    file::read_image_files,
    instructions::{
        IncomingInstruction, InstructionFromCameraProgram, InstructionFromTimingProgram,
        InstructionToTimingProgram,
    },
    server::camera_program_types::HeatStartList,
    times::DayTime,
    webserver::{DisplayClientState, MessageFromWebControl, MessageToWebControl},
    wind::format::{
        MessageToWindServer::SetTime,
        WindMessageBroadcast::{Measured, Started},
    },
};
use async_channel::Sender;
use clap::crate_version;
use images_core::images::{IconsStorage, ImageMeta, ImagesStorage};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use std::{path::Path, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerImposedSettings {
    pub position: (u32, u32, u32, u32),
    pub table_duration_nr_ms: u32,
    pub slideshow_duration_nr_ms: u32,
    pub slideshow_transition_duration_nr_ms: u32,
    pub scroll_text_speed: u32,
    pub scroll_text_deadzones_nr_ms: u32,
}
impl ServerImposedSettings {
    fn new(args: &Args) -> Self {
        Self {
            position: (args.dp_pos_x, args.dp_pos_y, args.dp_width, args.dp_height),
            table_duration_nr_ms: args.table_duration_nr_ms,
            slideshow_duration_nr_ms: args.slideshow_duration_nr_ms,
            slideshow_transition_duration_nr_ms: args.slideshow_transition_duration_nr_ms,
            scroll_text_speed: args.scroll_text_speed,
            scroll_text_deadzones_nr_ms: args.scroll_text_deadzones_nr_ms,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientInternalMessageFromServerToClient {
    EmitTimingSettingsUpdate(TimingSettings),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromServerToClient {
    DisplayText(String),
    RequestVersion,
    ProductKey(String),
    ServerImposedSettings(ServerImposedSettings),
    Clear,
    DisplayExternalFrame(Vec<u8>),
    AdvertisementImages(Vec<(String, Vec<u8>)>),
    Advertisements,
    Timing,
    TimingStateUpdate(TimingUpdate),
    TimingSettingsUpdate(TimingSettings),
    RequestTimingSettings,
    Clock(DayTime),
    ClientInternal(ClientInternalMessageFromServerToClient),
    PushDisplayEntry(DisplayEntry),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerInternalMessageFromClientToServer {
    SetMainDisplayState(bool),
    MakeVersionRequestToAllClients,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClientToServer {
    ServerInternal(ServerInternalMessageFromClientToServer),
    Version(String),
    CurrentWindow(Vec<u8>),
    TimingSettingsState(TimingSettings),
    FrametimeReport(FrametimeReport),
}

#[derive(PartialEq, Eq, Clone)]
pub enum ServerState {
    PassthroughClient,
    PassthroughDisplayProgram,
}

macro_rules! store_to_database_log_conditionally {
    ($value:expr, $self_val:expr, $log:expr, $ignore_date:expr) => {
        if let Some(dbss) = &$self_val.static_state {
            if $self_val.args.can_store_to_database_on_off_day || dbss.date == today() || $ignore_date {
                match $value.store_to_database(&$self_val.database_manager) {
                    Ok(()) => {
                        trace!("Success, we stored an instruction into the database");
                    }
                    Err(e) => {
                        error!("Database storage error: {}", e);
                    }
                }
                if $log {
                    $self_val.send_out_latest_n_logs_to_webclient(1);
                }
            } else {
                warn!("Storage can not be usead on a day that is not the database day. You might enable 'can_store_to_database_on_off_day' to change this");
            }
        } else {
            error!("Database static state is not initialized. Can not store to database");
        }
    };
}

macro_rules! store_to_database {
    ($value:expr, $self_val:expr) => {
        store_to_database_log_conditionally!($value, $self_val, true, false);
    };
}

macro_rules! store_to_database_ignore_date {
    ($value:expr, $self_val:expr) => {
        store_to_database_log_conditionally!($value, $self_val, true, true);
    };
}

#[derive(Clone)]
pub struct ServerStateMachineServerStateReader {
    reference: Arc<Mutex<ServerStateMachine>>,
}
impl ServerStateMachineServerStateReader {
    pub fn build(ssm: ServerStateMachine) -> (Arc<Mutex<ServerStateMachine>>, Self) {
        let arc = Arc::new(Mutex::new(ssm));
        (arc.clone(), Self { reference: arc })
    }

    pub async fn get_server_state(&self) -> ServerState {
        let guard = self.reference.lock().await;
        guard.state.clone()
    }

    pub async fn external_connection_is_allowed(&self) -> bool {
        let guard = self.reference.lock().await;
        guard.allows_external_connections()
    }
}

pub struct ServerStateMachine {
    args: Args,
    pub state: ServerState,
    comm_channel: InstructionCommunicationChannel,
    display_connected: bool,
    database_manager: DatabaseManager,
    sound_engine: Option<AudioPlayer>,
    timing_settings_template: TimingSettings,
    static_state: Option<DatabaseStaticState>,
    database_version_mismatch: Option<(String, String)>,
    bib_heat_selection: Option<Uuid>,
    heat_start_time_instant: Option<(DayTime, Instant)>,
}
impl ServerStateMachine {
    pub fn new(
        args: &Args,
        comm_channel: InstructionCommunicationChannel,
        database_manager: DatabaseManager,
    ) -> Self {
        let sound_engine = match AudioPlayer::new() {
            Err(e) => {
                error!("Failed to initialize Audio Playback: {}", e);
                None
            }
            Ok(e) => Some(e),
        };

        let mut database_version_mismatch = None;

        let static_state = match get_database_static_state(&database_manager) {
            Err((version_opt, e)) => {
                error!("Could not read static state from Database (might be un-initialized), because: {}", e);
                warn!("Initialize over the web interface, otherwise nothing can work...");
                if !comm_channel.web_control_there_to_receive() {
                    info!("Opening webcontrol in browser");
                    open_webcontrol(args);
                }

                if let Some(version_tupel) = version_opt {
                    database_version_mismatch = Some(version_tupel);
                }

                None
            }
            Ok(s) => Some(s),
        };

        Self {
            args: args.clone(),
            state: ServerState::PassthroughClient,
            comm_channel, // only used to send instructions outwards. Rest is done via incoming commands (there is a handler that continously takes them out of the channel and forwards them into us)
            display_connected: false,
            database_manager,
            sound_engine,
            timing_settings_template: TimingSettings::new(args),
            static_state,
            database_version_mismatch,
            bib_heat_selection: None,
            heat_start_time_instant: None,
        }
    }

    fn play_sound(&self, sound: Sound) {
        match &self.sound_engine {
            None => error!("Can not play sounds, as sound engine not initialized"),
            Some(se) => se.play_audio_background(sound),
        }
    }

    pub async fn parse_incoming_command(&mut self, msg: IncomingInstruction) {
        // check license
        let product_key: ProductKey = match product_key_valid(self.args.product_key.as_ref()) {
            Ok(product_key) => product_key,
            Err(e) => {
                error!("There is no valid product key for this software: {}", e);

                return;
            }
        };

        // check database
        let dbss = if let Some(dbss) = &self.static_state {
            dbss
        } else {
            if let Some(version_mismatch) = &self.database_version_mismatch {
                self.send_message_to_web_control(MessageToWebControl::VersionMismatch(
                    version_mismatch.clone(),
                ));
            }

            self.send_message_to_web_control(
                MessageToWebControl::StaticConfigurationNotInitialized,
            );

            // static state not initialized
            let updated_successfully = match msg {
                IncomingInstruction::FromWebControl(w) => match w {
                    MessageFromWebControl::RequestPassword => {
                        // this is still required, otherwise we cannot even init db
                        self.send_message_to_web_control(MessageToWebControl::Password(
                            self.args.webcontrol_password.clone(),
                        ));
                        None
                    }
                    MessageFromWebControl::InitStaticDatabaseState(mut init) => {
                        // overwrite setting with the local value
                        init.program_licensed_for = product_key.company_name.clone();

                        match init_database_static_state(init, &self.database_manager) {
                            Ok(dbss) => {
                                self.static_state = Some(dbss.clone());
                                info!("Database static state was updated (if program misbehaves, it might be neede to restart it once)");
                                Some(dbss)
                            }
                            Err(e) => {
                                error!(
                                    "Could not update the database static state: {}",
                                    e.to_string()
                                );
                                None
                            }
                        }
                    }
                    _ => None,
                },
                _ => None,
            };

            if let Some(dbss) = updated_successfully {
                self.send_message_to_web_control(MessageToWebControl::DatabaseStaticState(dbss));
            } else {
                warn!("Database is not initialized!! Initialize over the web interface, otherwise nothing can work...");
                if !self.comm_channel.web_control_there_to_receive() {
                    open_webcontrol(&self.args);
                }
            }

            // do not allow processing, unless static state is set!!
            return;
        };

        // handle all messages
        match msg {
            IncomingInstruction::FromBibServer(bm) => {
                // TODO store to database and use automated results
                // also filter for automated and manual events in the ui then
                match populate_display_from_bib(bm.bib, &self.database_manager) {
                    Ok(data) => match data {
                        Some(de) => self.send_message_to_client(
                            MessageFromServerToClient::PushDisplayEntry(de),
                        ),
                        None => debug!(
                            "Received a bib signal, but could not match to athlete: {}",
                            bm.bib
                        ),
                    },
                    Err(e) => {
                        error!(
                            "Could not resolve athlete data for bib {}, because: {}",
                            bm.bib, e
                        );
                    }
                }
            }
            IncomingInstruction::FromClient(inst) => match inst {
                MessageFromClientToServer::ServerInternal(internal) => match internal {
                    ServerInternalMessageFromClientToServer::MakeVersionRequestToAllClients => {
                        // this possibly must be changed for multiple clients (possible // TODO )
                        match self.args.product_key.as_ref() {
                            Some(key) => {
                                self.send_message_to_client(MessageFromServerToClient::ProductKey(
                                    key.clone(),
                                ));
                            }
                            None => {}
                        }
                        self.send_message_to_client(MessageFromServerToClient::RequestVersion);
                    }
                    ServerInternalMessageFromClientToServer::SetMainDisplayState(new_val) => {
                        self.set_main_display_state(new_val);
                    }
                },
                MessageFromClientToServer::Version(version) => {
                    info!("Client reported to have version: '{}'", version);
                    let our_version = String::from(crate_version!());
                    if version == our_version {
                        info!("That is a version match. Communication established!");
                    } else {
                        error!("CAUTION: the client version is NOT the same version as we expected. This might cause the program to misbehave or outright not work!");
                        error!("Client: {}; Server: {}", version, our_version);
                    }

                    if self.args.listen_to_timing_program {
                        // report the timing program our fake version
                        self.send_message_to_timing_program(
                            InstructionToTimingProgram::SendServerInfo,
                        );
                    }

                    // init-impose the server settings from the server
                    let server_imposed_settings = ServerImposedSettings::new(&self.args);
                    debug!(
                        "Requesting window change on client: {} {} {} {}",
                        server_imposed_settings.position.0,
                        server_imposed_settings.position.1,
                        server_imposed_settings.position.2,
                        server_imposed_settings.position.3,
                    );
                    self.send_message_to_client(MessageFromServerToClient::ServerImposedSettings(
                        server_imposed_settings,
                    ));
                    // init-impose the timing settings from the server
                    self.send_message_to_client(MessageFromServerToClient::TimingSettingsUpdate(
                        self.timing_settings_template.clone(),
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
                        // force our frame onto the timing program
                        if self.args.listen_to_timing_program
                            && self.comm_channel.timing_program_there_to_receive()
                        {
                            self.send_message_to_timing_program(
                                InstructionToTimingProgram::SendFrame(data.clone()),
                            );
                        }

                        if self.comm_channel.web_control_there_to_receive() {
                            self.send_message_to_web_control(
                                MessageToWebControl::CurrentDisplayFrame(data),
                            );
                        }
                    }
                    // ping the state to the web control
                    if self.comm_channel.web_control_there_to_receive() {
                        self.send_message_to_web_control(MessageToWebControl::DisplayClientState(
                            self.get_display_client_state(),
                        ));
                    }
                }
                MessageFromClientToServer::TimingSettingsState(set) => {
                    self.timing_settings_template = set.clone();
                    self.send_message_to_web_control(MessageToWebControl::TimingSettingsState(set));
                }
                MessageFromClientToServer::FrametimeReport(ftr) => {
                    if self.comm_channel.web_control_there_to_receive() {
                        self.send_message_to_web_control(MessageToWebControl::FrametimeReport(ftr));
                    }
                }
            },
            IncomingInstruction::FromTimingProgram(inst) => match inst {
                InstructionFromTimingProgram::ClientInfo => (),
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
                InstructionFromTimingProgram::Advertisements => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Advertisements);
                    }
                }
                InstructionFromTimingProgram::Timing => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::Timing,
                        ));
                    }
                }
                InstructionFromTimingProgram::Results
                | InstructionFromTimingProgram::ResultsUpdate => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::ResultList,
                        ));
                    }
                }
                InstructionFromTimingProgram::StartList => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::StartList,
                        ));
                    }
                }
                InstructionFromTimingProgram::SetProperty => {
                    // this is purposefully ignored -> there is no info we can get from it
                }
            },
            IncomingInstruction::FromCameraProgram(inst) => match inst {
                InstructionFromCameraProgram::HeatStartList(list) => {
                    self.handle_heat_start_list(list);
                }
                InstructionFromCameraProgram::HeatStart(start) => {
                    self.handle_heat_start(start);
                }
                InstructionFromCameraProgram::HeatFalseStart(false_start) => {
                    self.handle_heat_false_start(false_start);
                }
                InstructionFromCameraProgram::HeatIntermediate(intermediate) => {
                    self.handle_heat_intermediate(intermediate);
                }
                InstructionFromCameraProgram::HeatWind(wind) => {
                    self.handle_heat_wind(wind);
                }
                InstructionFromCameraProgram::HeatWindMissing(missing_wind) => {
                    store_to_database!(missing_wind, self); // this does not need to and can not be faked -> it is on and has no other purpose
                }
                InstructionFromCameraProgram::HeatFinish(finish) => {
                    self.handle_heat_finish(finish);
                }
                InstructionFromCameraProgram::CompetitorEvaluated(evaluated) => {
                    self.handle_competitor_evaluated(dbss.clone(), evaluated);
                }
                InstructionFromCameraProgram::HeatResult(result) => {
                    self.handle_heat_result(dbss.clone(), result);
                }
                InstructionFromCameraProgram::ZeroTime => {
                    self.handle_heat_reset_display();
                }
                InstructionFromCameraProgram::RaceTime(rt) => {
                    self.handle_race_time_display(rt);
                }
                InstructionFromCameraProgram::IntermediateTime(rt) => {
                    self.handle_intermediate_time_display(rt);
                }
                InstructionFromCameraProgram::EndTime(rt) => {
                    self.handle_end_time_display(rt);
                }
                InstructionFromCameraProgram::DayTime(dt) => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Clock(dt));
                    }
                }
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
                MessageFromWebControl::GetHeats => {
                    trace!("Heats were requested");
                    match HeatStartList::get_all_from_database(&self.database_manager) {
                        Ok(data) => {
                            self.send_message_to_web_control(MessageToWebControl::HeatsMeta(
                                data.into_iter().map(|h| h.into()).collect(),
                            ));
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
                MessageFromWebControl::SelectHeat(id_as_string) => {
                    let id = match Uuid::parse_str(&id_as_string) {
                        Err(e) => {
                            error!(
                                "Received request for uuid that could not be parsed as uuid: {} ( {} )",
                                e, id_as_string
                            );
                            return;
                        }
                        Ok(u) => u,
                    };

                    let data = match get_heat_data(id, &self.database_manager) {
                        Ok(d) => d,
                        Err(e) => {
                            error!("Error when reading heat data from the database: {}", e);
                            return;
                        }
                    };

                    self.send_message_to_web_control(MessageToWebControl::HeatDataMessage(data));
                }
                MessageFromWebControl::Timing => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::Timing,
                        ));
                    }
                }
                MessageFromWebControl::StartList => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::StartList,
                        ));
                    }
                }
                MessageFromWebControl::ResultList => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Timing);
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::ResultList,
                        ));
                    }
                }
                MessageFromWebControl::UpdateTimingSettings(set) => {
                    self.send_message_to_client(MessageFromServerToClient::TimingSettingsUpdate(
                        set,
                    ));
                }
                MessageFromWebControl::RequestTimingSettings => {
                    self.send_message_to_client(MessageFromServerToClient::RequestTimingSettings);
                }
                MessageFromWebControl::SelectHeatForBibMode(uuid) => {
                    self.bib_heat_selection = Some(uuid);
                    self.handle_bib_mode_selection();
                }
                MessageFromWebControl::RequestBibEntryModeData => {
                    self.handle_bib_mode_selection();
                }
                MessageFromWebControl::AddBibEquivalence(eq) => {
                    store_to_database!(eq, self);
                    self.handle_bib_mode_selection();
                }
                MessageFromWebControl::DeleteBibEquivalence(eq) => {
                    match delete_bib_equivalence(eq, &self.database_manager) {
                        Ok(()) => {}
                        Err(e) => {
                            error!("Deletion failed because of: {}", e);
                        }
                    }
                    self.handle_bib_mode_selection();
                }
                MessageFromWebControl::RecordBibRound(bdp_bib) => {
                    let now = Instant::now();

                    let heat_data = match generate_bib_data(
                        self.bib_heat_selection.clone(),
                        &self.database_manager,
                    ) {
                        Some(data) => data.heat_data,
                        None => {
                            error!("Could not record bib event, because no heat data found");
                            return;
                        }
                    };

                    let heat_id = heat_data.start_list.id;

                    if let Some(heat_start) = heat_data.start {
                        if let Some(start) = &self.heat_start_time_instant {
                            let time_of_heat_start = heat_start.time;
                            let time_of_instant_start = start.0.clone();
                            let instant_of_instant_start = start.1.clone();
                            let instant_now = now;

                            let time_diff =
                                instant_now.saturating_duration_since(instant_of_instant_start);

                            let time_of_heat_start_as_duration = Duration::from(time_of_heat_start);
                            let time_of_instant_start_as_duration =
                                Duration::from(time_of_instant_start);

                            let time_diff_compensated = if time_of_heat_start_as_duration
                                > time_of_instant_start_as_duration
                            {
                                time_diff.saturating_sub(
                                    time_of_heat_start_as_duration
                                        .saturating_sub(time_of_instant_start_as_duration),
                                )
                            } else if time_of_heat_start_as_duration
                                < time_of_instant_start_as_duration
                            {
                                time_diff.saturating_sub(
                                    time_of_instant_start_as_duration
                                        .saturating_sub(time_of_heat_start_as_duration),
                                )
                            } else {
                                time_diff
                            };

                            let bdp = BibDataPoint {
                                bib: bdp_bib,
                                heat_id: heat_id,
                                manual: true,
                                race_time: RaceTime::from(time_diff_compensated),
                            };

                            store_to_database!(bdp.clone(), self);
                            self.send_message_to_web_control(
                                MessageToWebControl::BibRoundRecorded(bdp),
                            );
                            self.handle_bib_mode_selection();
                        } else {
                            warn!("Could not record bib event, because no time sync in messages_to_send_out_to_server");
                        }
                    } else {
                        warn!("Could not record bib event, because heat not yet started");
                    }
                }
                MessageFromWebControl::Clock(dt) => {
                    if self.state == ServerState::PassthroughClient {
                        self.send_message_to_client(MessageFromServerToClient::Clock(dt));
                    }
                }
                MessageFromWebControl::RequestWindValues(wvdc) => {
                    match get_wind_readings(wvdc.from, wvdc.to, &self.database_manager) {
                        Err(e) => error!(
                            "Error while querying wind values from databae: {}",
                            e.to_string()
                        ),
                        Ok(vals) => self.send_message_to_web_control(
                            MessageToWebControl::WindMeasurements(vals),
                        ),
                    }
                }
                MessageFromWebControl::InitStaticDatabaseState(_) => {
                    error!("Received update to database static state. That should only be ever possible on an un-initialized database!");
                }
                MessageFromWebControl::RequestStaticDatabaseState => {
                    // dbss is set
                    self.send_message_to_web_control(MessageToWebControl::DatabaseStaticState(
                        dbss.clone(),
                    ));
                }
                MessageFromWebControl::ExportDataToFile => {
                    let file_name = format!("jta-dwa-{}.meetxml", dbss.date.to_string());

                    let meet = generate_meet_data(&dbss, &self.database_manager);

                    write_to_xml_output_file(&self.args, &file_name, meet);
                }
                MessageFromWebControl::CreateAthlete(ath) => {
                    store_to_database_ignore_date!(ath, self);

                    match get_all_athletes_meta_data(&self.database_manager) {
                        Ok(d) => {
                            self.send_message_to_web_control(MessageToWebControl::AthletesData(d))
                        }
                        Err(e) => error!("Encountered error, after creation of athlete: {}", e),
                    }
                }
                MessageFromWebControl::DeleteAthlete(ath_id) => {
                    match delete_athlete(ath_id, &self.database_manager) {
                        Ok(_) => {
                            debug!("Deleted athlete");
                            match get_all_athletes_meta_data(&self.database_manager) {
                                Ok(d) => self.send_message_to_web_control(
                                    MessageToWebControl::AthletesData(d),
                                ),
                                Err(e) => {
                                    error!("Encountered error, after deletion of athlete: {}", e)
                                }
                            }
                        }
                        Err(e) => error!(
                            "Encountered error, while deleting an athlete: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::CreateHeatAssignment(ha) => {
                    match create_heat_assignment(ha, &self.database_manager) {
                        Ok(a) => {
                            debug!("Created heat assignment: {:?}", a);
                            match get_all_athletes_meta_data(&self.database_manager) {
                                Ok(d) => self.send_message_to_web_control(
                                    MessageToWebControl::AthletesData(d),
                                ),
                                Err(e) => {
                                    error!(
                                        "Encountered error, after creation of heat assignment: {}",
                                        e
                                    )
                                }
                            }
                        }
                        Err(e) => error!(
                            "Encountered error, while creating a heat assignment: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::DeleteHeatAssignment(id) => {
                    match delete_heat_assignment(id, &self.database_manager) {
                        Ok(_) => {
                            debug!("Deleted heat assignment");

                            match get_all_athletes_meta_data(&self.database_manager) {
                                Ok(d) => self.send_message_to_web_control(
                                    MessageToWebControl::AthletesData(d),
                                ),
                                Err(e) => error!(
                                    "Encountered error, after deletion of heat assignment: {}",
                                    e
                                ),
                            }
                        }
                        Err(e) => error!(
                            "Encountered error, while deleting a heat assignment: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::RequestAthletes => {
                    match get_all_athletes_meta_data(&self.database_manager) {
                        Ok(d) => {
                            self.send_message_to_web_control(MessageToWebControl::AthletesData(d))
                        }
                        Err(e) => {
                            error!("Encountered error, while getting athlete metadata: {}", e)
                        }
                    }
                }
                MessageFromWebControl::StorePDFConfigurationSetting(set) => {
                    match set.store_to_database(&self.database_manager) {
                        Ok(_) => {
                            debug!("Upserted pdf setting");
                            self.send_out_all_database_settings_to_webclient();
                        }
                        Err(e) => error!(
                            "Encountered error, while upserting a pdf setting: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::DeletePDFConfigurationSetting(id) => {
                    match delete_pdf_setting(id, &self.database_manager) {
                        Ok(_) => {
                            debug!("Deleted pdf setting");
                            self.send_out_all_database_settings_to_webclient();
                        }
                        Err(e) => error!(
                            "Encountered error, while deleting a pdf setting: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::RequestPDFConfigurationSettings => {
                    self.send_out_all_database_settings_to_webclient();
                }
                MessageFromWebControl::GetMainHeat => {
                    self.send_out_main_heat_to_webcontrol();
                }
                MessageFromWebControl::DeleteCompetitorEvaluated(ft) => {
                    match delete_evaluation(ft, &self.database_manager) {
                        Ok(_) => {
                            debug!("Deleted evaluation manually");
                            self.send_out_main_heat_to_webcontrol();
                        }
                        Err(e) => error!(
                            "Encountered error, while deleting an evaluation: {}",
                            e.to_string()
                        ),
                    }
                }
                MessageFromWebControl::SendDebugDisplayCommand(entry) => {
                    self.send_message_to_client(MessageFromServerToClient::PushDisplayEntry(entry));
                }
                MessageFromWebControl::RequestDevMode => {
                    self.send_message_to_web_control(
                        MessageToWebControl::DevModeStatus(dev_mode()),
                    );
                }
                MessageFromWebControl::RequestLicense => {
                    self.send_message_to_web_control(MessageToWebControl::Licensed(Some(
                        product_key.clone(), // at the moment if this answers, we are always licensed. Otherwise it aborts earlier
                    )));
                }
                MessageFromWebControl::RequestConnectionStates => {
                    self.send_current_connection_state_to_webclient();
                }
                MessageFromWebControl::RequestPassword => {
                    self.send_message_to_web_control(MessageToWebControl::Password(
                        self.args.webcontrol_password.clone(),
                    ));
                }
                MessageFromWebControl::SendHeatDataToDisplay(uuid) => {
                    let data = match get_heat_data(uuid, &self.database_manager) {
                        Ok(d) => d,
                        Err(e) => {
                            error!("Error when reading heat data from the database: {}", e);
                            return;
                        }
                    };

                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::Reset,
                    ));
                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::Meta(data.start_list),
                    ));
                    if let Some(result) = data.result {
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::ResultMeta(result),
                        ));
                        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                            TimingUpdate::ResultList,
                        ));
                    }
                }
                // Dev mode and debug signals
                MessageFromWebControl::DevRequestMainHeatStartList => {
                    match fake_main_heat_start_list(dbss, &self.database_manager) {
                        Ok(_) => {
                            self.send_out_main_heat_to_webcontrol();
                        }
                        Err(e) => {
                            error!("Failed to generate fake heat start List: {}", e);
                            return;
                        }
                    };
                }
                MessageFromWebControl::DevReset => {
                    match fake_main_heat_start_list(dbss, &self.database_manager) {
                        Ok(hsl) => match purge_heat_data(hsl.id, &self.database_manager) {
                            Ok(()) => (),
                            Err(e) => {
                                error!("Error when purging heat data: {}", e);
                            }
                        },
                        Err(_) => {}
                    };

                    self.handle_bib_mode_selection();
                    self.send_out_main_heat_to_webcontrol();
                    self.handle_heat_reset_display();
                }
                MessageFromWebControl::DevSendStartList(hsl) => {
                    self.handle_heat_start_list(hsl);
                }
                MessageFromWebControl::DevStartRace(hs) => {
                    self.handle_heat_start(hs);

                    // this is NOT constantly updated, only initial. The client can count on its own
                    // the real one ticks this to keep it in sync
                    self.handle_race_time_display(RaceTime::get_zero_time());
                }
                MessageFromWebControl::DevSendIntermediateSignal(hi) => {
                    self.handle_intermediate_time_display(hi.intermediate_time_at.clone());
                    self.handle_heat_intermediate(hi);
                }
                MessageFromWebControl::DevSendFinishSignal(hf) => {
                    self.handle_end_time_display(hf.race_time.clone());
                    self.handle_heat_finish(hf);
                }
                MessageFromWebControl::DevSendEvaluated(ce) => {
                    self.handle_competitor_evaluated(dbss.clone(), ce);
                }
                MessageFromWebControl::DevSendResultList(hr) => {
                    self.handle_heat_result(dbss.clone(), hr);
                }
                MessageFromWebControl::DevSendWind(wind) => {
                    self.handle_heat_wind(wind);
                }
            },
            IncomingInstruction::FromWindServer(inst) => match inst {
                Measured(wind_measurement) => {
                    store_to_database!(wind_measurement, self);
                }
                Started(started_wind_measurement) => {
                    store_to_database!(started_wind_measurement, self);
                }
            },
            IncomingInstruction::FromExternalDisplayProgram(inst) => match inst {
                Frame(data) => {
                    // this is a bit of a hack, because technically this message comes from the display program
                    trace!("Got command to send a frame inbound (should be from external display program to send back and possibly proxy to our client)");

                    if self.state == ServerState::PassthroughDisplayProgram {
                        if self.comm_channel.web_control_there_to_receive() {
                            self.send_message_to_client(
                                MessageFromServerToClient::DisplayExternalFrame(data.clone()),
                            );
                            self.send_message_to_web_control(
                                MessageToWebControl::CurrentDisplayFrame(data),
                            );
                        } else {
                            // avoid clone in this case
                            self.send_message_to_client(
                                MessageFromServerToClient::DisplayExternalFrame(data),
                            );
                        }
                    }
                }
                ServerInfo => {}
            },
            IncomingInstruction::FromIdcaptureServer(inst) => match inst {
                IDCaptureMessage::JumpToTime(jtt) => {
                    if self.try_work_with_bib_server() {
                        self.send_message_to_bib_server(MessageToBibServer::SeekForTime(
                            SeekForTimeBibServer {
                                timestamp: jtt.to_exchange_float(),
                            },
                        ));
                    }

                    debug!("Jumping to Timestamp: {}", jtt);

                    let heat_data = match generate_bib_data(
                        self.bib_heat_selection.clone(),
                        &self.database_manager,
                    ) {
                        Some(data) => data.heat_data,
                        None => {
                            error!("Could not jump to timestamp, because no heat data found");
                            return;
                        }
                    };

                    if let Some(start) = heat_data.start {
                        let request_as_duration = Duration::from(jtt);
                        let start_as_duration = Duration::from(start.time);

                        self.send_message_to_web_control(MessageToWebControl::HighlightBibEntry(
                            RaceTime::from(request_as_duration.saturating_sub(start_as_duration)),
                        ));
                    } else {
                        warn!("Heat not yet started... Can not jump")
                    }
                }
            },
        }
    }

    fn handle_bib_mode_selection(&mut self) {
        self.send_message_to_web_control(MessageToWebControl::HeatDataSelectionForBibMode(
            generate_bib_data(self.bib_heat_selection.clone(), &self.database_manager),
        ));
    }

    fn handle_heat_start_list(&mut self, list: HeatStartList) {
        store_to_database!(list.clone(), self);
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::Meta(list),
        ));
    }

    fn handle_heat_start(&mut self, start: HeatStart) {
        let now = Instant::now();
        self.heat_start_time_instant = Some((start.time.clone(), now));

        // tell the bib server immediately (real time relevant)
        if self.try_work_with_bib_server() {
            self.send_message_to_bib_server(MessageToBibServer::RaceHasStarted(
                RaceHasStartedBibServer {
                    id: start.id.to_string(),
                    timestamp: start.time.to_exchange_float(),
                },
            ));
        }

        // every heat start, the wind clock gets re-synced (semi-real time relevant)
        if self.try_work_with_wind_server() {
            self.update_wind_server_time_reference(&start.time);
        }

        if self.timing_settings_template.play_sound_on_start {
            self.play_sound(Sound::Beep1);
        }

        store_to_database!(start, self);

        self.handle_bib_mode_selection();
    }

    fn handle_heat_false_start(&mut self, false_start: HeatFalseStart) {
        let id = false_start.id;
        store_to_database!(false_start, self);

        match purge_heat_data(id, &self.database_manager) {
            Ok(()) => (),
            Err(e) => {
                error!("Error when purging heat data: {}", e);
            }
        }

        self.handle_bib_mode_selection();
        self.send_out_main_heat_to_webcontrol();
    }

    fn handle_heat_reset_display(&mut self) {
        // TODO think about if we really want to reset (maybe not if meta-change = off)
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::Reset,
        ));
    }

    fn handle_race_time_display(&mut self, rt: RaceTime) {
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::Running(rt),
        ));
    }

    fn handle_heat_intermediate(&mut self, intermediate: HeatIntermediate) {
        if self.timing_settings_template.play_sound_on_intermediate {
            self.play_sound(Sound::Beep2);
        }

        store_to_database!(intermediate, self);
    }

    fn handle_intermediate_time_display(&mut self, rt: RaceTime) {
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::Intermediate(rt),
        ));
    }

    fn handle_heat_finish(&mut self, finish: HeatFinish) {
        if self.timing_settings_template.play_sound_on_finish {
            self.play_sound(Sound::Beep3);
        }

        store_to_database!(finish, self);
    }

    fn handle_end_time_display(&mut self, rt: RaceTime) {
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::End(rt),
        ));
    }

    fn handle_competitor_evaluated(
        &mut self,
        dbss: DatabaseStaticState,
        evaluated: CompetitorEvaluated,
    ) {
        store_to_database!(evaluated.clone(), self);
        // we can assume, that the "always emit result list on change" setting is active in the camera program
        // for this reason, we ignore singular evaluation emits

        // used in Sprinterkönig and Street run modes -> is quite unnecessary overhead in Track Mode, as there the heats come from external
        match dbss.mode {
            // street races work with evaluations. So this now is their time to shine
            ApplicationMode::StreetLongRun => {
                self.send_out_main_heat_to_webcontrol();
            }
            _ => {
                debug!("Main heat on evaluation is unnecessary overhead in this mode. Will not be generated");
            }
        };

        if self.try_work_with_bib_server() {
            self.send_message_to_bib_server(MessageToBibServer::CompetitorEvaluated(
                CompetitorEvaluatedBibServer {
                    bib: evaluated.competitor_result.competitor.bib,
                    timestamp: evaluated.competitor_result.finish_time.to_exchange_float(),
                },
            ));
        }
    }

    fn handle_heat_result(&mut self, dbss: DatabaseStaticState, result: HeatResult) {
        // this takes a DatabaseStaticState and not a &DatabaseStaticState because it is a self function that borrows mutable and we can not simultaneously borrow the dbss reference
        // I don't wanna extra unwrap the dbss though, because in the switch where this function is used, this is always done

        if let Some(wind) = &result.wind {
            // Can get wind here again, if it was missed
            self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                TimingUpdate::Wind(wind.clone()),
            ));
        }
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::ResultMeta(result.clone()),
        ));

        store_to_database!(result, self); // needs to be before athletes data read

        // used in Street run modes (might be used in Sprinterkönig - did not check) -> is quite unnecessary overhead in Track Mode, as there the heats come from external
        match dbss.mode {
            ApplicationMode::SprinterKing | ApplicationMode::StreetLongRun => {
                match get_all_athletes_meta_data(&self.database_manager) {
                    Ok(d) => self.send_message_to_web_control(MessageToWebControl::AthletesData(d)),
                    Err(e) => {
                        error!(
                            "Encountered error, after heat result possibly changed data: {}",
                            e
                        )
                    }
                }
            }
            _ => {
                debug!("All athletes metadata is unnecessary overhead in this mode. Will not be generated");
            }
        }
    }

    fn handle_heat_wind(&mut self, wind: HeatWind) {
        self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
            TimingUpdate::Wind(wind.wind.clone()),
        ));

        store_to_database!(wind, self);
    }

    fn send_out_main_heat_to_webcontrol(&mut self) {
        match get_main_heat(&self.database_manager) {
            Ok(data) => {
                if let Some(main_heat) = data {
                    self.send_message_to_web_control(MessageToWebControl::MainHeat(
                        main_heat.clone(),
                    ));
                    if dev_mode() {
                        self.send_message_to_web_control(
                            MessageToWebControl::DevMainHeatStartList(main_heat.start_list),
                        );
                    }
                }
            }
            Err(e) => {
                error!("Database loading error for main heat: {}", e);
            }
        }
    }

    fn send_out_all_database_settings_to_webclient(&mut self) {
        match PDFConfigurationSetting::get_all_from_database(&self.database_manager) {
            Ok(data) => {
                self.send_message_to_web_control(
                    MessageToWebControl::PDFConfigurationSettingsData(data),
                );
            }
            Err(e) => {
                error!("Database loading error for pdf settings: {}", e);
            }
        }
    }

    fn try_work_with_bib_server(&self) -> bool {
        self.args.address_bib_server.is_some()
    }

    fn try_work_with_wind_server(&self) -> bool {
        self.args.address_wind_server.is_some()
    }

    fn send_current_connection_state_to_webclient(&mut self) {
        let args = &self.args;

        let camera_program_connected_on_timing_port = self
            .comm_channel
            .connection_check(ConnectionCheck::CameraProgramTimingPort);
        let camera_program_connected_on_data_port = self
            .comm_channel
            .connection_check(ConnectionCheck::CameraProgramDataPort);
        let camera_program_connected_on_xml_port = self
            .comm_channel
            .connection_check(ConnectionCheck::CameraProgramXMLPort);

        let camera_program_connected = camera_program_connected_on_timing_port
            && camera_program_connected_on_data_port
            && camera_program_connected_on_xml_port;

        self.send_message_to_web_control(MessageToWebControl::ConnectionState(ConnectionState {
            // tries
            try_connect_to_wind: self.try_work_with_wind_server(),
            try_conect_to_display_client: args.address_display_client.is_some(),
            try_connect_to_bib: self.try_work_with_bib_server(),
            try_to_connect_to_camera_program: args.address_camera_program.is_some(),
            try_to_connect_to_idcapture: args.address_idcapture_server.is_some(),
            try_to_connect_to_display_passthrough: args.passthrough_to_display_program,
            listening_to_timing_program: args.listen_to_timing_program,
            // ports/addresses
            display_client_address_with_port: format!(
                "{}:{}",
                args.address_display_client
                    .as_ref()
                    .unwrap_or(&String::from(""))
                    .clone(),
                args.display_client_communication_port
            ),
            camera_program_timing_port: args.camera_exchange_timing_port.clone(),
            camera_program_data_port: args.camera_exchange_data_port.clone(),
            camera_program_xml_port: args.camera_exchange_xml_port.clone(),
            camera_program_address: args
                .address_camera_program
                .as_ref()
                .unwrap_or(&String::from(""))
                .clone()
                .clone(),
            wind_address_with_port: format!(
                "{}:{}",
                args.address_wind_server
                    .as_ref()
                    .unwrap_or(&String::from(""))
                    .clone(),
                args.wind_exchange_port
            ),
            bib_address_with_port: format!(
                "{}:{}",
                args.address_bib_server
                    .as_ref()
                    .unwrap_or(&String::from(""))
                    .clone(),
                args.bib_exchange_port
            ),
            idcapture_address_with_port: format!(
                "{}:{}",
                args.address_idcapture_server
                    .as_ref()
                    .unwrap_or(&String::from(""))
                    .clone(),
                args.idcapture_exchange_port
            ),
            display_passthrough_address: args.passthrough_address_display_program.clone(),
            // connection states
            bib_connected: self.comm_channel.bib_server_there_to_receive(),
            display_passthrough_connected: self
                .comm_channel
                .connection_check(ConnectionCheck::ExternalDisplayProgramPassthrough),
            camera_program_connected: camera_program_connected,
            camera_program_connected_on_timing_port: camera_program_connected_on_timing_port,
            camera_program_connected_on_data_port: camera_program_connected_on_data_port,
            camera_program_connected_on_xml_port: camera_program_connected_on_xml_port,
            wind_connected: self.comm_channel.wind_server_there_to_receive(),
            idcapture_connected: self.comm_channel.idcapture_server_there_to_receive(),
            timing_program_is_connected: self.comm_channel.timing_program_there_to_receive(),
        }));
    }

    fn send_out_latest_n_logs_to_webclient(&mut self, n: u32) {
        match get_log_limited(Some(n), &self.database_manager) {
            Ok(data) => {
                self.send_message_to_web_control(MessageToWebControl::Logs(data));
            }
            Err(e) => {
                error!("Database log loading error for logs: {}", e);
            }
        }
    }

    fn get_display_client_state(&self) -> DisplayClientState {
        DisplayClientState {
            alive: self.display_connected,
            external_passthrough_mode: self.state == ServerState::PassthroughDisplayProgram,
            can_switch_mode: self.args.passthrough_to_display_program
                && self.args.listen_to_timing_program,
        }
    }

    fn send_message_to_timing_program(&mut self, inst: InstructionToTimingProgram) {
        if self.comm_channel.timing_program_there_to_receive() {
            match self.comm_channel.send_out_command_to_timing_program(inst) {
                Ok(()) => (),
                Err(e) => error!("Failed to send out instruction: {}", e.to_string()),
            }
        } else {
            debug!("No timing program connected. Skipping sending");
        }
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

    fn send_message_to_bib_server(&mut self, inst: MessageToBibServer) {
        if self.comm_channel.bib_server_there_to_receive() {
            match self.comm_channel.send_out_command_to_bib_server(inst) {
                Ok(()) => (),
                Err(e) => error!(
                    "Failed to send out instruction to bib server: {}",
                    e.to_string()
                ),
            }
        } else {
            debug!("No bib server connected. Skipping sending");
        }
    }

    fn send_message_to_web_control(&mut self, inst: MessageToWebControl) {
        if self.comm_channel.web_control_there_to_receive() {
            match self.comm_channel.send_out_command_to_web_control(inst) {
                Ok(()) => (),
                Err(e) => error!(
                    "Failed to send out instruction to web control: {}",
                    e.to_string()
                ),
            }
        } else {
            debug!("No web control connected. Skipping sending");
        }
    }

    fn update_wind_server_time_reference(&mut self, dt: &DayTime) {
        if self.comm_channel.wind_server_there_to_receive() {
            match self
                .comm_channel
                .send_out_command_to_wind_server(SetTime(dt.clone()))
            {
                Ok(()) => (),
                Err(e) => error!(
                    "Failed to send out instruction to wind server: {}",
                    e.to_string()
                ),
            }
        } else {
            debug!("No wind server connected. Skipping sending");
        }
    }

    fn set_main_display_state(&mut self, state: bool) {
        self.display_connected = state;
    }

    pub fn allows_external_connections(&self) -> bool {
        return self.static_state.is_some();
    }
}

pub enum ClientState {
    Created,
    Idle,
    DisplayText(String),
    DisplayExternalFrame(ImageMeta),
    Advertisements,
    Timing(TimingStateMachine),
    TimingEmptyInit, // will immediately switch to Timing, but read the state machine from self.timing_state_machine_storage
    Clock(ClockState),
}

static STORAGE_BYTES: &[u8] = include_bytes!(env!("STORAGE_BYTES_LOCATION"));
static STORAGE_BYTES_ICONS: &[u8] = include_bytes!(env!("STORAGE_BYTES_ICONS_LOCATION"));

pub struct ClientStateMachine {
    pub product_key: Option<ProductKey>,
    pub state: ClientState,
    messages_to_send_out_to_server: Vec<MessageFromClientToServer>,
    pub frame_counter: u64,
    pub window_state_needs_update: Option<(u32, u32, u32, u32)>,
    pub permanent_images_storage: ImagesStorage,
    pub permanent_icons_storage: IconsStorage,
    pub current_frame_dimensions: Option<(u32, u32)>,
    pub server_imposed_settings: ServerImposedSettings,
    timing_state_machine_storage: Option<TimingStateMachine>,
    timing_settings_template: TimingSettings,
    outbound_connection_open: bool,
    self_sender: Sender<MessageFromServerToClient>,
    frametime_tracker: FrametimeTracker,
}
impl ClientStateMachine {
    pub fn new(args: &Args, sender: Sender<MessageFromServerToClient>) -> Self {
        debug!("Start loading Images storage");
        let images_storage = ImagesStorage::from_bytes(STORAGE_BYTES);
        let icons_storage = IconsStorage::from_bytes(STORAGE_BYTES_ICONS);
        debug!("DONE loading Images storage");

        Self {
            product_key: product_key_valid(None).ok(), // IF dev mode, this sets a default
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
            window_state_needs_update: None,
            permanent_images_storage: images_storage,
            permanent_icons_storage: icons_storage,
            current_frame_dimensions: None,
            server_imposed_settings: ServerImposedSettings::new(args),
            timing_state_machine_storage: None,
            timing_settings_template: TimingSettings::new(args),
            outbound_connection_open: false,
            self_sender: sender,
            frametime_tracker: FrametimeTracker::new(),
        }
    }

    pub fn parse_server_command(&mut self, msg: MessageFromServerToClient) {
        match msg {
            MessageFromServerToClient::ProductKey(key_string) => {
                match product_key_valid(Some(&key_string)) {
                    Ok(key) => {
                        info!("Valid product key loaded from server: {:?}", key);
                        self.product_key = Some(key);
                    }
                    Err(e) => {
                        self.product_key = None;
                        error!("Product key loading error from server: {}", e);
                    }
                }
            }
            MessageFromServerToClient::RequestVersion => {
                info!("Version was requested. Communication established!!");
                if matches!(self.state, ClientState::Created) {
                    // initial setting, there is no relevant timing state to cache
                    self.state = ClientState::Idle;
                }
                self.push_new_message(MessageFromClientToServer::Version(String::from(
                    crate_version!(),
                )));
            }
            MessageFromServerToClient::DisplayText(text) => {
                debug!("Server requested display mode to be switched to text");
                self.switch_mode_with_stashing_timing_state(ClientState::DisplayText(text));
            }
            MessageFromServerToClient::ServerImposedSettings(settings) => {
                // size/position properties of the window are not reflected in internal state but by the real window -> needs instructions to change
                let (x, y, w, h) = settings.position;
                debug!("Server requested an update of the window position/size");
                self.window_state_needs_update = Some((x, y, w, h));

                // store other (mainly rendering) settings
                self.server_imposed_settings = settings;

                // do not forget to do this on canvase size changes !! -> TODO maybe extract somewhere or do in true window resize handler
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
                self.switch_mode_with_stashing_timing_state(ClientState::Idle);
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
                    self.switch_mode_with_stashing_timing_state(ClientState::DisplayExternalFrame(
                        image.get_rescaled(w, h),
                    ));
                } else {
                    self.switch_mode_with_stashing_timing_state(ClientState::DisplayExternalFrame(
                        image,
                    ));
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
                self.switch_mode_with_stashing_timing_state(ClientState::Advertisements);
            }
            MessageFromServerToClient::Timing => {
                self.switch_mode_with_stashing_timing_state(ClientState::TimingEmptyInit);
            }
            MessageFromServerToClient::TimingStateUpdate(update) => {
                if let Some(tsm) = &mut self.timing_state_machine_storage {
                    tsm.process_update(update);
                } else {
                    match &mut self.state {
                        ClientState::Timing(tsm) => {
                            tsm.process_update(update);
                        }
                        _ => {
                            let mut new_timing_state_machine = TimingStateMachine::new(
                                &self.permanent_images_storage,
                                &self.timing_settings_template,
                                self.self_sender.clone(),
                            );
                            new_timing_state_machine.process_update(update);

                            // there was no timing state machine to update
                            self.timing_state_machine_storage = Some(new_timing_state_machine);
                        }
                    }
                }
            }
            MessageFromServerToClient::TimingSettingsUpdate(set) => {
                // force the new timing settings into possibly existing Timing state machines:
                if let Some(tsm) = &mut self.timing_state_machine_storage {
                    tsm.overwrite_settings(&set);
                }
                match &mut self.state {
                    ClientState::Timing(tsm) => {
                        tsm.overwrite_settings(&set);
                    }
                    _ => {}
                };

                self.update_internal_knowledge_of_timing_settings(set);
            }
            MessageFromServerToClient::RequestTimingSettings => {
                self.push_new_message(MessageFromClientToServer::TimingSettingsState(
                    self.timing_settings_template.clone(),
                ));
            }
            MessageFromServerToClient::Clock(dt) => {
                trace!("Switch to clock mode"); // this is called often if the camera program is in clock mode, so only trace
                self.switch_mode_with_stashing_timing_state(ClientState::Clock(ClockState::new(
                    &dt,
                )));
            }
            MessageFromServerToClient::PushDisplayEntry(entry) => {
                // force the new entry into possibly existing Timing state machines:
                if let Some(tsm) = &mut self.timing_state_machine_storage {
                    tsm.insert_new_display_entry(&entry);
                }
                match &mut self.state {
                    ClientState::Timing(tsm) => {
                        tsm.insert_new_display_entry(&entry);
                    }
                    _ => {}
                };
            }
            MessageFromServerToClient::ClientInternal(client_internal_message) => {
                match client_internal_message {
                    ClientInternalMessageFromServerToClient::EmitTimingSettingsUpdate(set) => {
                        self.update_internal_knowledge_of_timing_settings(set);
                    }
                }
            }
        }
    }

    pub fn digest_frame_time_percentage(&mut self, percentage: u64) {
        self.frametime_tracker
            .digest_new_frame_time_percentage(percentage);

        if let Some(report) = self.frametime_tracker.needs_to_send_out_report() {
            self.push_new_message(MessageFromClientToServer::FrametimeReport(report));
        }
    }

    fn update_internal_knowledge_of_timing_settings(&mut self, set: TimingSettings) {
        // update template so is timing mode gets re-initialized, the settings are applied
        self.timing_settings_template = set;

        // notify of the successful update
        self.push_new_message(MessageFromClientToServer::TimingSettingsState(
            self.timing_settings_template.clone(),
        ));
    }

    fn switch_mode_with_stashing_timing_state(&mut self, new_state: ClientState) {
        match std::mem::replace(&mut self.state, ClientState::TimingEmptyInit) {
            ClientState::Timing(timing_sm) => {
                self.timing_state_machine_storage = Some(timing_sm);
            }
            other => {
                // restore the previous state
                self.state = other;
            }
        }

        match new_state {
            ClientState::Timing(t) => {
                self.timing_state_machine_storage = None;

                self.state = ClientState::Timing(t);
            }
            ClientState::TimingEmptyInit => {
                match std::mem::replace(&mut self.timing_state_machine_storage, None) {
                    Some(timing_sm) => {
                        self.state = ClientState::Timing(timing_sm);
                    }
                    None => {
                        self.state = ClientState::Timing(TimingStateMachine::new(
                            &self.permanent_images_storage,
                            &self.timing_settings_template,
                            self.self_sender.clone(),
                        ));
                    }
                }
            }
            s => {
                self.state = s;
            }
        }
    }

    pub fn push_new_message(&mut self, msg: MessageFromClientToServer) {
        if self.outbound_connection_open
            || !matches!(msg, MessageFromClientToServer::CurrentWindow(_))
        {
            // avoid spamming warnings by emitting frames while nothing is connected
            self.messages_to_send_out_to_server.push(msg);
        }
    }

    pub fn advance_counters(&mut self) {
        self.frame_counter += 1;
    }

    pub fn get_one_message_to_send(&mut self) -> Option<MessageFromClientToServer> {
        self.messages_to_send_out_to_server.pop()
    }

    pub fn set_outbound_connection_open(&mut self, conn_open: bool) {
        self.outbound_connection_open = conn_open;
    }
}
