use std::collections::VecDeque;

use crate::Block;
use crate::Operator;
use crate::SynthContext;

#[derive(Debug, Clone)]
pub struct Delay<I> {
    input: I,
    length: usize,
    buffer: VecDeque<f32>,
}

impl<I> Delay<I>
where
    I: Operator,
{
    pub(crate) fn delay(input: I, time: f32, sample_rate: u32) -> Self {
        let length = (time * sample_rate as f32).round() as usize;

        Self {
            input,
            length,
            buffer: VecDeque::with_capacity(length),
        }
    }
}

impl<I> Operator for Delay<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        Block::from_sample_fn(|i| {
            let sample = if self.buffer.len() == self.length {
                self.buffer.pop_front().unwrap()
            } else {
                0.0
            };

            self.buffer.push_back(input[i]);
            sample
        })
    }
}
