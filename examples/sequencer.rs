use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::*;

const C4: f32 = 261.6;
const E4: f32 = 329.6;
const G4: f32 = 392.0;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    let synth = Clock::bpm(80.0).sequential_switch([
        Sine::oscillator(C4).boxed(),
        Sine::oscillator(E4).boxed(),
        Sine::oscillator(G4).boxed(),
        Sine::oscillator(G4).boxed(),
    ]);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
