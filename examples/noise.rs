use cpal::traits::DeviceTrait;
use cpal::traits::HostTrait;
use opsynth::filter::SinglePoleHpf;
use opsynth::filter::SinglePoleLpf;
use opsynth::*;

const CUTOFF: f32 = 3000.0;

fn main() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device
        .supported_output_configs()
        .unwrap()
        .next()
        .unwrap()
        .with_max_sample_rate();

    let mut context = SynthContext::new(config.sample_rate().0);

    let noise = WhiteNoise.tap();

    let lpf = SinglePoleLpf::lpf(noise.clone(), CUTOFF, context.sample_rate())
        .simple_lpf(CUTOFF, context.sample_rate())
        .simple_lpf(CUTOFF, context.sample_rate());
    let hpf = SinglePoleHpf::hpf(noise.clone(), CUTOFF, context.sample_rate())
        .simple_hpf(CUTOFF, context.sample_rate())
        .simple_hpf(CUTOFF, context.sample_rate());
    let pass = noise.clone();

    let switch =
        Clock::bpm(30.0).sequential_switch([lpf.boxed(), hpf.boxed(), pass.boxed()].into_iter());

    let synth = switch.mul(0.8);

    let cpal_out = CpalMono::new(&device, &config);
    let mut sink = Sink::cpal_mono(synth, cpal_out);

    loop {
        context.render_to_sink(&mut sink);
    }
}
