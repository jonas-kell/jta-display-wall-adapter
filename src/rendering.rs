use crate::{
    interface::{ClientState, ClientStateMachine},
    rasterizing::{clear, draw_text, RasterizerMeta},
};

pub fn render_client_frame(meta: &mut RasterizerMeta, state: &ClientStateMachine) {
    clear(meta);

    match &state.state {
        ClientState::Idle | ClientState::Created => (),
        ClientState::DisplayText(text) => {
            draw_text(&text, 55.0, 22.0, 20.0, meta);
        }
    }
}
