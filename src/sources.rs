use crate::Block;
use crate::Operator;
use crate::SynthContext;
use crate::BLOCK_SIZE;

use std::f32::consts::PI;

pub trait PhaseSampler {
    fn sample(&mut self, phase: f32) -> f32;
}

#[derive(Debug, Clone, Copy)]
pub struct Sine;

impl PhaseSampler for Sine {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }
}

impl Sine {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Sine,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Saw;

impl PhaseSampler for Saw {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0) - 1.0
    }
}

impl Saw {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Saw,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle;

impl PhaseSampler for Triangle {
    fn sample(&mut self, phase: f32) -> f32 {
        if phase < 0.25 {
            phase * 4.0
        } else if phase < 0.75 {
            2.0 - (phase * 4.0)
        } else {
            phase * 4.0 - 4.0
        }
    }
}

impl Triangle {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Triangle,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Square;

impl PhaseSampler for Square {
    fn sample(&mut self, phase: f32) -> f32 {
        if phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

impl Square {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Square,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Silence;

impl Operator for Silence {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block::silence()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Const(pub f32);

impl Operator for Const {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block([self.0; BLOCK_SIZE])
    }
}

impl Operator for f32 {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block([*self; BLOCK_SIZE])
    }
}

#[derive(Debug, Clone)]
pub struct Oscillator<S> {
    frequency: f32,
    phase: f32,
    inner: S,
}

impl<S> Oscillator<S>
where
    S: PhaseSampler,
{
    pub fn shift_phase(self, offset: f32) -> Self {
        Self {
            frequency: self.frequency,
            phase: self.phase + offset,
            inner: self.inner,
        }
    }
}

impl<S> PhaseSampler for Oscillator<S>
where
    S: PhaseSampler,
{
    fn sample(&mut self, phase: f32) -> f32 {
        self.inner.sample(phase)
    }
}

#[derive(Debug, Clone)]
pub struct VoltageOscillator<Cv, S> {
    v_oct: Cv,
    inner: Oscillator<S>,
}

impl<Cv, S> VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    pub fn v_oct<I>(self, input: I) -> VoltageOscillator<I, S>
    where
        I: Operator,
    {
        VoltageOscillator {
            v_oct: input,
            inner: self.inner,
        }
    }
}

impl<Cv, S> VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    pub fn shift_phase(self, offset: f32) -> Self {
        Self {
            v_oct: self.v_oct,
            inner: self.inner.shift_phase(offset),
        }
    }
}

impl<Cv, S> PhaseSampler for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    fn sample(&mut self, phase: f32) -> f32 {
        self.inner.sample(phase)
    }
}

impl<Cv, S> Operator for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let v_oct = self.v_oct.render(context);
        let sample_t = context.sample_time();

        let frequency = self.inner.frequency;

        Block::from_sample_fn(|i| {
            let frequency = crate::volt_octave(frequency, v_oct[i]);

            let phase = self.inner.phase;
            let phase = (phase + frequency * sample_t) % 1.0;
            self.inner.phase = phase;

            self.sample(phase)
        })
    }
}

pub struct Clock {
    interval_sec: f32,
    completed: u32,
}

impl Clock {
    pub fn bpm(bpm: f32) -> Self {
        Self {
            interval_sec: 60.0 / bpm,
            completed: 0,
        }
    }
}

impl Operator for Clock {
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let interval = (self.interval_sec * context.sample_rate as f32).ceil() as u32;

        Block::from_sample_fn(|_| {
            if self.completed == interval {
                self.completed = 0;
                1.0
            } else {
                self.completed += 1;
                0.0
            }
        })
    }
}

pub struct Gate<Cv> {
    interval_sec: f32,
    completed: u32,
    width: Cv,
}

impl Gate<Const> {
    pub fn bpm(bpm: f32) -> Self {
        Self {
            interval_sec: 60.0 / bpm,
            completed: 0,
            width: Const(0.5),
        }
    }
}

impl<Cv> Gate<Cv>
where
    Cv: Operator,
{
    pub fn width(self, input: Cv) -> Self {
        Self {
            interval_sec: self.interval_sec,
            completed: self.completed,
            width: input,
        }
    }
}

impl<Cv> Operator for Gate<Cv>
where
    Cv: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let interval = (self.interval_sec * context.sample_rate as f32).ceil() as u32;
        let width = self.width.render(context);

        Block::from_sample_fn(|i| {
            let width_sec = width[i] * self.interval_sec;
            let width = (width_sec * context.sample_rate as f32).ceil() as u32;

            let sample = if self.completed == interval {
                self.completed = 0;
                1.0
            } else if self.completed < width {
                1.0
            } else {
                0.0
            };

            self.completed += 1;
            sample
        })
    }
}
