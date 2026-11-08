use crate::traits::CueOutput;
use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::num::{NonZeroU16, NonZeroU32};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

// set by the stream error callback when the device goes away; play reads it
#[derive(Clone, Default)]
pub struct DeviceLost(Arc<AtomicBool>);

impl DeviceLost {
    pub fn mark(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_lost(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    pub fn reset(&self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

// the default sound device through rodio; the sink must stay alive or the sound stops
#[derive(Default)]
pub struct RodioOutput {
    sink: Option<MixerDeviceSink>,
    rate: u32,
    channels: u16,
    lost: DeviceLost,
}

impl RodioOutput {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CueOutput for RodioOutput {
    fn open(&mut self) -> Result<(u32, u16), String> {
        self.lost.reset();
        let lost = self.lost.clone();
        let builder = DeviceSinkBuilder::from_default_device().map_err(|error| error.to_string())?.with_error_callback(
            move |error| {
                curia::warn!("sound device reported an error", { error: error.to_string() });
                lost.mark();
            },
        );
        let mut sink = builder.open_sink_or_fallback().map_err(|error| error.to_string())?;
        // the game owns the process; no message on drop
        sink.log_on_drop(false);
        let config = sink.config();
        self.rate = config.sample_rate().get();
        self.channels = config.channel_count().get();
        self.sink = Some(sink);
        Ok((self.rate, self.channels))
    }

    fn play(&mut self, samples: Vec<f32>) -> Result<(), String> {
        if self.lost.is_lost() {
            return Err("sound device was lost".to_string());
        }
        let sink = self.sink.as_ref().ok_or("output is not open")?;
        let rate = NonZeroU32::new(self.rate).ok_or("sample rate is zero")?;
        let channels = NonZeroU16::new(self.channels).ok_or("channel count is zero")?;
        sink.mixer().add(SamplesBuffer::new(channels, rate, samples));
        Ok(())
    }

    fn close(&mut self) {
        self.sink = None;
    }
}
