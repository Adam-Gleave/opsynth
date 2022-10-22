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

    let lfo = Triangle::oscillator(0.2);

    let major = lfo.clone().quantize(scales::QuantizeMode::Major);
    let vco = Sine::oscillator(A4);

    let synth = vco.v_oct(major).mul(0.8);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
