// where the rendered samples go; the audio thread owns one and calls it from that thread only
pub trait CueOutput: Send + 'static {
    // open the output; returns the sample rate and the channel count it runs at
    fn open(&mut self) -> Result<(u32, u16), String>;

    // play interleaved samples at the rate and channels that open returned
    fn play(&mut self, samples: Vec<f32>) -> Result<(), String>;

    // drop the output; a later open makes a new one
    fn close(&mut self);
}
