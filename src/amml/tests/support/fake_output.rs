use alaydriem_durability_ping::CueOutput;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct FakeState {
    pub opens: usize,
    pub plays: usize,
    pub closes: usize,
    pub fail_open: bool,
    pub fail_next_plays: usize,
    pub play_delay: Duration,
    pub last_len: usize,
    pub samples: Vec<Vec<f32>>,
}

// records what the audio thread asks of it; the test holds the other end of the Arc
#[derive(Clone, Default)]
pub struct FakeOutput {
    pub state: Arc<Mutex<FakeState>>,
}

impl FakeOutput {
    pub fn read<T>(&self, f: impl FnOnce(&FakeState) -> T) -> T {
        f(&self.state.lock().unwrap())
    }

    pub fn change(&self, f: impl FnOnce(&mut FakeState)) {
        f(&mut self.state.lock().unwrap());
    }

    // wait up to 2 s for a condition on the counters
    pub fn wait_for(&self, condition: impl Fn(&FakeState) -> bool) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(2) {
            if self.read(&condition) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        self.read(&condition)
    }
}

impl CueOutput for FakeOutput {
    fn open(&mut self) -> Result<(u32, u16), String> {
        let mut state = self.state.lock().unwrap();
        state.opens += 1;
        if state.fail_open {
            return Err("no device".to_string());
        }
        Ok((48_000, 2))
    }

    fn play(&mut self, samples: Vec<f32>) -> Result<(), String> {
        let delay = self.state.lock().unwrap().play_delay;
        std::thread::sleep(delay);
        let mut state = self.state.lock().unwrap();
        state.plays += 1;
        state.last_len = samples.len();
        state.samples.push(samples);
        if state.fail_next_plays > 0 {
            state.fail_next_plays -= 1;
            return Err("device lost".to_string());
        }
        Ok(())
    }

    fn close(&mut self) {
        self.state.lock().unwrap().closes += 1;
    }
}
