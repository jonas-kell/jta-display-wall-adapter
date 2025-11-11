use crate::{
    interface::{ClientState, ClientStateMachine},
    rasterizing::{clear, draw_image, draw_text, fill_with_color, RasterizerMeta, JTA_COLOR},
};

pub fn render_client_frame(meta: &mut RasterizerMeta, state: &ClientStateMachine) {
    clear(meta);

    match &state.state {
        ClientState::Created => (),
        ClientState::Idle => {
            fill_with_color(JTA_COLOR, meta);

            draw_image(
                0,
                0,
                100,
                100,
                &state.permanent_images_storage.jta_logo,
                meta,
            );
        }
        ClientState::DisplayText(text) => {
            draw_text(&text, 55.0, 22.0, 20.0, meta);
        }
    }
}
