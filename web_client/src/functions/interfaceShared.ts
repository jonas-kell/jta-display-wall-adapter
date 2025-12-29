export type Uuid = string;

export enum TimingTimeDisplayMode {
    TimeBigAndHold = "TimeBigAndHold",
    TimeBigAndHoldTop = "TimeBigAndHoldTop",
    TimeBigAndHoldWithRunName = "TimeBigAndHoldWithRunName",
    TimeBigAndHoldTopWithRunName = "TimeBigAndHoldTopWithRunName",
    StreetRun = "StreetRun",
}

export type TimingSettings = {
    fireworks_on_intermediate: boolean;
    fireworks_on_finish: boolean;
    max_decimal_places_after_comma: number;
    hold_time_ms: number;
    display_time_ms: number;
    play_sound_on_start: boolean;
    play_sound_on_intermediate: boolean;
    play_sound_on_finish: boolean;
    can_currently_update_meta: boolean;
    time_continues_running: boolean;
    switch_to_start_list_automatically: boolean;
    switch_to_timing_automatically: boolean;
    switch_to_results_automatically: boolean;
    mode: TimingTimeDisplayMode;
};

export enum ApplicationMode {
    TrackCompetition = "TrackCompetition",
    StreetLongRun = "StreetLongRun",
    SprinterKing = "SprinterKing",
}

export type DatabaseStaticState = {
    mode: ApplicationMode;
    date: string;
    meet_id: string;
    meet_city: string;
    meet_location: string;
};

export enum Gender {
    Male = "Male",
    Female = "Female",
    Mixed = "Mixed",
}

export type Athlete = {
    id: Uuid;
    gender: Gender;
    bib: number;
    club: string;
    first_name: string;
    last_name: string;
    nation: string;
    spk_guess: number | null;
    street_run_rounds: number | null;
};

export type HeatAssignment = {
    id: number;
    heat_id: Uuid;
    distance: number;
    heat_descriminator: number;
    athlete_ids: { [key: number]: Uuid };
};

export enum PDFSettingFor {
    Bib = "Bib",
    Certificate = "Certificate",
}

export type PDFConfigurationSetting = {
    id: Uuid;
    for: PDFSettingFor;
    pos_x: number;
    pos_y: number;
    content: PDFConfigurationContent;
};

export type PDFConfigurationContent = PDFConfigurationContentText | PDFConfigurationContentReference; //...;

export type PDFConfigurationContentText = {
    type: "PDFConfigurationContentText";
    text: string;
};

export type PDFConfigurationContentReference = {
    type: "PDFConfigurationContentReference";
    reference: string;
};

export type DisplayEntry = {
    bib: number;
    name: string;
    round: number;
    max_rounds: number;
};
