use image::{imageops::FilterType, DynamicImage, ImageReader};
use image::{GenericImageView, Rgba};
use include_dir::Dir;
use std::{collections::HashMap, io::Cursor, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
pub struct ImageMeta {
    id: Uuid,
    pub width: u32,
    pub height: u32,
    img: Arc<DynamicImage>,
}
impl ImageMeta {
    pub fn from_dynamic_image(dynamic: DynamicImage) -> Self {
        ImageMeta {
            id: Uuid::new_v4(),
            width: dynamic.width(),
            height: dynamic.height(),
            img: Arc::new(dynamic),
        }
    }

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
