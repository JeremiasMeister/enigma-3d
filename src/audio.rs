use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

#[derive(Clone)]
pub struct AudioClip {
    pub name: String,
    data: Arc<[u8]>,
}

impl AudioClip {

    pub fn load(file_path: &str, name: Option<&str>) -> Self{
        let audio_data = std::fs::read(file_path).expect("Failed to load Audio file");
        let clip_name = match name {
            Some(n) => n.to_string(),
            None => {
                let path = std::path::Path::new(file_path);
                let name = match path.file_name() {
                    Some(n2) => {
                        n2.to_str().expect("Failed to get String").to_string()
                    },
                    None => panic!("Cannot get clip name from Path")

                };
                name
            }
        };

        Self{
            name: clip_name,
            data: Arc::from(audio_data)
        }
    }

    pub fn from_resource(data: &[u8], name: &str, ) -> Self {
        Self {
            name: name.to_string(),
            data: Arc::from(data.to_vec()),
        }
    }

    /// Internal helper to create a `rodio` `Source` from the clip's data.
    /// This is what allows the audio data to be played.
    /// The visibility is `pub(crate)` so only code within this crate (i.e., the AudioEngine)
    /// can use it.
    pub(crate) fn create_source(&self) -> Decoder<Cursor<Arc<[u8]>>> {
        // This now works because `Arc<[u8]>` implements `AsRef<[u8]>`.
        let cursor = Cursor::new(self.data.clone()); // `clone()` is a cheap reference count bump.
        Decoder::new(cursor).expect("Failed to decode audio data")
    }
}


pub struct AudioEngine {
    // The `OutputStream` needs to be kept alive for audio to play,
    // so we store it here. The underscore indicates we don't use it directly.
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    // Stores the actively playing sound instances, identified by the same name
    // as their corresponding AudioClip.
    active_sinks: HashMap<String, Sink>,
}

impl AudioEngine {
    pub fn new() -> Self {
        let (_stream, stream_handle) =
            OutputStream::try_default().expect("Failed to get default audio output stream");
        Self {
            _stream,
            stream_handle,
            active_sinks: HashMap::new(),
        }
    }

    pub fn play_clip_once(&mut self, name: &str, clips: &HashMap<String, AudioClip>) {
        // Find the requested audio clip data.
        if let Some(clip) = clips.get(name) {
            // A new sink is created for each sound playback instance.
            // `try_new` will not fail as long as the stream_handle is valid.
            let sink = Sink::try_new(&self.stream_handle).unwrap();
            sink.append(clip.create_source());

            // By inserting the new sink, we automatically drop and stop any
            // previously playing sink with the same name.
            self.active_sinks.insert(name.to_string(), sink);
        } else {
            // It's good practice to log when a requested sound doesn't exist.
            eprintln!("Audio warning: Tried to play non-existent clip '{}'", name);
        }
    }

    pub fn play_clip_loop(&mut self, name: &str, clips: &HashMap<String, AudioClip>) {
        if let Some(clip) = clips.get(name) {
            let sink = Sink::try_new(&self.stream_handle).unwrap();
            // The `.repeat_infinite()` adapter turns the source into a looping one.
            sink.append(clip.create_source().repeat_infinite());
            self.active_sinks.insert(name.to_string(), sink);
        } else {
            eprintln!("Audio warning: Tried to play non-existent clip '{}'", name);
        }
    }

    pub fn stop_clip(&mut self, name: &str) {
        self.active_sinks.remove(name);
    }

    pub fn toggle_pause_clip(&self, name: &str) {
        if let Some(sink) = self.active_sinks.get(name) {
            if sink.is_paused() {
                sink.play();
            } else {
                sink.pause();
            }
        }
    }

    pub fn set_clip_volume(&self, name: &str, volume: f32) {
        if let Some(sink) = self.active_sinks.get(name) {
            sink.set_volume(volume);
        }
    }
}
