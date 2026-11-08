use super::tone::Tone;
use super::tone_spec::ToneSpec;

/// A short tone that reports a durability change the player may miss while mining.
///
/// `Warning` is two falling notes in a high register: a tool is nearly spent. `Blocked` is a
/// single low note: the action was stopped. The two differ in pitch and in note count, so
/// they are told apart by ear without looking at the screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cue {
    Warning,
    Blocked,
}

impl Cue {
    const PARTIALS: [(f32, f32); 2] = [(1.0, 1.0), (2.0, 0.25)];
    const DECAY_SECONDS: f32 = 0.05;
    const ATTACK_SECONDS: f32 = 0.004;
    const PEAK: f32 = 0.25;

    const WARNING: ToneSpec = ToneSpec {
        notes: &[(1567.98, 0.0), (1174.66, 0.07)],
        partials: &Cue::PARTIALS,
        decay_seconds: Cue::DECAY_SECONDS,
        attack_seconds: Cue::ATTACK_SECONDS,
        peak: Cue::PEAK,
        duration_seconds: 0.34,
    };

    const BLOCKED: ToneSpec = ToneSpec {
        notes: &[(220.0, 0.0)],
        partials: &Cue::PARTIALS,
        decay_seconds: 0.04,
        attack_seconds: Cue::ATTACK_SECONDS,
        peak: Cue::PEAK,
        duration_seconds: 0.22,
    };

    fn spec(&self) -> ToneSpec {
        match self {
            Cue::Warning => Cue::WARNING,
            Cue::Blocked => Cue::BLOCKED,
        }
    }

    pub fn duration_seconds(&self) -> f32 {
        self.spec().duration_seconds
    }

    /// Interleaved samples. `volume` is a percentage; above 100 counts as 100.
    pub fn samples(&self, sample_rate: u32, channels: u16, volume: u8) -> Vec<f32> {
        let gain = f32::from(volume.min(100)) / 100.0;
        let mut samples = Tone::samples(&self.spec(), sample_rate, channels);
        for sample in samples.iter_mut() {
            *sample *= gain;
        }
        samples
    }
}
