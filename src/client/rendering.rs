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
        FRAME_TIME_NS,
    },
    interface::{ClientState, ClientStateMachine},
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

            match &timing_state_machine.timing_mode {
                TimingMode::Timing => {
                    match timing_state_machine.settings.mode {
                        TimingTimeDisplayMode::StreetRun => {
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
                            draw_text("#1 Max Mustermann", 10.0, 60.0, 18.0, meta);
                            draw_text("#2 John Doe", 10.0, 80.0, 18.0, meta);
                            draw_text("#3 Miriam Musterfrau (Runde 1/4)", 10.0, 100.0, 18.0, meta);
                        }
                        TimingTimeDisplayMode::TimeBigAndHold => {
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
                                0.0,
                                (window_width - 2.0 * border) as usize,
                                window_height as usize,
                                &mut cache.font_size_cache_time_main_number_a,
                                Some(&mut cache.main_number_display_width_debouncer_a),
                                Some(&mut cache.main_number_display_size_debouncer_a),
                                meta,
                            );
                        }
                        TimingTimeDisplayMode::TimeBigAndHoldTop => {
                            draw_text("Not implemented", 10.0, 10.0, 20.0, meta);
                            // if timing_state_machine.race_finished() {
                            //     draw_text("Finished", 150.0, 30.0, 20.0, meta);
                            // }
                            // if let Some(hts) = timing_state_machine.get_held_display_race_time() {
                            //     draw_text(
                            //         &hts.held_at_time
                            //             .optimize_representation_for_display(Some(
                            //                 timing_state_machine
                            //                     .settings
                            //                     .max_decimal_places_after_comma,
                            //             ))
                            //             .to_string(),
                            //         10.0,
                            //         50.0,
                            //         20.0,
                            //         meta,
                            //     );
                            //     if let Some(held_distance) = hts.held_at_m {
                            //         draw_text(
                            //             &format!("{}m", held_distance),
                            //             150.0,
                            //             50.0,
                            //             20.0,
                            //             meta,
                            //         );
                            //     }
                            // }
                        }
                        TimingTimeDisplayMode::TimeBigAndHoldWithRunName => {
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
                            draw_text("Not implemented", 10.0, 10.0, 20.0, meta);
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
                TimingMode::StartList => {
                    draw_text("Start list", 100.0, 30.0, 20.0, meta);
                }
                TimingMode::ResultList => {
                    draw_text("Result list", 100.0, 30.0, 20.0, meta);
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
}
