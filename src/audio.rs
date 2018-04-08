//! Rodio-based audio system

use rodio::{default_endpoint, Endpoint, Sink, Decoder};
use std::{thread, sync::mpsc};
use FastHashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::Cursor;

/// Core audio context, maintains a handle to the background audio thread
pub struct AudioContext {
    sender: mpsc::Sender<AudioMsg>,
    thread_handle: thread::JoinHandle<()>,
}

/// An audio message 
pub struct AudioMsg {
    /// Which song to affect. Usually this is the file name, e.g. "hiteffect1.ogg"
    pub song: String,
    /// Which speaker to play the sound on
    /// TODO: does nothing yet
    pub speaker: Speaker,
    /// What to do with the song?
    pub action: AudioAction,
}

/// Which speaker to play the sound on
pub enum Speaker {
    /// Audio will be played on both speakers with the same volume
    Mono,
    /// Spatial audio
    Stereo { left: f32, right: f32 },
}

impl Default for Speaker {
    fn default() -> Self {
        Speaker::Mono
    }
}

pub enum AudioAction {
    /// Start a given song. Does nothing if the song is already playing. 
    /// do_loop controls if the song is looped.
    /// song is the actual song data, loaded from the file.
    Start { song_data: Vec<u8>, do_loop: bool }, 
    /// From 0.0 to 100.0 - adjusts the volume of the given song
    AdjustVolume(f32),
    /// Pauses the song. 
    Pause,
    /// Returns to playing the song, if previously paused
    Play,
    /// Fades out the song, with the value in milliseconds how long to fade out
    /// TODO: does nothing yet
    FadeOut(u32),
    /// Plays the current song until it ends, with an optional fadeout effect in milliseconds
    /// TODO: does nothing yet
    PlayUntilEnd { fade_out: Option<u32> },
}

/// Error marker that the audio data isn't yet loaded and needs to be loaded 
/// from the games main audio hashmap
#[derive(Debug, Copy, Clone)]
pub struct AudioDataNotLoaded;

/// audio.rs internal audio cache, for caching multiple rodio::Sinks and 
/// controlling the decoding of 
struct AudioCache {
    pub endpoint: Endpoint,
    pub active_songs: FastHashMap<String, AudioSink>,
}

struct AudioSink {
    /// The rodio::Sing
    sink: Sink,
    /// 1.0 = default volume
    volume: f32,
}

impl AudioCache {

    /// Creates a new audio cache
    pub fn new() -> Self {
        AudioCache {
            endpoint: default_endpoint().unwrap(),
            active_songs: FastHashMap::default(),
        }
    }

    /// Updates or inserts a new song, based on the audio message
    pub fn upsert(&mut self, msg: AudioMsg) -> Result<(), AudioDataNotLoaded> {
        use self::AudioAction::*;

        match self.active_songs.entry(msg.song) {
            Occupied(o) => {
                match msg.action {
                    Start { .. } => { },
                    AdjustVolume(vol) => {
                        let vol = vol / 100.0;
                        if o.get().volume != vol {
                            o.get().sink.set_volume(vol);
                        }
                    },
                    Pause => {
                        o.get().sink.pause();
                    },
                    Play => {
                        o.get().sink.play();
                    },
                    _ => {
                        #[cfg(debug_assertions)]
                        { println!("unimplemented audio message!"); }
                    }
                }
            },
            Vacant(v) => {
                match msg.action {
                    Start { song_data, do_loop } => {
                        // insert a new song
                        let sink = Sink::new(&self.endpoint);
                        let decoder = Decoder::new(Cursor::new(song_data)).unwrap();
                        sink.append(decoder);
                        v.insert(AudioSink { sink: sink, volume: 1.0 });
                    },
                    _ => {
                        return Err(AudioDataNotLoaded);
                    }
                }
            }
        }

        Ok(())
    }
}

impl AudioContext {

    /// Starts a thread, returns the context
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let thread_handle = thread::spawn(move || Self::music_loop(rx));
        Self {
            sender: tx,
            thread_handle: thread_handle,
        }
    }

    pub fn send_msg(&self, msg: AudioMsg) -> Result<(), mpsc::SendError<AudioMsg>> {
        self.sender.send(msg)
    }

    // music loop that runs on a background thread
    fn music_loop(rx: mpsc::Receiver<AudioMsg>) {
        let mut audio_cache = AudioCache::new();
        while let Ok(event) = rx.recv() {
            audio_cache.upsert(event).unwrap_or_else(|e| {
                #[cfg(debug_assertions)]
                { println!("error upserting song: {:?}!", e); }
            })
        }
    }
}

impl Drop for AudioContext {
    fn drop(&mut self) {
        self.thread_handle.join().unwrap();
    }
}