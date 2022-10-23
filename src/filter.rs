use std::f32::consts::E;
use std::f32::consts::PI;

use crate::Block;
use crate::Operator;
use crate::SynthContext;

#[derive(Debug, Clone, Copy)]
pub struct SinglePoleLpf<I> {
    input: I,
    decay: f32,
    buffer: f32,
}

impl<I> SinglePoleLpf<I>
where
    I: Operator,
{
    pub fn lpf(input: I, cutoff: f32, sample_rate: u32) -> Self {
        let fc = (cutoff / sample_rate as f32).min(0.5);
        let decay = E.powf(-2.0 * PI * fc);

        Self {
            input,
            decay,
            buffer: 0.0,
        }
    }
}

impl<I> Operator for SinglePoleLpf<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        let a0 = 1.0 - self.decay;
        let b1 = self.decay;

        Block::from_sample_fn(|i| {
            let input = input[i];

            let output = input * a0 + self.buffer * b1;
            self.buffer = output;
            output
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SinglePoleHpf<I> {
    input: I,
    decay: f32,
    buffer: f32,
}

impl<I> SinglePoleHpf<I>
where
    I: Operator,
{
    pub fn hpf(input: I, cutoff: f32, sample_rate: u32) -> Self {
        let fc = (cutoff / sample_rate as f32).min(0.5);
        let decay = 0.0 - E.powf(-2.0 * PI * (0.5 - fc));

        Self {
            input,
            decay,
            buffer: 0.0,
        }
    }
}

impl<I> Operator for SinglePoleHpf<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        let a0 = 1.0 + self.decay;
        let b1 = self.decay;

        Block::from_sample_fn(|i| {
            let input = input[i];

            let output = input * a0 + self.buffer * b1;
            self.buffer = output;
            output
        })
    }
}
