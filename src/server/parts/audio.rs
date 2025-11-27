use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;
use std::thread;

#[derive(Eq, Hash, PartialEq, Debug)]
pub enum Sound {
    Beep1,
    Beep2,
    Beep3,
}

pub struct AudioPlayer {
    out: OutputStream,
    sound_map: HashMap<Sound, &'static [u8]>,
}
impl AudioPlayer {
    pub fn new() -> Result<Self, String> {
        let stream_handle: OutputStream =
            OutputStreamBuilder::open_default_stream().map_err(|e| e.to_string())?;

        let mut sound_map: HashMap<Sound, &'static [u8]> = HashMap::new();

        let beep_1 = include_bytes!("../../../assets/Boop/boop-1.ogg");
        sound_map.insert(Sound::Beep1, beep_1);
        Self::decoded_source(beep_1)?; // check on initial run that this works
        let beep_2 = include_bytes!("../../../assets/Boop/boop-2.ogg");
        sound_map.insert(Sound::Beep2, beep_2);
        Self::decoded_source(beep_2)?; // check on initial run that this works
        let beep_3 = include_bytes!("../../../assets/Boop/boop-3.ogg");
        sound_map.insert(Sound::Beep3, beep_3);
        Self::decoded_source(beep_3)?; // check on initial run that this works

        Ok(Self {
            out: stream_handle,
            sound_map,
        })
    }

    fn decoded_source(bytes: &'static [u8]) -> Result<Box<dyn Source + Send + Sync>, String> {
        let cursor = Cursor::new(&bytes[..]);
        let decoder = Decoder::new(cursor).map_err(|e| e.to_string())?;
        Ok(Box::new(decoder))
    }

    fn get_source(&self, sound: &Sound) -> Option<Box<dyn Source + Send + Sync>> {
        let bytes = match self.sound_map.get(&sound) {
            Some(bytes) => bytes,
            None => return None,
        };

        Self::decoded_source(bytes).ok()
    }

    pub fn play_audio_background(&self, sound: Sound) {
        // Spawn a background thread so this function returns immediately
        let source = self.get_source(&sound);
        let sink = Sink::connect_new(&self.out.mixer());

        thread::spawn(move || {
            trace!("Playing audio");

            if let Some(source) = source {
                sink.append(source);
                sink.sleep_until_end();
                trace!("Audio finished");
            } else {
                error!("Sound: {:?} not loaded", sound);
            }
        });
    }
}
