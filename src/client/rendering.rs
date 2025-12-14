use crate::{
    client::{
        rasterizing::{
            clear, draw_image, draw_image_at_opacity, draw_text, draw_text_right_aligned,
            draw_text_scrolling_with_width, fill_with_color, RasterizerMeta, JTA_COLOR,
        },
        FRAME_TIME_NS,
    },
    interface::{ClientState, ClientStateMachine},
};

pub fn render_client_frame(meta: &mut RasterizerMeta, state: &mut ClientStateMachine) {
    match &mut state.state {
        ClientState::Created | ClientState::TimingEmptyInit => {
            clear(meta);
        }
        ClientState::Idle => {
            fill_with_color(JTA_COLOR, meta);

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
            draw_text(&text, 55.0, 22.0, 20.0, meta);
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
            fill_with_color(JTA_COLOR, meta);

            // Title
            if let Some(tsm) = &timing_state_machine.meta {
                draw_text_scrolling_with_width(
                    &tsm.title,
                    10.0,
                    10.0,
                    20.0,
                    (meta.texture_width - 30) as f32,
                    state.frame_counter,
                    meta,
                );
            }
            // line 1
            draw_text_right_aligned(
                &timing_state_machine
                    .get_main_display_race_time()
                    .optimize_representation_for_display(Some(
                        timing_state_machine.settings.max_decimal_places_after_comma,
                    ))
                    .to_string(),
                130.0,
                30.0,
                20.0,
                meta,
            );
            if timing_state_machine.race_finished() {
                draw_text("Finished", 150.0, 30.0, 20.0, meta);
            }
            // line 2
            if let Some(hts) = timing_state_machine.get_held_display_race_time() {
                draw_text(
                    &hts.held_at_time
                        .optimize_representation_for_display(Some(
                            timing_state_machine.settings.max_decimal_places_after_comma,
                        ))
                        .to_string(),
                    10.0,
                    50.0,
                    20.0,
                    meta,
                );
                if let Some(held_distance) = hts.held_at_m {
                    draw_text(&format!("{}m", held_distance), 150.0, 50.0, 20.0, meta);
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
        ClientState::Clock(clock_state) => {
            fill_with_color(JTA_COLOR, meta);

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
