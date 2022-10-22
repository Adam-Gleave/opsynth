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

    let clock = Clock::bpm(400.0);

    let notes = clock.clone().sequential_switch([
        Const(0.0).boxed(),
        Const(4.0 * SEMITONE).boxed(),
        Const(7.0 * SEMITONE).boxed(),
        Const(4.0 * SEMITONE).boxed(),
        Const(12.0 * SEMITONE).boxed(),
        Const(4.0 * SEMITONE).boxed(),
        Const(10.0 * SEMITONE).boxed(),
        Const(12.0 * SEMITONE).boxed(),
        Const(15.0 * SEMITONE).boxed(),
        Const(7.0 * SEMITONE).boxed(),
        Const(10.0 * SEMITONE).boxed(),
        Const(12.0 * SEMITONE).boxed(),
        Const(17.0 * SEMITONE).boxed(),
        Const(12.0 * SEMITONE).boxed(),
        Const(16.0 * SEMITONE).boxed(),
        Const(16.0 * SEMITONE).boxed(),
    ]);

    let sequencer = Triangle::oscillator(C4)
        .v_oct(notes)
        .mul(clock.ad_envelope(0.005, 0.2));

    let synth = sequencer.mul(0.8);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
