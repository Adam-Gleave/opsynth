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

    // Create a clock for generating triggers.
    let clock = Clock::bpm(440.0);

    // Switch through a sequence of "control voltages" using the clock triggers.
    // These values will modulate the VCO frequecy.
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

    // Modulate a VCO, with a base frequency of C4.
    // This gives us our basic 16-step sequencer.
    let sequencer = Triangle::oscillator(C4).v_oct(notes);

    // Create an attack/decay envelope, triggered by the same clock source.
    let envelope = clock.ad_envelope(0.001, 0.1);

    // Modulate amplitude using the envelope.
    let voice = sequencer.mul(envelope);

    // Add some movement with some variable clipping, driven by a couple of
    // sine wave LFOs.
    let clip_lfo = Sine::oscillator(0.1)
        .mul(0.3)
        .add(0.6)
        .add(Sine::oscillator(0.07).mul(0.1));

    let synth = voice.clip(clip_lfo).mul(0.8);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
