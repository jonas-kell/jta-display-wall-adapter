use crate::{
    args::Args,
    client::{ClockState, TimingSettings, TimingStateMachine, TimingUpdate},
    database::{
        get_heat_data, get_log_limited, get_wind_readings, purge_heat_data, DatabaseManager,
        DatabaseSerializable,
    },
    file::read_image_files,
    instructions::{
        IncomingInstruction, InstructionCommunicationChannel, InstructionFromCameraProgram,
        InstructionFromTimingProgram, InstructionToTimingProgram,
    },
    server::{camera_program_types::HeatStartList, AudioPlayer, Sound},
    times::DayTime,
    webserver::{DisplayClientState, MessageFromWebControl, MessageToWebControl},
    wind::format::{
        MessageToWindServer::SetTime,
        WindMessageBroadcast::{Measured, Started},
    },
};
use clap::crate_version;
use images_core::images::{ImageMeta, ImagesStorage};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

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
    Timing,
    TimingStateUpdate(TimingUpdate),
    TimingSettingsUpdate(TimingSettings),
    RequestTimingSettings,
    Clock(DayTime),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageFromClientToServer {
    Version(String),
    CurrentWindow(Vec<u8>),
    TimingSettingsState(TimingSettings),
}

#[derive(PartialEq, Eq, Clone)]
pub enum ServerState {
    PassthroughClient,
    PassthroughDisplayProgram,
}

macro_rules! store_to_database_log_conditionally {
    ($value:expr, $self_val:expr, $log:expr) => {
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
    };
}

macro_rules! store_to_database {
    ($value:expr, $self_val:expr) => {
        store_to_database_log_conditionally!($value, $self_val, true);
    };
}

pub struct ServerStateMachine {
    args: Args,
    pub state: ServerState,
    comm_channel: InstructionCommunicationChannel,
    display_connected: bool,
    database_manager: DatabaseManager,
    sound_engine: Option<AudioPlayer>,
    timing_settings_template: TimingSettings,
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

        Self {
            args: args.clone(),
            state: ServerState::PassthroughClient,
            comm_channel, // only used to send instructions outwards. Rest is done via incoming commands (there is a handler that continously takes them out of the channel and forwards them into us)
            display_connected: false,
            database_manager,
            sound_engine,
            timing_settings_template: TimingSettings::new(args),
        }
    }

    fn play_sound(&self, sound: Sound) {
        match &self.sound_engine {
            None => error!("Can not play sounds, as sound engine not initialized"),
            Some(se) => se.play_audio_background(sound),
        }
    }

    pub async fn parse_client_command(&mut self, msg: MessageFromClientToServer) {
        // handle all messages
        match msg {
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
                    self.send_message_to_timing_program(InstructionToTimingProgram::SendServerInfo);
                }

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
                if self.state == ServerState::PassthroughClient
                    && self.args.listen_to_timing_program
                {
                    self.send_message_to_timing_program(InstructionToTimingProgram::SendFrame(
                        data,
                    ));
                }
                // ping the state to the web control
                self.send_message_to_web_control(MessageToWebControl::DisplayClientState(
                    self.get_display_client_state(),
                ));
            }
            MessageFromClientToServer::TimingSettingsState(set) => {
                self.timing_settings_template = set.clone();
                self.send_message_to_web_control(MessageToWebControl::TimingSettingsState(set));
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
                InstructionFromTimingProgram::SendFrame(data) => {
                    // this is a bit of a hack, because technically this message comes from the display program
                    trace!("Got command to send a frame inbound (should be from external display program to send back and possibly proxy to our client)");
                    if self.state == ServerState::PassthroughDisplayProgram {
                        self.send_message_to_client(
                            MessageFromServerToClient::DisplayExternalFrame(data),
                        );
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
                    }
                }
                InstructionFromTimingProgram::SetProperty => {
                    // this is purposefully ignored -> there is no info we can get from it
                }
                inst => error!("Unhandled instruction from timing program: {}", inst),
            },
            IncomingInstruction::FromCameraProgram(inst) => match inst {
                InstructionFromCameraProgram::HeatStartList(list) => {
                    store_to_database!(list, self);
                }
                InstructionFromCameraProgram::HeatStart(start) => {
                    if self.timing_settings_template.play_sound_on_start {
                        self.play_sound(Sound::Beep1);
                    }

                    // every heat start, the wind clock gets re-synced
                    self.update_wind_server_time_reference(&start.time);

                    store_to_database!(start, self);
                }
                InstructionFromCameraProgram::HeatFalseStart(false_start) => {
                    let id = false_start.id;
                    store_to_database!(false_start, self);

                    match purge_heat_data(id, &self.database_manager) {
                        Ok(()) => (),
                        Err(e) => {
                            error!("Error when purging heat data: {}", e);
                        }
                    }
                }
                InstructionFromCameraProgram::HeatIntermediate(intermediate) => {
                    if self.timing_settings_template.play_sound_on_intermediate {
                        self.play_sound(Sound::Beep2);
                    }
                    store_to_database!(intermediate, self);
                }
                InstructionFromCameraProgram::HeatWind(wind) => {
                    store_to_database!(wind, self);
                }
                InstructionFromCameraProgram::HeatWindMissing(missing_wind) => {
                    store_to_database!(missing_wind, self);
                }
                InstructionFromCameraProgram::HeatFinish(finish) => {
                    if self.timing_settings_template.play_sound_on_finish {
                        self.play_sound(Sound::Beep3);
                    }
                    store_to_database!(finish, self);
                }
                InstructionFromCameraProgram::CompetitorEvaluated(evaluated) => {
                    store_to_database!(evaluated, self);
                }
                InstructionFromCameraProgram::HeatResult(result) => {
                    store_to_database!(result, self);
                }
                InstructionFromCameraProgram::ZeroTime => {
                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::Reset,
                    ));
                }
                InstructionFromCameraProgram::RaceTime(rt) => {
                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::Running(rt),
                    ));
                }
                InstructionFromCameraProgram::EndTime(rt) => {
                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::End(rt),
                    ));
                }
                InstructionFromCameraProgram::IntermediateTime(rt) => {
                    self.send_message_to_client(MessageFromServerToClient::TimingStateUpdate(
                        TimingUpdate::Intermediate(rt),
                    ));
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
            },
            IncomingInstruction::FromWindServer(inst) => match inst {
                Measured(wind_measurement) => {
                    store_to_database!(wind_measurement, self);
                }
                Started(started_wind_measurement) => {
                    store_to_database!(started_wind_measurement, self);
                }
            },
        }
    }

    fn send_out_latest_n_logs_to_webclient(&mut self, n: u32) {
        match get_log_limited(Some(n), &self.database_manager) {
            Ok(data) => {
                self.send_message_to_web_control(MessageToWebControl::Logs(data));
            }
            Err(e) => {
                error!("Database log loading error: {}", e);
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
        match self.comm_channel.send_out_command_to_timing_program(inst) {
            Ok(()) => (),
            Err(e) => error!("Failed to send out instruction: {}", e.to_string()),
        }
    }

    pub async fn make_server_request_client_version(&mut self) {
        self.send_message_to_client(MessageFromServerToClient::RequestVersion)
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

    fn send_message_to_web_control(&mut self, inst: MessageToWebControl) {
        match self.comm_channel.send_out_command_to_web_control(inst) {
            Ok(()) => (),
            Err(e) => error!(
                "Failed to send out instruction to web control: {}",
                e.to_string()
            ),
        }
    }

    fn update_wind_server_time_reference(&mut self, dt: &DayTime) {
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
    }

    pub fn set_main_display_state(&mut self, state: bool) {
        self.display_connected = state;
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

static STORAGE_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/image_storage.bin"));

pub struct ClientStateMachine {
    pub state: ClientState,
    messages_to_send_out_to_server: Vec<MessageFromClientToServer>,
    pub frame_counter: u64,
    pub window_state_needs_update: Option<(u32, u32, u32, u32)>,
    pub permanent_images_storage: ImagesStorage,
    pub current_frame_dimensions: Option<(u32, u32)>,
    pub slideshow_duration_nr_ms: u32,
    pub slideshow_transition_duration_nr_ms: u32,
    timing_state_machine_storage: Option<TimingStateMachine>,
    timing_settings_template: TimingSettings,
}
impl ClientStateMachine {
    pub fn new(args: &Args) -> Self {
        debug!("Start loading Images storage");
        let images_storage = ImagesStorage::from_bytes(STORAGE_BYTES);
        debug!("DONE loading Images storage");

        Self {
            state: ClientState::Created,
            messages_to_send_out_to_server: Vec::new(),
            frame_counter: 0,
            window_state_needs_update: None,
            permanent_images_storage: images_storage,
            current_frame_dimensions: None,
            slideshow_duration_nr_ms: args.slideshow_duration_nr_ms,
            slideshow_transition_duration_nr_ms: args.slideshow_transition_duration_nr_ms,
            timing_state_machine_storage: None,
            timing_settings_template: TimingSettings::new(args),
        }
    }

    pub fn parse_server_command(&mut self, msg: MessageFromServerToClient) {
        match msg {
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
                let (x, y, w, h) = settings.position;
                debug!("Server requested an update of the window position/size");
                self.window_state_needs_update = Some((x, y, w, h));

                debug!(
                    "Server set the slideshow duration to {} ms",
                    settings.slideshow_duration_in_ms
                );
                self.slideshow_duration_nr_ms = settings.slideshow_duration_in_ms;

                // do not forget to do this on size changes -> TODO maybe extract somewhere or do in true window resize handler
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
                    tsm.update_race_time(update);
                } else {
                    match &mut self.state {
                        ClientState::Timing(tsm) => {
                            tsm.update_race_time(update);
                        }
                        _ => {
                            let mut new_timing_state_machine = TimingStateMachine::new(
                                &self.permanent_images_storage,
                                &self.timing_settings_template,
                            );
                            new_timing_state_machine.update_race_time(update);

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

                // update template so is timing mode gets re-initialized, the settings are applied
                self.timing_settings_template = set;

                // notify of the successful update
                self.push_new_message(MessageFromClientToServer::TimingSettingsState(
                    self.timing_settings_template.clone(),
                ));
            }
            MessageFromServerToClient::RequestTimingSettings => {
                self.push_new_message(MessageFromClientToServer::TimingSettingsState(
                    self.timing_settings_template.clone(),
                ));
            }
            MessageFromServerToClient::Clock(dt) => {
                trace!("Switch to clock mode"); // this is called often if the camera program is in clock mode, so only trace
                self.state = ClientState::Clock(ClockState::new(&dt));
            }
        }
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
        self.messages_to_send_out_to_server.push(msg);
    }

    pub fn advance_counters(&mut self) {
        self.frame_counter += 1;
    }

    pub fn get_one_message_to_send(&mut self) -> Option<MessageFromClientToServer> {
        self.messages_to_send_out_to_server.pop()
    }
}
