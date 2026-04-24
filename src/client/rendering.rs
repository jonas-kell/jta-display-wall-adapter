use super::timing::TimingTimeDisplayMode;
use crate::{
    client::{
        rasterizing::{
            clear, draw_image, draw_image_at_opacity, draw_text, draw_text_as_big_as_possible,
            draw_text_as_big_as_possible_right_aligned, draw_text_right_aligned,
            draw_text_scrolling_with_width, fill_box_with_color, fill_with_color,
            FontSizeChooserCache, FontSizeDebouncer, FontWidthDebouncer, RasterizerMeta,
            JTA_GRAY_COLOR, JTA_GREEN_COLOR,
        },
        timing::TimingMode,
        TimingSettings, TimingStateMachine, FRAME_TIME_NS,
    },
    interface::{ClientState, ClientStateMachine},
    server::camera_program_types::HeatCompetitor,
    times::RaceTime,
};

pub struct RenderCache {
    main_number_display_width_debouncer_street_race: FontWidthDebouncer,
    font_size_cache_freetext: FontSizeChooserCache,
    font_size_cache_time_main_number_a: FontSizeChooserCache,
    font_size_cache_time_main_number_b: FontSizeChooserCache,
    font_size_cache_time_main_number_c: FontSizeChooserCache,
    font_size_cache_time_main_number_d: FontSizeChooserCache,
    main_number_display_width_debouncer_a: FontWidthDebouncer,
    main_number_display_width_debouncer_b: FontWidthDebouncer,
    main_number_display_width_debouncer_c: FontWidthDebouncer,
    main_number_display_width_debouncer_d: FontWidthDebouncer,
    main_number_display_size_debouncer_a: FontSizeDebouncer,
    main_number_display_size_debouncer_b: FontSizeDebouncer,
    main_number_display_size_debouncer_c: FontSizeDebouncer,
    main_number_display_size_debouncer_d: FontSizeDebouncer,
}
impl RenderCache {
    pub fn new() -> Self {
        Self {
            // relevant to avoid jittering
            main_number_display_width_debouncer_street_race: FontWidthDebouncer::new(),
            main_number_display_width_debouncer_a: FontWidthDebouncer::new(),
            main_number_display_width_debouncer_b: FontWidthDebouncer::new(),
            main_number_display_width_debouncer_c: FontWidthDebouncer::new(),
            main_number_display_width_debouncer_d: FontWidthDebouncer::new(),
            main_number_display_size_debouncer_a: FontSizeDebouncer::new(),
            main_number_display_size_debouncer_b: FontSizeDebouncer::new(),
            main_number_display_size_debouncer_c: FontSizeDebouncer::new(),
            main_number_display_size_debouncer_d: FontSizeDebouncer::new(),
            // only caching for performance
            font_size_cache_freetext: FontSizeChooserCache::new(),
            font_size_cache_time_main_number_a: FontSizeChooserCache::new(),
            font_size_cache_time_main_number_b: FontSizeChooserCache::new(),
            font_size_cache_time_main_number_c: FontSizeChooserCache::new(),
            font_size_cache_time_main_number_d: FontSizeChooserCache::new(),
        }
    }
}

pub fn render_client_frame(
    meta: &mut RasterizerMeta,
    state: &mut ClientStateMachine,
    cache: &mut RenderCache,
) {
    let intermediate = state.convert_to_table_info_ro();

    match &mut state.state {
        ClientState::Created | ClientState::TimingEmptyInit => {
            clear(meta);
        }
        ClientState::Idle => {
            fill_with_color(JTA_GRAY_COLOR, meta);

            let logo = &state.permanent_images_storage.jta_logo;

            let window_width: f32 = meta.texture_width as f32;
            let window_height: f32 = meta.texture_height as f32;
            let logo_width: f32 = logo.width as f32;
            let logo_height: f32 = logo.height as f32;

            // Calculate the scale factor to fit the logo in the window while maintaining aspect ratio
            let mut new_width = window_width;
            let mut new_height = logo_height * window_width / logo_width;
            if new_height > window_height {
                new_height = window_height;
                new_width = logo_width * window_height / logo_height;
            }

            // Center the logo
            let pos_x = ((window_width - new_width as f32) / 2.0).round() as u32;
            let pos_y = ((window_height - new_height as f32) / 2.0).round() as u32;

            draw_image(
                pos_x,
                pos_y,
                &state.permanent_images_storage.cached_rescaler.scale_cached(
                    logo,
                    new_width.round() as u32,
                    new_height.round() as u32,
                ),
                meta,
            );
        }
        ClientState::DisplayText(text) => {
            clear(meta);
            draw_text_as_big_as_possible(
                &text,
                5.0,
                0.0,
                meta.texture_width.saturating_sub(10),
                meta.texture_height,
                &mut cache.font_size_cache_freetext,
                meta,
            );
        }
        ClientState::DisplayExternalFrame(image) => {
            draw_image(0, 0, &image, meta);
        }
        ClientState::Advertisements => {
            let nr_images = state.permanent_images_storage.advertisement_images.len();
            if nr_images > 0 {
                let frames_per_image = ((state.server_imposed_settings.slideshow_duration_nr_ms
                    * 1000000)
                    / (FRAME_TIME_NS as u32))
                    + 1; // +1 to avoid dividing by 0
                let current_frame = state.frame_counter;
                let frame_index = (current_frame / frames_per_image as u64) % (nr_images as u64);

                let image_to_render = state
                    .permanent_images_storage
                    .advertisement_images
                    .get(frame_index as usize);

                match image_to_render {
                    None => error!("Error when selecting the image to display"),
                    Some(img) => {
                        let resized = state.permanent_images_storage.cached_rescaler.scale_cached(
                            img,
                            meta.texture_width as u32,
                            meta.texture_height as u32,
                        );

                        draw_image(0, 0, &resized, meta);

                        // blend to the next frame if desired
                        let frames_per_transition = ((std::cmp::min(
                            state.server_imposed_settings.slideshow_transition_duration_nr_ms,
                            state.server_imposed_settings.slideshow_duration_nr_ms - 1, // transition may not be longer than the slideshow itself.
                        ) as u64 * 1000000) // ms to ns
                            / (FRAME_TIME_NS))
                            as u32
                            + 1; // +1 to avoid dividing by 0
                        let frame_of_image = (current_frame % frames_per_image as u64) as u32;
                        let frames_of_image_without_transition =
                            frames_per_image - frames_per_transition;
                        if frame_of_image > frames_of_image_without_transition {
                            // we are in the transition -> linearly map the opacity
                            let frame_of_transition =
                                frame_of_image - frames_of_image_without_transition;
                            let opacity =
                                ((255 * frame_of_transition) / frames_per_transition) as u8;

                            // render over top if possible (with scaled opacity)
                            if let Some(imge_to_render_over_top) = state
                                .permanent_images_storage
                                .advertisement_images
                                .get(((frame_index + 1) % nr_images as u64) as usize)
                            {
                                let imge_to_render_over_top_resized =
                                    state.permanent_images_storage.cached_rescaler.scale_cached(
                                        imge_to_render_over_top,
                                        meta.texture_width as u32,
                                        meta.texture_height as u32,
                                    );
                                draw_image_at_opacity(
                                    0,
                                    0,
                                    &imge_to_render_over_top_resized,
                                    opacity,
                                    meta,
                                );
                            }
                        }
                    }
                }
            } else {
                warn!("There are no advertisement images loaded");
                draw_text("No images", 10.0, 10.0, 20.0, meta);
            }
        }
        ClientState::Timing(timing_state_machine) => {
            let to_set = match timing_state_machine.settings.max_decimal_places_after_comma {
                -1 => 0u8,
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 4,
                4 => 4,
                _ => 4,
            };
            cache
                .main_number_display_width_debouncer_street_race
                .set_debounce_number_chars(to_set);
            cache
                .main_number_display_width_debouncer_a
                .set_debounce_number_chars(to_set);
            cache
                .main_number_display_size_debouncer_a
                .set_debounce_number_chars(to_set);

            fill_with_color(JTA_GRAY_COLOR, meta);

            fn has_title(mode: &TimingTimeDisplayMode) -> bool {
                match mode {
                    TimingTimeDisplayMode::StreetRun => false,
                    TimingTimeDisplayMode::TimeBigAndHold => false,
                    TimingTimeDisplayMode::TimeBigAndHoldTop => false,
                    TimingTimeDisplayMode::TimeBigAndHoldTopWithRunName => true,
                    TimingTimeDisplayMode::TimeBigAndHoldWithRunName => true,
                }
            }

            let window_width: f32 = meta.texture_width as f32;
            let window_height: f32 = meta.texture_height as f32;
            let title_height = meta.texture_height / 5;
            let text_height = meta.texture_height / 6;
            let border = window_width / 36.0;

            // Title
            if !(timing_state_machine.timing_mode == TimingMode::Timing
                && !has_title(&timing_state_machine.settings.mode))
            {
                fill_box_with_color(
                    0,
                    0,
                    meta.texture_width,
                    title_height,
                    JTA_GREEN_COLOR,
                    meta,
                );

                if let Some(tsm) = &timing_state_machine.meta {
                    draw_text_scrolling_with_width(
                        &tsm.title,
                        border,
                        0.0,
                        text_height as f32,
                        meta.texture_width as f32 - 2.0 * border,
                        state.frame_counter,
                        meta,
                    );
                }
            }

            let info_for_table = timing_state_machine
                .settings
                .convert_to_table_info_ro(intermediate);

            match &mut timing_state_machine.timing_mode {
                TimingMode::Timing => {
                    match timing_state_machine.settings.mode {
                        TimingTimeDisplayMode::StreetRun => {
                            let street_run_line_height = window_height / 6.0;
                            let street_run_font_size = window_height / 6.6666666;
                            let line_1_y = window_height / 2.0;
                            let line_2_y = window_height / 2.0 + street_run_line_height;
                            let line_3_y = window_height / 2.0 + 2.0 * street_run_line_height;

                            draw_text_right_aligned(
                                &timing_state_machine
                                    .get_main_display_race_time()
                                    .optimize_representation_for_display(Some(
                                        timing_state_machine
                                            .settings
                                            .max_decimal_places_after_comma,
                                    ))
                                    .to_string(),
                                330.0,
                                -5.0,
                                60.0,
                                Some(&mut cache.main_number_display_width_debouncer_street_race),
                                meta,
                            );

                            let entries = timing_state_machine
                                .get_display_entries_at_lines_and_advance_frame_countdown();
                            for (y_pos, entry_opt) in [
                                (line_1_y, entries.0),
                                (line_2_y, entries.1),
                                (line_3_y, entries.2),
                            ] {
                                if let Some(entry) = entry_opt {
                                    draw_text(
                                        &format!("{}", entry.bib),
                                        border,
                                        y_pos,
                                        street_run_font_size,
                                        meta,
                                    );
                                    draw_text_scrolling_with_width(
                                        &format!("{}", entry.name),
                                        border + window_width / 9.0 + border / 2.0,
                                        y_pos,
                                        street_run_font_size,
                                        window_width
                                            - window_width / 9.0
                                            - window_width / 12.0
                                            - window_width / 9.0
                                            - border / 2.0
                                            - 3.0 * border,
                                        state.frame_counter,
                                        meta,
                                    );
                                    draw_image(
                                        (window_width
                                            - window_width / 9.0
                                            - window_width / 12.0
                                            - border)
                                            as u32,
                                        y_pos as u32,
                                        &state
                                            .permanent_icons_storage
                                            .cached_rescaler
                                            .scale_cached(
                                                &state.permanent_icons_storage.round_icon,
                                                (window_width / 12.0) as u32,
                                                (window_height / 6.0) as u32,
                                            ),
                                        meta,
                                    );
                                    draw_text(
                                        &format!("{}/{}", entry.round, entry.max_rounds),
                                        window_width - window_width / 9.0,
                                        y_pos,
                                        street_run_font_size,
                                        meta,
                                    );
                                }
                            }
                        }
                        TimingTimeDisplayMode::TimeBigAndHold => {
                            if let Some(wind_text) = timing_state_machine.race_wind() {
                                draw_image(
                                    (border) as u32,
                                    (border as f32 / 2.0) as u32,
                                    &state.permanent_icons_storage.cached_rescaler.scale_cached(
                                        &state.permanent_icons_storage.wind_icon,
                                        (window_width / 12.0) as u32,
                                        (window_height / 6.0) as u32,
                                    ),
                                    meta,
                                );
                                draw_text(
                                    &wind_text,
                                    (border) as f32 + (window_width / 12.0),
                                    border as f32 / 2.0,
                                    text_height as f32,
                                    meta,
                                );
                            }
                            draw_text_as_big_as_possible_right_aligned(
                                &timing_state_machine
                                    .get_main_display_race_time()
                                    .optimize_representation_for_display(Some(
                                        timing_state_machine
                                            .settings
                                            .max_decimal_places_after_comma,
                                    ))
                                    .to_string(),
                                window_width - border,
                                border,
                                (window_width - 2.0 * border) as usize,
                                (window_height - border / 4.0) as usize,
                                &mut cache.font_size_cache_time_main_number_a,
                                Some(&mut cache.main_number_display_width_debouncer_a),
                                Some(&mut cache.main_number_display_size_debouncer_a),
                                meta,
                            );
                        }
                        TimingTimeDisplayMode::TimeBigAndHoldTop => {
                            if let Some(top_text) = get_holding_top_text(timing_state_machine) {
                                draw_text_right_aligned(
                                    &top_text,
                                    window_width - border,
                                    border as f32 / 2.0,
                                    text_height as f32,
                                    None,
                                    meta,
                                );
                                if timing_state_machine.race_finished()
                                    && timing_state_machine.time_continues_running()
                                {
                                    draw_image(
                                        (border) as u32 + (window_width / 2.8) as u32, // TODO calculate size of holding top text and move this right respectively
                                        (border as f32 / 2.0) as u32,
                                        &state
                                            .permanent_icons_storage
                                            .cached_rescaler
                                            .scale_cached(
                                                &state.permanent_icons_storage.finish_icon,
                                                (window_width / 12.0) as u32,
                                                (window_height / 6.0) as u32,
                                            ),
                                        meta,
                                    );
                                }
                            } else {
                                if timing_state_machine.race_finished()
                                    && timing_state_machine.time_continues_running()
                                {
                                    draw_image(
                                        (window_width - border - (window_width / 12.0)) as u32,
                                        (border as f32 / 2.0) as u32,
                                        &state
                                            .permanent_icons_storage
                                            .cached_rescaler
                                            .scale_cached(
                                                &state.permanent_icons_storage.finish_icon,
                                                (window_width / 12.0) as u32,
                                                (window_height / 6.0) as u32,
                                            ),
                                        meta,
                                    );
                                }
                            }
                            if let Some(wind_text) = timing_state_machine.race_wind() {
                                draw_image(
                                    (border) as u32,
                                    (border as f32 / 2.0) as u32,
                                    &state.permanent_icons_storage.cached_rescaler.scale_cached(
                                        &state.permanent_icons_storage.wind_icon,
                                        (window_width / 12.0) as u32,
                                        (window_height / 6.0) as u32,
                                    ),
                                    meta,
                                );
                                draw_text(
                                    &wind_text,
                                    (border) as f32 + (window_width / 12.0),
                                    border as f32 / 2.0,
                                    text_height as f32,
                                    meta,
                                );
                            }
                            draw_text_as_big_as_possible_right_aligned(
                                &timing_state_machine
                                    .get_main_display_race_time()
                                    .optimize_representation_for_display(Some(
                                        timing_state_machine
                                            .settings
                                            .max_decimal_places_after_comma,
                                    ))
                                    .to_string(),
                                window_width - border,
                                title_height as f32,
                                (window_width - 2.0 * border) as usize,
                                window_height as usize - title_height,
                                &mut cache.font_size_cache_time_main_number_b,
                                Some(&mut cache.main_number_display_width_debouncer_b),
                                Some(&mut cache.main_number_display_size_debouncer_b),
                                meta,
                            );
                        }
                        TimingTimeDisplayMode::TimeBigAndHoldWithRunName => {
                            if let Some(wind_text) = timing_state_machine.race_wind() {
                                draw_image(
                                    (border) as u32,
                                    (title_height as f32 * 1.1) as u32,
                                    &state.permanent_icons_storage.cached_rescaler.scale_cached(
                                        &state.permanent_icons_storage.wind_icon,
                                        (window_width / 12.0) as u32,
                                        (window_height / 6.0) as u32,
                                    ),
                                    meta,
                                );
                                draw_text(
                                    &wind_text,
                                    (border) as f32 + (window_width / 12.0),
                                    title_height as f32 * 1.15,
                                    text_height as f32,
                                    meta,
                                );
                            }
                            draw_text_as_big_as_possible_right_aligned(
                                &timing_state_machine
                                    .get_main_display_race_time()
                                    .optimize_representation_for_display(Some(
                                        timing_state_machine
                                            .settings
                                            .max_decimal_places_after_comma,
                                    ))
                                    .to_string(),
                                window_width - border,
                                title_height as f32,
                                (window_width - 2.0 * border) as usize,
                                window_height as usize - title_height,
                                &mut cache.font_size_cache_time_main_number_c,
                                Some(&mut cache.main_number_display_width_debouncer_c),
                                Some(&mut cache.main_number_display_size_debouncer_c),
                                meta,
                            );
                        }
                        TimingTimeDisplayMode::TimeBigAndHoldTopWithRunName => {
                            if let Some(top_text) = get_holding_top_text(timing_state_machine) {
                                draw_text_right_aligned(
                                    &top_text,
                                    window_width - border,
                                    title_height as f32 * 1.15,
                                    text_height as f32,
                                    None,
                                    meta,
                                );
                                if timing_state_machine.race_finished()
                                    && timing_state_machine.time_continues_running()
                                {
                                    draw_image(
                                        (border) as u32 + (window_width / 2.8) as u32, // TODO calculate size of holding top text and move this right respectively
                                        (title_height as f32 * 1.1) as u32,
                                        &state
                                            .permanent_icons_storage
                                            .cached_rescaler
                                            .scale_cached(
                                                &state.permanent_icons_storage.finish_icon,
                                                (window_width / 12.0) as u32,
                                                (window_height / 6.0) as u32,
                                            ),
                                        meta,
                                    );
                                }
                            } else {
                                if timing_state_machine.race_finished()
                                    && timing_state_machine.time_continues_running()
                                {
                                    draw_image(
                                        (window_width - border - (window_width / 12.0)) as u32,
                                        (title_height as f32 * 1.1) as u32,
                                        &state
                                            .permanent_icons_storage
                                            .cached_rescaler
                                            .scale_cached(
                                                &state.permanent_icons_storage.finish_icon,
                                                (window_width / 12.0) as u32,
                                                (window_height / 6.0) as u32,
                                            ),
                                        meta,
                                    );
                                }
                            }
                            if let Some(wind_text) = timing_state_machine.race_wind() {
                                draw_image(
                                    (border) as u32,
                                    (title_height as f32 * 1.1) as u32,
                                    &state.permanent_icons_storage.cached_rescaler.scale_cached(
                                        &state.permanent_icons_storage.wind_icon,
                                        (window_width / 12.0) as u32,
                                        (window_height / 6.0) as u32,
                                    ),
                                    meta,
                                );
                                draw_text(
                                    &wind_text,
                                    (border) as f32 + (window_width / 12.0),
                                    title_height as f32 * 1.15,
                                    text_height as f32,
                                    meta,
                                );
                            }
                            draw_text_as_big_as_possible_right_aligned(
                                &timing_state_machine
                                    .get_main_display_race_time()
                                    .optimize_representation_for_display(Some(
                                        timing_state_machine
                                            .settings
                                            .max_decimal_places_after_comma,
                                    ))
                                    .to_string(),
                                window_width - border,
                                title_height as f32 * 1.5, // takes somewhat more space, push main time down a little
                                (window_width - 2.0 * border) as usize,
                                window_height as usize - title_height,
                                &mut cache.font_size_cache_time_main_number_d,
                                Some(&mut cache.main_number_display_width_debouncer_d),
                                Some(&mut cache.main_number_display_size_debouncer_d),
                                meta,
                            );
                        }
                    }

                    // animations
                    if let Some(over_top_player) = &mut timing_state_machine.over_top_animation {
                        match over_top_player.get_current_frame(
                            meta.texture_width as u32,
                            meta.texture_height as u32,
                            state.frame_counter,
                            &mut state.permanent_images_storage.cached_rescaler,
                        ) {
                            Some(frame) => draw_image(0, 0, &frame, meta),
                            None => (),
                        }
                    }
                }
                TimingMode::StartList(tms) => {
                    let list = match &timing_state_machine.meta {
                        Some(meta) => match &meta.start_list {
                            Some(list) => list.competitors.clone(),
                            None => Vec::new(),
                        },
                        None => Vec::new(),
                    }
                    .iter()
                    .map(|a| ListLine {
                        number: a.lane,
                        athlete: a.clone(),
                        res: None,
                    })
                    .collect();

                    draw_table(
                        info_for_table,
                        list,
                        true,
                        tms,
                        meta,
                        0.0,
                        (title_height + 1) as f32,
                        window_width,
                        window_height - 1.0 - title_height as f32,
                    );
                }
                TimingMode::ResultList(tms) => {
                    let list = match &timing_state_machine.meta {
                        Some(meta) => match &meta.result {
                            Some(list) => list.competitors_evaluated.clone(),
                            None => Vec::new(),
                        },
                        None => Vec::new(),
                    }
                    .iter()
                    .map(|a| ListLine {
                        number: a.rank,
                        athlete: a.competitor.clone(),
                        res: Some(a.runtime_full_precision.clone()),
                    })
                    .collect();

                    // TODO somehow note of the "competitors left to evaluate" maybe
                    draw_table(
                        info_for_table,
                        list,
                        false,
                        tms,
                        meta,
                        0.0,
                        (title_height + 1) as f32,
                        window_width,
                        window_height - 1.0 - title_height as f32,
                    );
                }
            }
        }
        ClientState::Clock(clock_state) => {
            fill_with_color(JTA_GRAY_COLOR, meta);

            draw_text("Clock:", 10.0, 10.0, 20.0, meta);
            draw_text(
                &clock_state.get_currently_computed_day_time().to_string(),
                10.0,
                30.0,
                20.0,
                meta,
            );
        }
    }

    if state.product_key.is_none() {
        draw_text_as_big_as_possible(
            "Unlicensed",
            5.0,
            0.0,
            meta.texture_width.saturating_sub(10),
            meta.texture_height,
            &mut cache.font_size_cache_freetext,
            meta,
        );
    }
}

fn get_holding_top_text(timing_state_machine: &TimingStateMachine) -> Option<String> {
    if let Some(hts) = timing_state_machine.get_held_display_race_time() {
        let time_string = hts
            .held_at_time
            .optimize_representation_for_display(Some(
                timing_state_machine.settings.max_decimal_places_after_comma,
            ))
            .to_string();

        if let Some(held_distance) = hts.held_at_m {
            return Some(format!("{}m: {}", held_distance, time_string));
        } else {
            return Some(time_string);
        }
    }

    return None;
}

fn construct_list_name_repr(athlete: &HeatCompetitor) -> String {
    format!(
        "{} {}",
        athlete.last_name.to_ascii_uppercase(),
        athlete.first_name
    )
}

#[derive(PartialEq, Eq, Clone)]
pub struct TableMetaStorage {
    animation_frame_counter: u128,
}
impl TableMetaStorage {
    pub fn new() -> Self {
        Self {
            animation_frame_counter: 0,
        }
    }
}

struct ListLine {
    pub number: u32,
    pub athlete: HeatCompetitor,
    pub res: Option<RaceTime>,
}

/// THIS DOES NOT need a mutable pointer to self. And not a copy of TimingStateMachine. Both would suffice as pointers.
/// But currently that is what I have to make it work... : // TODO refactor and make pretty
struct TSMForTableRenderIntermediate {
    pub table_duration_nr_ms: u32,
}
impl ClientStateMachine {
    fn convert_to_table_info_ro(&self) -> TSMForTableRenderIntermediate {
        TSMForTableRenderIntermediate {
            table_duration_nr_ms: self.server_imposed_settings.table_duration_nr_ms,
        }
    }
}
struct TSMForTableRender {
    pub max_decimal_places_after_comma: i8,
    pub table_duration_nr_ms: u32,
    pub animations_paused: bool,
    pub no_lines_in_lists: u8,
    pub display_bibs_in_start_list: bool,
}
impl TimingSettings {
    fn convert_to_table_info_ro(
        &mut self,
        intermediate: TSMForTableRenderIntermediate,
    ) -> TSMForTableRender {
        TSMForTableRender {
            max_decimal_places_after_comma: self.max_decimal_places_after_comma,
            table_duration_nr_ms: intermediate.table_duration_nr_ms,
            animations_paused: self.list_animations_stopped,
            no_lines_in_lists: self.entries_in_lists,
            display_bibs_in_start_list: self.display_bibs_in_start_list,
        }
    }
}

fn draw_table(
    list_settings: TSMForTableRender,
    lines: Vec<ListLine>,
    start_list: bool,
    table_meta: &mut TableMetaStorage,
    meta: &mut RasterizerMeta,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) {
    let mut lines = lines;
    let lines_on_page = list_settings.no_lines_in_lists.max(1) as u64;

    let has_a_third_col = (list_settings.display_bibs_in_start_list && start_list) || !start_list;

    let frames_per_page =
        ((list_settings.table_duration_nr_ms * 1000000) / (FRAME_TIME_NS as u32)) + 1;

    const NUMBER_SPACE_FRACTION: f32 = 0.08;
    const IN_BETWEEN_SPACE_FRACTION: f32 = 0.015;
    const RESULT_SPACE_FRACTION_TEMPLATE: f32 = 0.20;
    let result_space_fraction: f32 = match has_a_third_col {
        true => RESULT_SPACE_FRACTION_TEMPLATE + IN_BETWEEN_SPACE_FRACTION,
        false => 0.0,
    };
    let identifier_space_fraction: f32 =
        1.0 - NUMBER_SPACE_FRACTION - result_space_fraction - IN_BETWEEN_SPACE_FRACTION;

    lines.sort_by(|a, b| a.number.cmp(&b.number));

    let lines_in_total = lines.len() as u64;
    let no_pages = if lines_in_total == 0 {
        1
    } else {
        if lines_in_total % lines_on_page == 0 {
            lines_in_total / lines_on_page
        } else {
            (lines_in_total as f32 / lines_on_page as f32).ceil() as u64
        }
    };
    let we_are_on_page: u64 =
        ((table_meta.animation_frame_counter / (frames_per_page as u128)) as u64) % no_pages;
    let frame_on_page: u64 =
        (table_meta.animation_frame_counter % (frames_per_page as u128)) as u64;

    let line_height: f32 = (height as f32) / (lines_on_page as f32);
    for (i, line) in lines.iter().enumerate() {
        if ((i as u64) >= (we_are_on_page * lines_on_page))
            && ((i as u64) < (we_are_on_page + 1) * lines_on_page)
        {
            let line_y_start = ((i % lines_on_page as usize) as f32) * line_height + y;

            draw_text_as_big_as_possible(
                &format!("{}", line.number),
                x,
                line_y_start,
                (width * NUMBER_SPACE_FRACTION) as usize,
                (line_height) as usize,
                &mut FontSizeChooserCache::new(), // TODO this is REALLY inefficient. But there si no time to store it currently... Sorry...
                meta,
            );
            draw_text_scrolling_with_width(
                &construct_list_name_repr(&line.athlete),
                x + (width * NUMBER_SPACE_FRACTION),
                line_y_start,
                line_height * 0.85,
                width * identifier_space_fraction,
                frame_on_page,
                meta,
            );

            if has_a_third_col {
                let text = match start_list {
                    true => match list_settings.display_bibs_in_start_list {
                        true => line.athlete.bib.to_string(),
                        false => String::from(""),
                    },
                    false => {
                        if let Some(rt) = &line.res {
                            rt.optimize_representation_for_display(Some(
                                list_settings.max_decimal_places_after_comma,
                            ))
                            .to_string()
                        } else {
                            String::from("")
                        }
                    }
                };

                draw_text_as_big_as_possible(
                    &text,
                    width
                        * (NUMBER_SPACE_FRACTION
                            + identifier_space_fraction
                            + IN_BETWEEN_SPACE_FRACTION),
                    line_y_start,
                    (width * (result_space_fraction - IN_BETWEEN_SPACE_FRACTION)) as usize,
                    (line_height) as usize,
                    &mut FontSizeChooserCache::new(), // TODO this is REALLY inefficient. But there si no time to store it currently... Sorry...
                    meta,
                );
            }
        }
    }

    // finally increase the animation frame counter after rendering a frame
    if !list_settings.animations_paused {
        table_meta.animation_frame_counter += 1;
    }
}
