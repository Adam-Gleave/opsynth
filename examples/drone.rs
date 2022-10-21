use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::*;

const C3: f32 = 130.86;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    // Play C3 as base note, with octave and major third.
    let vco_a = Sine::oscillator(C3);
    let vco_b = Sine::oscillator(C3).v_oct(1.0);
    let vco_c = Sine::oscillator(C3).v_oct(4.0 / 12.0);

    // Create square wave at the octave, and apply slight frequency modulation.
    let fm_lfo = Sine::oscillator(0.05).mul(0.001).add(1.0);
    let vco_d = Square::oscillator(C3).v_oct(fm_lfo).mul(0.25);

    // Mix oscillators and do some hard clipping.
    let voice = vco_a
        .mix(vco_b, 0.75)
        .mix(vco_c, 0.5)
        .mix(vco_d, 0.35)
        .clip(0.8);

    // Apply some amplitude modulation to the voice.
    let am_lfo = Sine::oscillator(0.02).mul(0.2).add(0.6);
    let synth = voice.mul(am_lfo);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
