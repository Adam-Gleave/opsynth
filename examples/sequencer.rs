use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::*;

const C4: f32 = 261.6;
const SEMITONE: f32 = 1.0 / 12.0;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    let cv = Clock::bpm(80.0).sequential_switch([
        Const(0.0).boxed(),
        Const(4.0 * SEMITONE).boxed(),
        Const(7.0 * SEMITONE).boxed(),
        Const(7.0 * SEMITONE).boxed(),
    ]);
    let sequencer = Sine::oscillator(C4).v_oct(cv);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(sequencer, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
