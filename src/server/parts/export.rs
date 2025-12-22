use crate::{
    args::Args,
    database::{ApplicationMode, DatabaseStaticState},
    file::{create_file_if_not_there_and_write, make_sure_folder_exists},
    server::camera_program_types::{
        DistanceType, Event, Gender, Heat, HeatCompetitor, Meet, Session,
    },
    times::DayTime,
};
use chrono::Datelike;
use chrono::NaiveDate;
use std::path::Path;
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

pub fn generate_meet_data(dbss: &DatabaseStaticState) -> Meet {
    let _ = Gender::Female;
    let _ = Gender::Mixed;
    let _ = Gender::Male;

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
        city: "Irgendeine Stadt".into(), // TODO
        sessions: [Session {
            date: dbss.date.clone(),
            location: "Irgendein Stadion".into(), // TODO
            events: [Event {
                distance: 100,
                distance_type: DistanceType::Normal,
                id: Uuid::new_v4(),
                name: "Test Event".into(),
                scheduled_start_time: DayTime::from_hms_opt(10, 10, 0).unwrap(),
                heats: [Heat {
                    id: Uuid::new_v4(),
                    distance: 100,
                    distance_type: DistanceType::Normal,
                    name: "Test Event Heat 1".into(),
                    scheduled_start_time: DayTime::from_hms_opt(10, 10, 0).unwrap(),
                    competitors: [HeatCompetitor {
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
                .into(),
            }]
            .into(),
        }]
        .into(),
    }
}
