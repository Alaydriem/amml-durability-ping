use crate::support::FakeOutput;
use amml_durability_ping::CuePlayer;
use durability_ping_core::audio::Cue;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[test]
fn opens_lazily_on_first_cue() {
    let fake = FakeOutput::default();
    let player = CuePlayer::start(fake.clone());
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(fake.read(|s| s.opens), 0);

    player.play(Cue::Warning, 80);
    assert!(fake.wait_for(|s| s.plays == 1));
    assert_eq!(fake.read(|s| (s.opens, s.plays)), (1, 1));
    assert!(fake.read(|s| s.last_len) > 0);
    player.stop();
}

#[test]
fn reuses_open_stream() {
    let fake = FakeOutput::default();
    let player = CuePlayer::start(fake.clone());
    player.play(Cue::Warning, 80);
    player.play(Cue::Blocked, 80);
    assert!(fake.wait_for(|s| s.plays == 2));
    assert_eq!(fake.read(|s| s.opens), 1);
    player.stop();
}

#[test]
fn failed_play_reopens_next_time() {
    let fake = FakeOutput::default();
    fake.change(|s| s.fail_next_plays = 1);
    let player = CuePlayer::start(fake.clone());
    player.play(Cue::Warning, 80);
    assert!(fake.wait_for(|s| s.closes == 1));

    player.play(Cue::Warning, 80);
    assert!(fake.wait_for(|s| s.plays == 2));
    assert_eq!(fake.read(|s| s.opens), 2);
    assert_eq!(fake.read(|s| s.closes), 1);
    player.stop();
}

#[test]
fn failed_open_drops_cue_and_logs_once() {
    let fake = FakeOutput::default();
    fake.change(|s| s.fail_open = true);
    let player = CuePlayer::start(fake.clone());
    for _ in 0..3 {
        player.play(Cue::Warning, 80);
    }
    assert!(fake.wait_for(|s| s.opens == 3));
    assert_eq!(fake.read(|s| s.plays), 0);
    assert_eq!(player.warnings(), 1);
    player.stop();
}

#[test]
fn play_never_blocks() {
    let fake = FakeOutput::default();
    fake.change(|s| s.play_delay = Duration::from_millis(200));
    let player = CuePlayer::start(fake.clone());
    let start = Instant::now();
    for _ in 0..10 {
        player.play(Cue::Warning, 80);
    }
    assert!(start.elapsed() < Duration::from_millis(100));
    // do not wait for the queue to drain
    fake.change(|s| s.play_delay = Duration::ZERO);
    player.stop();
}

#[test]
fn stop_joins() {
    let fake = FakeOutput::default();
    let player = CuePlayer::start(fake.clone());
    player.play(Cue::Warning, 80);
    assert!(fake.wait_for(|s| s.plays == 1));
    player.stop();
    // the thread owned a clone of the fake; it is gone, and it closed the output
    assert_eq!(Arc::strong_count(&fake.state), 1);
    assert_eq!(fake.read(|s| s.closes), 1);
}

#[test]
fn device_lost_flag_marks_and_resets() {
    let lost = amml_durability_ping::DeviceLost::default();
    let callback_copy = lost.clone();
    assert!(!lost.is_lost());
    callback_copy.mark();
    assert!(lost.is_lost());
    lost.reset();
    assert!(!lost.is_lost());
}
