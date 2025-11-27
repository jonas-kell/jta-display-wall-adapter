use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::fs::File;
use std::thread;

pub fn play_audio_background() {
    // Spawn a background thread so this function returns immediately
    thread::spawn(|| {
        error!("playing audio");

        // Open the default audio output stream
        let stream_handle =
            OutputStreamBuilder::open_default_stream().expect("open default audio stream");

        // Create a Sink connected to the stream’s mixer
        let sink = Sink::connect_new(&stream_handle.mixer());

        // Load your audio file
        let file = File::open("assets/Sounds/example.ogg").expect("failed to open audio file");
        let source = Decoder::try_from(file).expect("failed to decode audio");

        // Add the audio source to the sink
        sink.append(source);

        // Block this background thread until audio finishes
        sink.sleep_until_end();

        error!("audio finished");

        // Thread exits here, stream + sink drop naturally
    });
}
