use durability_ping_core::audio::Cue;

const PEAK: f32 = 0.25;

fn peak_of(samples: &[f32]) -> f32 {
    samples.iter().fold(0.0f32, |m, s| m.max(s.abs()))
}

#[test]
fn samples_length() {
    for cue in [Cue::Warning, Cue::Blocked] {
        let frames = (48_000.0f32 * cue.duration_seconds()).ceil() as usize;
        assert_eq!(cue.samples(48_000, 2, 100).len(), frames * 2);
    }
}

#[test]
fn peak_matches_volume() {
    for cue in [Cue::Warning, Cue::Blocked] {
        assert!((peak_of(&cue.samples(48_000, 1, 100)) - PEAK).abs() < 1e-3);
        assert!((peak_of(&cue.samples(48_000, 1, 50)) - PEAK / 2.0).abs() < 1e-3);
        assert!(cue.samples(48_000, 1, 0).iter().all(|s| *s == 0.0));
    }
}

#[test]
fn volume_above_100_clamps() {
    let full = Cue::Warning.samples(48_000, 1, 100);
    assert_eq!(Cue::Warning.samples(48_000, 1, 255), full);
}

#[test]
fn ends_quiet() {
    for cue in [Cue::Warning, Cue::Blocked] {
        let samples = cue.samples(48_000, 1, 100);
        let peak = peak_of(&samples);
        let tail = samples.len() / 100;
        assert!(samples[samples.len() - tail..].iter().all(|s| s.abs() < 0.01 * peak));
    }
}

#[test]
fn channels_identical() {
    for cue in [Cue::Warning, Cue::Blocked] {
        let samples = cue.samples(48_000, 2, 100);
        assert!(samples.chunks(2).all(|f| f[0] == f[1]));
    }
}

#[test]
fn cues_differ() {
    assert_ne!(Cue::Warning.samples(48_000, 1, 100), Cue::Blocked.samples(48_000, 1, 100));
}
