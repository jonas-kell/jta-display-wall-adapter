use crate::{client::FRAME_TIME_NS, interface::ServerImposedSettings};
use core::f32;
use fontdue::{
    layout::{GlyphPosition, Layout, LayoutSettings, TextStyle},
    Font,
};
use image::Rgba;
use image::{DynamicImage, ImageBuffer};
use images_core::images::ImageMeta;
use std::collections::VecDeque;

pub struct RasterizerMeta<'a> {
    pub font: &'a Font,
    pub font_layout: &'a mut Layout,
    pub frame: &'a mut [u8],
    pub texture_width: usize,
    pub texture_height: usize,
    pub server_imposed_settings: ServerImposedSettings,
}
impl<'a> RasterizerMeta<'a> {
    pub fn get_buffer_as_image(&self) -> Result<ImageMeta, String> {
        // Validate buffer size
        let expected_len = self.texture_width * self.texture_height * 4;
        if self.frame.len() != expected_len {
            return Err("Frame buffer size does not match expected dimensions.".into());
        }

        // Clone the frame into a Vec<u8>, since `ImageBuffer` needs ownership
        let buf = self.frame.to_vec();

        // SAFETY: image crate requires owned data for ImageBuffer,
        // but we just cloned it.
        let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            match ImageBuffer::from_vec(self.texture_width as u32, self.texture_height as u32, buf)
            {
                Some(b) => b,
                None => return Err("Invalid image buffer size".into()),
            };

        let dynamic = DynamicImage::ImageRgba8(img_buffer);

        Ok(ImageMeta::from_dynamic_image(dynamic))
    }
}

pub fn draw_text_as_big_as_possible(
    text: &str,
    pos_x: f32,
    pos_y: f32,
    max_width: usize,
    max_height: usize,
    meta: &mut RasterizerMeta,
) {
    const TEXT_SIZE_FINDING_STEP: f32 = 2.0;
    const MAX_STEPS: usize = 400;

    let mut current_text_size = 0f32;
    let mut step = 0;
    let mut result_width = 0;
    let mut result_height = 0;
    loop {
        layout_text(
            text,
            Some(0f32),
            None,
            current_text_size + TEXT_SIZE_FINDING_STEP,
            meta,
        );
        let (_, text_width, text_height) = text_meta_data(0f32, &meta.font_layout);

        if text_width > max_width || text_height > max_height {
            break;
        } else {
            result_width = text_width;
            result_height = text_height;
            current_text_size += TEXT_SIZE_FINDING_STEP; // now it is set, to what it was tested on before.
        }

        step += 1;
        if step > MAX_STEPS {
            break;
        }
    }

    let x_space = max_width.saturating_sub(result_width) / 2;
    let y_space = max_height.saturating_sub(result_height) / 2;

    draw_text(
        text,
        pos_x + x_space as f32,
        pos_y + y_space as f32,
        current_text_size,
        meta,
    );
}

pub fn draw_text_scrolling_with_width(
    text: &str,
    x: f32,
    y: f32,
    script_size: f32,
    box_w: f32,
    global_frame: u64,
    meta: &mut RasterizerMeta,
) {
    draw_text_scrolling_with_width_internal(
        text,
        x,
        y,
        script_size,
        box_w,
        global_frame,
        true,
        None,
        meta,
    );
}

fn draw_text_scrolling_with_width_internal(
    text: &str,
    x: f32,
    y: f32,
    script_size: f32,
    box_w: f32,
    global_frame: u64,
    left_align: bool, // this must be set to true for scrolling (alignment does not make sense anyway with scrolling)
    debouncer: Option<&mut FontPositionDebouncer>,
    meta: &mut RasterizerMeta,
) {
    // calculate bounding box for moving text
    let mut left_bound_box = 0isize.max(x.floor() as isize);
    let mut right_bound_box = meta.texture_width.min((x.floor() + box_w) as usize);
    if !left_align {
        left_bound_box = 0isize.max((x.floor() - box_w) as isize);
        right_bound_box = meta.texture_width.min(x.floor() as usize);
    }

    layout_text(text, Some(x), Some(y), script_size, meta);
    let (glyphs, text_width, _) = text_meta_data(x, &meta.font_layout);

    let offset: isize = if left_align {
        // only on left aligned text, we possibly can have scrolling text (there is a box, technically nothing is aligned here)
        let amount_to_scroll = 0i64.max((text_width as i64) - box_w.ceil() as i64) as u64;

        if amount_to_scroll == 0 {
            0
        } else {
            // dynamically calculate the scroll amount
            let nr_frames_deadzones =
                (meta.server_imposed_settings.scroll_text_deadzones_nr_ms as u64 * 1000000)
                    / FRAME_TIME_NS;
            const PIXEL_PER_SEC_DEFAULT: u64 = 60;
            let nr_frames_scrolling = (amount_to_scroll
                * meta.server_imposed_settings.scroll_text_speed as u64
                * 1000000000)
                / FRAME_TIME_NS
                / 100
                / PIXEL_PER_SEC_DEFAULT;

            let progress = global_frame % (nr_frames_deadzones * 2 + nr_frames_scrolling);
            if progress < nr_frames_deadzones {
                0
            } else {
                // between 0 and nr_frames_scrolling, depending on scrolling progress
                let scroll_anim_progress_frame =
                    0i64.max(progress as i64 - nr_frames_deadzones as i64)
                        .min(nr_frames_scrolling as i64) as u64;

                if scroll_anim_progress_frame == nr_frames_scrolling {
                    amount_to_scroll as isize
                } else {
                    // interpolate
                    (scroll_anim_progress_frame as f32 * amount_to_scroll as f32
                        / nr_frames_scrolling as f32)
                        .round() as isize
                }
            }
        }
    } else {
        // right align, no scrolling for right align (does not make sense together)
        let glyh_width = widest_glyph(glyphs);

        match debouncer {
            Some(debouncer) => {
                let compare_from_debouncer = debouncer.add_width_and_process(text_width, text);

                if compare_from_debouncer >= text_width
                    && text_width + glyh_width / 2 > compare_from_debouncer
                {
                    // in range, take value from debouncer
                    compare_from_debouncer as isize
                } else {
                    // reset debouncer, as the value just jumped bigtime
                    debouncer.reset();
                    text_width as isize
                }
            }
            None => text_width as isize,
        }
    };

    // iterate over glyphs to draw them
    for glyph in glyphs {
        // Rasterize the glyph at the specified font size
        let (metrics, bitmap) = meta.font.rasterize(glyph.parent, script_size);

        // Get the target pixel position
        let x_cursor = glyph.x as isize;
        let y_cursor = glyph.y as isize;

        for y in 0..metrics.height {
            let ty = y_cursor + y as isize;
            for x in 0..metrics.width {
                let px = bitmap[y * metrics.width + x];
                let tx = x_cursor + x as isize - offset;

                if tx >= left_bound_box
                    && ty >= 0
                    && (tx as usize) < right_bound_box
                    && (ty as usize) < meta.texture_height
                {
                    let i = ((ty as usize) * meta.texture_width + (tx as usize)) * 4;
                    if i + 3 < meta.frame.len() {
                        blend_pixel(&mut meta.frame[i..], [255, 255, 255, px]);
                    }
                }
            }
        }
    }
}

fn rightmost_x(glyphs: &Vec<GlyphPosition>) -> f32 {
    glyphs
        .iter()
        .map(|g| g.x + g.width as f32)
        .fold(0.0, f32::max)
}

fn widest_glyph(glyphs: &Vec<GlyphPosition>) -> usize {
    glyphs.iter().map(|g| g.width).fold(0, usize::max)
}

fn layout_text<'a>(
    text: &'a str,
    x: Option<f32>,
    y: Option<f32>,
    script_size: f32,
    meta: &'a mut RasterizerMeta,
) {
    // filter for renderable glyphs
    let filtered: String = text.replace('\n', "\r").replace("\r\r", "\r");

    // typeset text to glyphs
    meta.font_layout.reset(&LayoutSettings {
        x: x.unwrap_or(0.0),
        y: y.unwrap_or(0.0),
        max_width: None, // no line wrap
        ..LayoutSettings::default()
    });
    meta.font_layout
        .append(&[meta.font], &TextStyle::new(&filtered, script_size, 0));
}

fn text_meta_data<'a>(x: f32, font_layout: &'a Layout) -> (&'a Vec<GlyphPosition>, usize, usize) {
    let glyphs: &Vec<GlyphPosition> = font_layout.glyphs();
    let text_width = (0f32.max(rightmost_x(glyphs) - x)).ceil() as usize;
    let text_height = font_layout.height().ceil() as usize;

    return (glyphs, text_width, text_height);
}

pub fn draw_text(text: &str, x: f32, y: f32, script_size: f32, meta: &mut RasterizerMeta) {
    // if the box is as wide as possible, there will be no scrolling
    // no animation necessary for static text, therefore frame = 0
    draw_text_scrolling_with_width(text, x, y, script_size, f32::MAX, 0, meta);
}

pub fn draw_text_right_aligned(
    text: &str,
    x: f32,
    y: f32,
    script_size: f32,
    debouncer: Option<&mut FontPositionDebouncer>,
    meta: &mut RasterizerMeta,
) {
    // if the box is as wide as possible, there will be no scrolling
    // no animation necessary for static text, therefore frame = 0
    draw_text_scrolling_with_width_internal(
        text,
        x,
        y,
        script_size,
        f32::MAX,
        0,
        false,
        debouncer,
        meta,
    );
}

pub struct FontPositionDebouncer {
    data: Ringbuffer<usize>,
    last_added_text: Option<String>,
}
impl FontPositionDebouncer {
    pub fn new_for_number_debouncing() -> Self {
        Self {
            data: Ringbuffer::new_with_capacity(10),
            last_added_text: None,
        }
    }

    fn add_width_and_process(&mut self, width: usize, text: &str) -> usize {
        if let Some(last_added_text) = &self.last_added_text {
            if last_added_text != text {
                self.data.push(width);
                self.last_added_text = Some(String::from(text));
            }
        } else {
            self.data.push(width);
            self.last_added_text = Some(String::from(text));
        }

        match self.data.get_max() {
            Some(a) => a,
            None => width,
        }
    }

    fn reset(&mut self) {
        self.last_added_text = None;
        self.data.empty();
    }
}

struct Ringbuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}
impl<T> Ringbuffer<T>
where
    T: Clone + Ord,
{
    pub fn new_with_capacity(cap: usize) -> Self {
        Self {
            capacity: cap,
            data: VecDeque::with_capacity(cap),
        }
    }

    pub fn push(&mut self, elem: T) {
        self.data.push_front(elem);
        if self.data.len() > self.capacity {
            self.data.pop_back();
        }
    }

    pub fn get_max(&self) -> Option<T> {
        self.data.iter().max().cloned()
    }

    pub fn empty(&mut self) {
        self.data.clear();
    }
}

fn blend_pixel(dst: &mut [u8], src: [u8; 4]) {
    dst[3] = 255; // fully opaque sub-surface. This is an assumption, as there is never anything below the window
    let src_a = src[3];

    if src_a == 0 {
        return;
    }
    if src_a == 255 {
        dst[0] = src[0];
        dst[1] = src[1];
        dst[2] = src[2];
        return;
    }

    dst[0] =
        ((((255 - src_a) as u32) * (dst[0] as u32) + (src_a as u32) * (src[0] as u32)) / 255) as u8;
    dst[1] =
        ((((255 - src_a) as u32) * (dst[1] as u32) + (src_a as u32) * (src[1] as u32)) / 255) as u8;
    dst[2] =
        ((((255 - src_a) as u32) * (dst[2] as u32) + (src_a as u32) * (src[2] as u32)) / 255) as u8;
}

fn draw_image_internal(
    x: u32,
    y: u32,
    img: &ImageMeta,
    opacity_override: Option<u8>,
    meta: &mut RasterizerMeta,
) {
    let buffer = img.get_image_buffer();

    if (x as usize) < meta.texture_width && (y as usize) < meta.texture_height {
        let width_to_draw_to = std::cmp::min(meta.texture_width, (x + img.width) as usize);
        let height_to_draw_to = std::cmp::min(meta.texture_height, (y + img.height) as usize);
        let x = x as usize;
        let y = y as usize;

        let mut x_cursor: u32;
        let mut y_cursor: u32 = 0;
        for ty in y..height_to_draw_to {
            let line_offset = ty * meta.texture_width;
            x_cursor = 0;
            for tx in x..width_to_draw_to {
                let index = (line_offset + tx) * 4;
                let px = buffer.get_pixel(x_cursor, y_cursor);

                // compile should optimize this generalization out, hopefully.
                if let Some(opacity_override) = opacity_override {
                    blend_pixel(
                        &mut meta.frame[index..],
                        [px[0], px[1], px[2], opacity_override],
                    );
                } else {
                    blend_pixel(&mut meta.frame[index..], [px[0], px[1], px[2], px[3]]);
                }

                x_cursor += 1;
            }
            y_cursor += 1;
        }
    }
}

pub fn draw_image(x: u32, y: u32, img: &ImageMeta, meta: &mut RasterizerMeta) {
    draw_image_internal(x, y, img, None, meta);
}

pub fn draw_image_at_opacity(
    x: u32,
    y: u32,
    img: &ImageMeta,
    opacity: u8,
    meta: &mut RasterizerMeta,
) {
    draw_image_internal(x, y, img, Some(opacity), meta);
}

pub type Color = [u8; 3];
pub const JTA_COLOR: Color = [46, 46, 46];

pub fn fill_with_color(color: Color, meta: &mut RasterizerMeta) {
    let [r, g, b] = color;
    let pixel = [r, g, b, 255u8];
    meta.frame
        .chunks_mut(4)
        .for_each(|px| px.copy_from_slice(&pixel));
}

pub fn clear(meta: &mut RasterizerMeta) {
    meta.frame.fill(0); // clear the window
}
