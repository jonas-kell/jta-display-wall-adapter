use fontdue::{
    layout::{Layout, LayoutSettings, TextStyle},
    Font,
};

pub struct RasterizerMeta<'a> {
    pub font: &'a Font,
    pub font_layout: &'a mut Layout,
    pub frame: &'a mut [u8],
    pub texture_width: usize,
    pub texture_height: usize,
}

pub fn draw_text(text: &str, x: f32, y: f32, script_size: f32, meta: &mut RasterizerMeta) {
    let filtered: String = text.replace('\n', "\r").replace("\r\r", "\r");

    meta.font_layout.reset(&LayoutSettings {
        x: x,
        y: y,
        max_width: None, // no line wrap
        ..LayoutSettings::default()
    });

    meta.font_layout
        .append(&[meta.font], &TextStyle::new(&filtered, script_size, 0));

    for glyph in meta.font_layout.glyphs() {
        // Rasterize the glyph at the specified font size
        let (metrics, bitmap) = meta.font.rasterize(glyph.parent, script_size);

        // Get the target pixel position
        let x_cursor = glyph.x as isize;
        let y_cursor = glyph.y as isize;

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let px = bitmap[y * metrics.width + x];
                let tx = x_cursor + x as isize;
                let ty = y_cursor + y as isize;

                if tx >= 0
                    && ty >= 0
                    && (tx as usize) < meta.texture_width
                    && (ty as usize) < meta.texture_height
                {
                    let i = ((ty as usize) * meta.texture_width + (tx as usize)) * 4;
                    if i + 3 < meta.frame.len() {
                        meta.frame[i] = 255; // R
                        meta.frame[i + 1] = 255; // G
                        meta.frame[i + 2] = 255; // B
                        meta.frame[i + 3] = px; // alpha
                    }
                }
            }
        }
    }
}

pub fn clear(meta: &mut RasterizerMeta) {
    meta.frame.fill(0); // clear the window
}
