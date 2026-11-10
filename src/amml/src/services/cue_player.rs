use crate::traits::CueOutput;
use durability_ping_core::audio::Cue;
use std::cell::Cell;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::thread::JoinHandle;

// plays cues on its own thread so the game thread never waits on the sound device
pub struct CuePlayer {
    sender: Option<Sender<(Cue, u8)>>,
    thread: Option<JoinHandle<()>>,
    warnings: Arc<AtomicUsize>,
}

impl CuePlayer {
    // start the audio thread; the output opens on the first cue
    pub fn start<O: CueOutput>(output: O) -> Self {
        let (sender, receiver) = mpsc::channel::<(Cue, u8)>();
        let warnings = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&warnings);
        let spawned = std::thread::Builder::new().name("durability-ping-audio".to_string()).spawn(move || {
            let mut output = output;
            let mut format: Option<(u32, u16)> = None;
            // one warning per run of failures; a success resets it
            let warned = Cell::new(false);
            let warn = |step: &str, error: String| {
                if !warned.replace(true) {
                    counter.fetch_add(1, Ordering::SeqCst);
                    curia::warn!("cue could not be played", { step: step.to_string(), error: error });
                }
            };
            while let Ok((cue, volume)) = receiver.recv() {
                let (rate, channels) = match format {
                    Some(known) => known,
                    None => match output.open() {
                        Ok(opened) => {
                            format = Some(opened);
                            opened
                        }
                        Err(error) => {
                            warn("open", error);
                            continue;
                        }
                    },
                };
                match output.play(cue.samples(rate, channels, volume)) {
                    Ok(()) => warned.set(false),
                    Err(error) => {
                        warn("play", error);
                        output.close();
                        format = None;
                    }
                }
            }
            output.close();
        });
        let thread = match spawned {
            Ok(handle) => Some(handle),
            Err(error) => {
                curia::warn!("audio thread did not start", { error: error.to_string() });
                None
            }
        };
        Self { sender: Some(sender), thread, warnings }
    }

    // queue a cue; never waits
    pub fn play(&self, cue: Cue, volume: u8) {
        if let Some(sender) = &self.sender {
            // an error means the thread is gone; a cue is a courtesy
            let _ = sender.send((cue, volume));
        }
    }

    // how many warnings the audio thread logged
    pub fn warnings(&self) -> usize {
        self.warnings.load(Ordering::SeqCst)
    }

    // end the thread and wait for it
    pub fn stop(self) {
        drop(self);
    }
}

impl Drop for CuePlayer {
    fn drop(&mut self) {
        self.sender = None;
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
