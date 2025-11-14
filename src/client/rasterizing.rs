use fontdue::{
    layout::{Layout, LayoutSettings, TextStyle},
    Font,
};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, ImageReader};
use image::{GenericImageView, Rgba};
use include_dir::Dir;
use std::{collections::HashMap, io::Cursor, sync::Arc};
use uuid::Uuid;

pub struct RasterizerMeta<'a> {
    pub font: &'a Font,
    pub font_layout: &'a mut Layout,
    pub frame: &'a mut [u8],
    pub texture_width: usize,
    pub texture_height: usize,
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

        Ok(ImageMeta {
            id: Uuid::new_v4(),
            width: self.texture_width as u32,
            height: self.texture_height as u32,
            img: Arc::new(dynamic),
        })
    }
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
                        blend_pixel(&mut meta.frame[i..], [255, 255, 255, px]);
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ImageMeta {
    id: Uuid,
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
                        id: Uuid::new_v4(),
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

    pub fn get_image_buffer(&self) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        return self.img.to_rgba8();
    }

    pub fn get_rescaled(&self, new_width: u32, new_height: u32) -> Self {
        if new_width == self.width && new_height == self.height {
            return ImageMeta {
                id: Uuid::new_v4(),
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
            id: Uuid::new_v4(),
            height: new_height,
            width: new_width,
            img: new_image,
        };
    }
}

#[derive(Clone)]
pub struct Animation {
    real_frames_per_frame: u32,
    data: Vec<ImageMeta>,
}
impl Animation {
    pub fn from_dir(dir: Dir, real_frames_per_frame: u32) -> Result<Animation, String> {
        if real_frames_per_frame == 0 {
            return Err(String::from(
                "Animation can have 0 for real_frames_per_frame",
            ));
        }

        let mut data_aggregator: Vec<(u32, ImageMeta)> = Vec::new();

        let mut animation_width: Option<u32> = None;
        let mut animation_height: Option<u32> = None;

        for entry in dir.entries() {
            match entry.as_file() {
                None => return Err(String::from("Folder with frames must only contain files")),
                Some(file) => {
                    let name = match file.path().file_name() {
                        None => return Err(String::from("The file name of one file in the folder with the frames could not be determined")),
                        Some(name) => match name.to_str() {
                            Some(str) => str,
                            None => return Err(String::from("The file name of one file in the folder with the frames could not be converted"))
                        }
                    };

                    let index = match Animation::parse_prefix_u32(name) {
                        None => {
                            return Err(format!(
                            "The file with the name \"{}\" has no filename that is only an index",
                            name
                        ))
                        }
                        Some(a) => a,
                    };

                    let image = match ImageMeta::from_image_bytes(file.contents()) {
                        Err(e) => {
                            return Err(format!(
                                "One frame of the file \"{}\" could not be parsed as an image: {}",
                                name, e
                            ))
                        }
                        Ok(meta) => meta,
                    };

                    if let Some(animation_width_val) = animation_width {
                        if animation_width_val != image.width {
                            return Err(format!(
                                "Not all frames of the animation have the same width",
                            ));
                        }
                    } else {
                        animation_width = Some(image.width)
                    }
                    if let Some(animation_height_val) = animation_height {
                        if animation_height_val != image.height {
                            return Err(format!(
                                "Not all frames of the animation have the same height",
                            ));
                        }
                    } else {
                        animation_height = Some(image.height)
                    }

                    data_aggregator.push((index, image));
                }
            }
        }

        data_aggregator.sort_by(|(ind_a, _), (ind_b, _)| ind_a.cmp(ind_b));
        let frames: Vec<ImageMeta> = data_aggregator.into_iter().map(|(_, img)| img).collect();

        if frames.len() == 0 {
            return Err(format!("Animation contains no frames",));
        }

        return Ok(Animation {
            real_frames_per_frame,
            data: frames,
        });
    }

    fn parse_prefix_u32(s: &str) -> Option<u32> {
        let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
        if end == 0 {
            return None;
        }
        s[..end].parse().ok()
    }

    pub fn cache_animation_for_size(
        &self,
        width: u32,
        height: u32,
        rescaler: &mut CachedImageScaler,
    ) {
        for image in &self.data {
            rescaler.purge_from_cache(image);
            rescaler.scale_cached(image, width, height);
        }
    }
}

pub struct AnimationPlayer {
    has_started_on_global_frame: u64,
    animation: Animation,
    loop_animation: bool,
}

impl AnimationPlayer {
    pub fn new(animation: &Animation, global_frame: u64, loop_animation: bool) -> AnimationPlayer {
        return AnimationPlayer {
            has_started_on_global_frame: global_frame,
            animation: animation.clone(), // that should be an acceptable cost for now
            loop_animation,
        };
    }

    pub fn get_current_frame(
        &self,
        width: u32,
        height: u32,
        global_frame: u64,
        rescaler: &mut CachedImageScaler,
    ) -> Option<ImageMeta> {
        let number_frames = self.animation.data.len() as u64;
        let real_frames_since_start =
            global_frame - std::cmp::min(global_frame, self.has_started_on_global_frame);
        let animation_frames_since_start =
            real_frames_since_start / self.animation.real_frames_per_frame as u64;

        if animation_frames_since_start > number_frames && !self.loop_animation {
            // animation is over
            return None;
        }

        let index = animation_frames_since_start % number_frames;

        let image_to_use = match self.animation.data.get(index as usize) {
            None => return None,
            Some(img) => img,
        };

        return Some(rescaler.scale_cached(image_to_use, width, height));
    }
}

pub struct CachedImageScaler {
    data: HashMap<(Uuid, u32, u32), ImageMeta>,
}
impl CachedImageScaler {
    pub fn new() -> Self {
        return Self {
            data: HashMap::new(),
        };
    }

    pub fn scale_cached(&mut self, img: &ImageMeta, new_width: u32, new_height: u32) -> ImageMeta {
        match self.data.get(&(img.id, new_width, new_height)) {
            Some(met) => {
                return met.clone();
            }
            None => {
                let rescaled = img.get_rescaled(new_width, new_height);

                self.data
                    .insert((img.id, new_width, new_height), rescaled.clone());

                return rescaled;
            }
        }
    }

    // TODO this is not optimally efficient. Could use a hash map that only goes for the uuid and that contains hash map with the dimensions.
    pub fn purge_from_cache(&mut self, img: &ImageMeta) {
        let filtered: Vec<_>;
        {
            filtered = self
                .data
                .iter()
                .filter_map(|((id, w, h), _)| {
                    if *id == img.id {
                        Some((id.clone(), w.clone(), h.clone()))
                    } else {
                        None
                    }
                })
                .collect();
        }

        for key in filtered {
            let _ = self.data.remove_entry(&key);
        }
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
    let buffer = img.img.to_rgba8(); // bit ugly acces from outside of class

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
