use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::*;

const LOW: f32 = 65.4;
const HIGH: f32 = 2093.0;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    let low = Sine::oscillator(LOW);
    let high = Sine::oscillator(HIGH);

    let filtered = low
        .add(high)
        .simple_lpf(100.0, context.sample_rate())
        .simple_lpf(100.0, context.sample_rate());

    let synth = filtered.mul(0.8);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
