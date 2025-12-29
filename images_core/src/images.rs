use bincode::serde::{borrow_decode_from_slice, encode_to_vec};
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::{imageops::FilterType, DynamicImage, ImageReader};
use image::{ColorType, GenericImageView, ImageEncoder, Rgba};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor, sync::Arc};
use uuid::Uuid;

const CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
> = bincode::config::standard()
    .with_little_endian()
    .with_fixed_int_encoding();

#[derive(Serialize, Deserialize)]
struct ImageMetaSerealizer<'a> {
    id: Uuid,
    width: u32,
    height: u32,
    #[serde(borrow)]
    img: &'a [u8],
}

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

    /// Can fail, as it is used as a compile-time only method
    fn to_bytes(&self) -> Vec<u8> {
        let data = self.img.to_rgba8().into_raw();

        let mut bytes: Vec<u8> = Vec::new();
        let encoder = PngEncoder::new_with_quality(
            Cursor::new(&mut bytes),
            CompressionType::Best,
            PngFilterType::Adaptive,
        );

        encoder
            .write_image(&data, self.width, self.height, ColorType::Rgba8.into())
            .expect("PNG encoding failed");

        let ser = ImageMetaSerealizer {
            id: self.id,
            width: self.width,
            height: self.height,
            img: &bytes,
        };
        return encode_to_vec(&ser, CONFIG).unwrap();
    }

    /// Can fail, as it is used as a compile-time only method
    fn from_bytes<'a>(data: &'a [u8]) -> Self {
        let (dec, _) =
            borrow_decode_from_slice::<'a, ImageMetaSerealizer<'a>, _>(data, CONFIG).unwrap();

        // this actually copies still
        // TODO we could do a variant for global data, that always indexes into slices (somehow)
        let mut reader = ImageReader::new(Cursor::new(dec.img));
        reader.set_format(image::ImageFormat::Png);
        let image = reader.decode().unwrap();

        return Self {
            id: dec.id,
            width: dec.width,
            height: dec.height,
            img: Arc::new(image),
        };
    }
}

#[derive(Serialize, Deserialize)]
struct AnimationSerealizer<'a> {
    real_frames_per_frame: u32,
    #[serde(borrow)]
    data: Vec<&'a [u8]>,
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
        purge: bool,
    ) {
        for image in &self.data {
            if purge {
                rescaler.purge_from_cache(image);
            }
            rescaler.scale_cached(image, width, height);
        }
    }

    /// Can fail, as it is used as a compile-time only method
    fn to_bytes(&self) -> Vec<u8> {
        let image_data: Vec<Vec<u8>> = self.data.iter().map(|a| a.to_bytes()).collect();

        let ser = AnimationSerealizer {
            real_frames_per_frame: self.real_frames_per_frame,
            data: image_data.iter().map(|a| a.as_slice()).collect(),
        };
        let bytes = encode_to_vec(&ser, CONFIG).unwrap();
        return bytes;
    }

    /// Can fail, as it is used as a compile-time only method
    fn from_bytes<'a>(data: &'a [u8]) -> Self {
        let (dec, _) =
            borrow_decode_from_slice::<'a, AnimationSerealizer<'a>, _>(data, CONFIG).unwrap();

        return Self {
            real_frames_per_frame: dec.real_frames_per_frame,
            data: dec
                .data
                .into_iter()
                .map(|a| ImageMeta::from_bytes(&a))
                .collect(),
        };
    }
}

pub struct AnimationPlayer {
    has_started_on_global_frame: Option<u64>,
    animation: Animation,
    loop_animation: bool,
}

impl AnimationPlayer {
    pub fn new(animation: &Animation, loop_animation: bool) -> AnimationPlayer {
        return AnimationPlayer {
            has_started_on_global_frame: None,
            animation: animation.clone(), // that should be an acceptable cost for now
            loop_animation,
        };
    }

    pub fn get_current_frame(
        &mut self,
        width: u32,
        height: u32,
        global_frame: u64,
        rescaler: &mut CachedImageScaler,
    ) -> Option<ImageMeta> {
        if self.has_started_on_global_frame.is_none() {
            self.has_started_on_global_frame = Some(global_frame)
        }

        if let Some(has_started_on_global_frame) = self.has_started_on_global_frame {
            let number_frames = self.animation.data.len() as u64;
            let real_frames_since_start =
                global_frame - std::cmp::min(global_frame, has_started_on_global_frame);
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
        } else {
            return None;
        }
    }
}

#[derive(Serialize, Deserialize)]
struct CachedImageScalerSerealizer<'a> {
    #[serde(borrow)]
    data: Vec<((Uuid, u32, u32), &'a [u8])>,
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

    /// Can fail, as it is used as a compile-time only method
    fn to_bytes(&self) -> Vec<u8> {
        let images_data: Vec<((Uuid, u32, u32), Vec<u8>)> = self
            .data
            .iter()
            .map(|(k, v)| (k.clone(), v.to_bytes()))
            .collect();

        let ser = CachedImageScalerSerealizer {
            data: images_data
                .iter()
                .map(|(k, v)| (k.clone(), v.as_slice()))
                .collect(),
        };
        let bytes = encode_to_vec(&ser, CONFIG).unwrap();
        return bytes;
    }

    /// Can fail, as it is used as a compile-time only method
    fn from_bytes<'a>(data: &'a [u8]) -> Self {
        let (dec, _) =
            borrow_decode_from_slice::<'a, CachedImageScalerSerealizer<'a>, _>(data, CONFIG)
                .unwrap();

        return Self {
            data: dec
                .data
                .into_iter()
                .map(|(k, v)| (k, ImageMeta::from_bytes(v)))
                .collect(),
        };
    }
}

#[derive(Serialize, Deserialize)]
struct ImagesStorageSerealizer<'a> {
    #[serde(borrow)]
    pub jta_logo: &'a [u8],
    #[serde(borrow)]
    pub advertisement_images: Vec<&'a [u8]>,
    #[serde(borrow)]
    pub cached_rescaler: &'a [u8],
    #[serde(borrow)]
    pub fireworks_animation: &'a [u8],
}

pub struct ImagesStorage {
    pub jta_logo: ImageMeta,
    pub advertisement_images: Vec<ImageMeta>,
    pub cached_rescaler: CachedImageScaler,
    pub fireworks_animation: Animation,
}
impl ImagesStorage {
    pub fn new_with_compile_data(precache_for_sizes: &[(u32, u32)]) -> ImagesStorage {
        // include static files
        let jta_logo =
            ImageMeta::from_image_bytes(include_bytes!("./../../assets/JTA-Logo.png")).unwrap();
        let fireworks_animation = Animation::from_dir(
            include_dir!("$CARGO_MANIFEST_DIR/../assets/Fireworks/frames"),
            // std::cmp::min((TARGET_FPS as u32) / 30, 1),
            3, // is technically a 30 fps animation, but looks better like this
        )
        .unwrap();

        let mut scaler = CachedImageScaler::new();
        for (w, h) in precache_for_sizes {
            fireworks_animation.cache_animation_for_size(*w, *h, &mut scaler, false);
        }

        return ImagesStorage {
            jta_logo,
            cached_rescaler: scaler,
            advertisement_images: Vec::new(),
            fireworks_animation,
        };
    }

    /// Can fail, as it is used as a compile-time only method
    pub fn to_bytes(&self) -> Vec<u8> {
        let images_data: Vec<Vec<u8>> = self
            .advertisement_images
            .iter()
            .map(|a| a.to_bytes())
            .collect();

        let ser = ImagesStorageSerealizer {
            jta_logo: &self.jta_logo.to_bytes(),
            advertisement_images: images_data.iter().map(|a| a.as_slice()).collect(),
            cached_rescaler: &self.cached_rescaler.to_bytes(),
            fireworks_animation: &self.fireworks_animation.to_bytes(),
        };
        let bytes = encode_to_vec(&ser, CONFIG).unwrap();
        return bytes;
    }

    /// Can fail, as it is used as a compile-time only method
    pub fn from_bytes<'a>(data: &'a [u8]) -> Self {
        let (dec, _) =
            borrow_decode_from_slice::<'a, ImagesStorageSerealizer<'a>, _>(data, CONFIG).unwrap();

        return Self {
            jta_logo: ImageMeta::from_bytes(dec.jta_logo),
            advertisement_images: dec
                .advertisement_images
                .into_iter()
                .map(|a| ImageMeta::from_bytes(a))
                .collect(),
            cached_rescaler: CachedImageScaler::from_bytes(dec.cached_rescaler),
            fireworks_animation: Animation::from_bytes(dec.fireworks_animation),
        };
    }
}

#[derive(Serialize, Deserialize)]
struct IconsStorageSerealizer<'a> {
    #[serde(borrow)]
    pub round_icon: &'a [u8],
    #[serde(borrow)]
    pub cached_rescaler: &'a [u8],
}

pub struct IconsStorage {
    pub round_icon: ImageMeta,
    pub cached_rescaler: CachedImageScaler,
}
impl IconsStorage {
    pub fn new_with_compile_data(precache_for_sizes: &[(u32, u32)]) -> IconsStorage {
        // include static files
        let round_icon =
            ImageMeta::from_image_bytes(include_bytes!("./../../assets/Round-Icon.png")).unwrap();

        let mut scaler = CachedImageScaler::new();
        for (w, h) in precache_for_sizes {
            scaler.scale_cached(&round_icon, *w, *h);
        }

        return IconsStorage {
            round_icon,
            cached_rescaler: scaler,
        };
    }

    /// Can fail, as it is used as a compile-time only method
    pub fn to_bytes(&self) -> Vec<u8> {
        let ser = IconsStorageSerealizer {
            round_icon: &self.round_icon.to_bytes(),
            cached_rescaler: &self.cached_rescaler.to_bytes(),
        };
        let bytes = encode_to_vec(&ser, CONFIG).unwrap();
        return bytes;
    }

    /// Can fail, as it is used as a compile-time only method
    pub fn from_bytes<'a>(data: &'a [u8]) -> Self {
        let (dec, _) =
            borrow_decode_from_slice::<'a, IconsStorageSerealizer<'a>, _>(data, CONFIG).unwrap();

        return Self {
            round_icon: ImageMeta::from_bytes(dec.round_icon),
            cached_rescaler: CachedImageScaler::from_bytes(dec.cached_rescaler),
        };
    }
}
