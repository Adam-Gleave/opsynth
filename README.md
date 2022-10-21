# opsynth

`opsynth` is a library for simple, composable modular synthesis.

This crate provides a number of small constructs for audio signal processing, that can be composed together to form increasingly complex signal chains.

## Example

Here is a basic modulated drone synth:

```rust
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
```

The full code needed to render audio from this signal chain is provided in [examples/drone.rs](examples/drone.rs)
