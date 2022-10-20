use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::*;

const A4: f32 = 440.0;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    let varying_tone = Sine::oscillator(A4)
        .v_oct(Sine::oscillator(0.1))
        .mul(Sine::oscillator(0.2));
    let stable_tone = Triangle::oscillator(A4);
    let level = Sine::oscillator(1.0).mul(Const(0.5)).add(Const(1.0));
    let synth = varying_tone.mix(stable_tone, level).clip(Const(0.5));

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
