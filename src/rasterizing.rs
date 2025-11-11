use fontdue::{
    layout::{Layout, LayoutSettings, TextStyle},
    Font,
};
use image::{imageops::FilterType, DynamicImage, ImageReader};
use image::{GenericImageView, Rgba};
use std::{io::Cursor, sync::Arc};

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

pub struct ImageMeta {
    pub width: u32,
    pub height: u32,
    img: Arc<DynamicImage>,
}
impl ImageMeta {
    pub fn from_image_bytes(bytes: &[u8]) -> Result<ImageMeta, String> {
        match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
            Ok(rd) => match rd.decode() {
                Ok(img) => {
                    let (width, height) = img.dimensions();

                    Ok(ImageMeta {
                        img: Arc::new(img),
                        height,
                        width,
                    })
                }
                Err(e) => Err(e.to_string()),
            },
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn get_pixel_at(&self, x: u32, y: u32) -> ColorWithAlpha {
        let intermediate = self.img.to_rgba8();
        let pixel = intermediate.get_pixel(x, y);
        return [pixel[0], pixel[1], pixel[2], pixel[3]];
    }

    pub fn get_rescaled(&self, new_width: u32, new_height: u32) -> Self {
        if new_width == self.width && new_height == self.height {
            return ImageMeta {
                width: self.width,
                height: self.height,
                img: self.img.clone(),
            }; // do not scale
        }

        let new_image = Arc::new(self.img.resize_exact(
            new_width,
            new_height,
            FilterType::Lanczos3,
        ));

        return Self {
            height: new_height,
            width: new_width,
            img: new_image,
        };
    }
}

fn blend_pixel(dst: &mut [u8], src: &Rgba<u8>) {
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

pub fn draw_image(
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    img: &ImageMeta,
    meta: &mut RasterizerMeta,
) {
    let image_rescaled = img.get_rescaled(width, height);

    let buffer = image_rescaled.img.to_rgba8(); // bit ugly acces from outside of class

    if (x as usize) < meta.texture_width && (y as usize) < meta.texture_height {
        let width_to_draw_to =
            std::cmp::min(meta.texture_width, (x + image_rescaled.width) as usize);
        let height_to_draw_to =
            std::cmp::min(meta.texture_height, (y + image_rescaled.height) as usize);
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

                blend_pixel(&mut meta.frame[index..], px);

                x_cursor += 1;
            }
            y_cursor += 1;
        }
    }
}

pub type Color = [u8; 3];
pub type ColorWithAlpha = [u8; 4];
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
