use crate::{
    args::Args,
    database::{
        get_all_athletes_meta_data, get_all_heat_assignments, ApplicationMode, DatabaseManager,
        DatabaseStaticState,
    },
    file::{create_file_if_not_there_and_write, make_sure_folder_exists},
    helpers::uuids_from_seed,
    server::camera_program_types::{
        Athlete, AthleteWithMetadata, DistanceType, Event, Gender, Heat, HeatAssignment,
        HeatCompetitor, Meet, Session,
    },
    times::DayTime,
};
use chrono::Datelike;
use chrono::NaiveDate;
use std::{path::Path, time::Duration};
use uuid::Uuid;

pub fn write_to_xml_output_file(args: &Args, file_name: &str, data: Meet) {
    let path_string = match &args.export_folder_path {
        Some(a) => a,
        None => {
            error!("Args has no output folder provided (export_folder_path). Can never save to anywhere");
            return;
        }
    };

    let folder = Path::new(path_string);

    match make_sure_folder_exists(folder) {
        Ok(()) => (),
        Err(e) => {
            error!("Could not create or access output folder: {}", e);
            return;
        }
    };

    let file_path = folder.join(file_name);
    let data = match data.as_xml_serealized_string() {
        Ok(d) => d,
        Err(e) => {
            error!("Could not convert to XML string: {}", e);
            return;
        }
    };

    match create_file_if_not_there_and_write(&file_path, &data) {
        Ok(()) => debug!("Export output written to file"),
        Err(e) => error!("Could not write the output to the file: {}", e),
    };
}

fn rounded_year(date: NaiveDate) -> i32 {
    match date.month() {
        11 | 12 => date.year() + 1,
        _ => date.year(),
    }
}

fn generate_heats_spk(
    event_key: String,
    distance: u32,
    index: u8,
    start: &mut CountingOrderedStartTime,
    athletes_meta: &Vec<AthleteWithMetadata>,
    all_heats: &Vec<HeatAssignment>,
) -> Vec<Heat> {
    let _ = event_key;

    let mut relevant_heats: Vec<&HeatAssignment> = all_heats
        .iter()
        .filter(|h| h.distance == distance && h.heat_descriminator == index)
        .collect();
    relevant_heats.sort_by(|h1, h2| h1.id.cmp(&h2.id));

    let mut heats = Vec::new();

    for (i, heat) in relevant_heats.into_iter().enumerate() {
        let mut athletes_with_lanes: Vec<(&u32, &Athlete)> = heat
            .athlete_ids
            .iter()
            .filter_map(
                |(lane, id)| match athletes_meta.iter().find(|a| a.athlete.id == *id) {
                    Some(a) => Some((lane, &a.athlete)),
                    None => return None,
                },
            )
            .collect();
        athletes_with_lanes.sort_by(|a, b| a.0.cmp(b.0));
        let competitors: Vec<HeatCompetitor> = athletes_with_lanes
            .into_iter()
            .map(|(lane, athlete)| HeatCompetitor {
                lane: lane.clone(),
                bib: athlete.bib.clone(),
                class: athlete.gender.to_string(),
                gender: athlete.gender.to_string(),
                club: athlete.club.clone(),
                first_name: athlete.first_name.clone(),
                last_name: athlete.last_name.clone(),
                id: athlete.id.to_string(),
                nation: athlete.nation.clone(),
                disqualified: None,
            })
            .collect();

        heats.push(Heat {
            id: heat.heat_id.clone(),
            distance,
            distance_type: DistanceType::Normal,
            name: format!("SPK {}m-{}, Run {}", distance, index, i + 1),
            scheduled_start_time: start.get_next(),
            competitors,
        });
    }

    heats
}

fn generate_heats_street_race(event_key: String, distance: u32) -> Vec<Heat> {
    let ids = uuids_from_seed(&format!("{}_heat", event_key), 1);
    let id = ids[0];

    [Heat {
        id,
        distance,
        distance_type: DistanceType::Normal,
        name: "Main Heat".into(),
        scheduled_start_time: DayTime::from_hms_opt(10, 10, 0).unwrap(),
        competitors: [HeatCompetitor {
            // TODO from db
            bib: 101,
            class: Gender::Male.to_string(),
            club: "Testverein".into(),
            first_name: "Test Name".into(),
            last_name: "Test Nachname".into(),
            gender: Gender::Male.to_string(),
            id: Uuid::new_v4().to_string(),
            lane: 1,
            nation: "GER".into(),
            disqualified: None,
        }]
        .into(),
    }]
    .into()
}

struct CountingOrderedStartTime {
    start: DayTime,
    current_index: u32,
}
impl CountingOrderedStartTime {
    pub fn new(init: DayTime) -> Self {
        Self {
            current_index: 0,
            start: init,
        }
    }

    pub fn get_next(&mut self) -> DayTime {
        let res = self
            .start
            .add_duration(Duration::from_secs(self.current_index as u64 * 60));
        self.current_index += 1;
        res
    }
}

pub fn generate_meet_data(dbss: &DatabaseStaticState, manager: &DatabaseManager) -> Meet {
    let _ = Gender::Female;
    let _ = Gender::Mixed;
    let _ = Gender::Male;

    let event_key = format!("{}-{}", dbss.mode.to_string(), dbss.date.to_string());

    let mut events: Vec<Event> = Vec::new();
    match dbss.mode {
        ApplicationMode::TrackCompetition => (), // the program does not generate meetxml for this case (currently)
        ApplicationMode::SprinterKing => {
            let mut start = CountingOrderedStartTime::new(DayTime::from_hms_opt(10, 0, 0).unwrap());
            let athletes_meta = match get_all_athletes_meta_data(manager) {
                Ok(e) => e,
                Err(e) => {
                    error!(
                        "Error while generating export - could not read from database: {}",
                        e
                    );
                    Vec::new()
                }
            };
            let all_heats = match get_all_heat_assignments(manager) {
                Ok(e) => e,
                Err(e) => {
                    error!(
                        "Error while generating export - could not read from database: {}",
                        e
                    );
                    Vec::new()
                }
            };

            for distance in [15u32, 20, 30] {
                let ids = uuids_from_seed(&format!("{}_event_id_{}", event_key, distance), 2);
                let id_a = ids[0];
                let id_b = ids[1];

                events.push(Event {
                    distance,
                    distance_type: DistanceType::Normal,
                    id: id_a,
                    name: format!("SPK {}m Run 1", distance),
                    scheduled_start_time: start.get_next(),
                    heats: generate_heats_spk(
                        event_key.clone(),
                        distance,
                        1,
                        &mut start,
                        &athletes_meta,
                        &all_heats,
                    ),
                });
                events.push(Event {
                    distance,
                    distance_type: DistanceType::Normal,
                    id: id_b,
                    name: format!("SPK {}m Run 2", distance),
                    scheduled_start_time: start.get_next(),
                    heats: generate_heats_spk(
                        event_key.clone(),
                        distance,
                        2,
                        &mut start,
                        &athletes_meta,
                        &all_heats,
                    ),
                });
            }
        }
        ApplicationMode::StreetLongRun => {
            let ids = uuids_from_seed(&format!("{}_long_run", event_key.clone()), 1);
            let id = ids[0];
            let distance = 1000u32; // TODO dynamically set this

            events.push(Event {
                distance,
                distance_type: DistanceType::Normal,
                id,
                name: format!("Main Race"),
                scheduled_start_time: DayTime::from_hms_opt(10, 0, 0).unwrap(),
                heats: generate_heats_street_race(event_key, distance),
            })
        }
    }

    Meet {
        name: match dbss.mode {
            ApplicationMode::SprinterKing => {
                format!("Sprinter König {}", rounded_year(dbss.date.clone()))
            }
            ApplicationMode::StreetLongRun => {
                format!("Lauf {}", dbss.date.to_string())
            }
            ApplicationMode::TrackCompetition => {
                format!("Bahnveranstaltung {}", dbss.date.to_string())
            }
        },
        id: dbss.meet_id.clone(),
        city: dbss.meet_city.clone(),
        sessions: [Session {
            date: dbss.date.clone(),
            location: dbss.meet_location.clone(),
            events,
        }]
        .into(),
    }
}
