use crate::Block;
use crate::Operator;
use crate::SynthContext;

use ordered_float::OrderedFloat;

const SEMITONE_CV: f32 = 1.0 / 12.0;

const ALL_INTERVALS: [OrderedFloat<f32>; 13] = [
    OrderedFloat(0.0),
    OrderedFloat(1.0),
    OrderedFloat(2.0),
    OrderedFloat(3.0),
    OrderedFloat(4.0),
    OrderedFloat(5.0),
    OrderedFloat(6.0),
    OrderedFloat(7.0),
    OrderedFloat(8.0),
    OrderedFloat(9.0),
    OrderedFloat(10.0),
    OrderedFloat(11.0),
    OrderedFloat(12.0),
];

const MAJOR_INTERVALS: [OrderedFloat<f32>; 8] = [
    OrderedFloat(0.0),
    OrderedFloat(2.0),
    OrderedFloat(4.0),
    OrderedFloat(5.0),
    OrderedFloat(7.0),
    OrderedFloat(9.0),
    OrderedFloat(11.0),
    OrderedFloat(12.0),
];

const MINOR_INTERVALS: [OrderedFloat<f32>; 8] = [
    OrderedFloat(0.0),
    OrderedFloat(2.0),
    OrderedFloat(3.0),
    OrderedFloat(5.0),
    OrderedFloat(7.0),
    OrderedFloat(8.0),
    OrderedFloat(10.0),
    OrderedFloat(12.0),
];

#[derive(Default, Debug, Clone, Copy)]
pub enum QuantizeMode {
    #[default]
    All,
    Major,
    Minor,
}

#[derive(Debug, Clone)]
pub struct Quantizer<I> {
    pub input: I,
    pub mode: QuantizeMode,
}

impl<I> Operator for Quantizer<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        Block::from_sample_fn(|i| {
            let input = input[i];

            let rem = if input.fract() < 0.0 {
                input.fract() + 1.0
            } else {
                input.fract()
            };
            let rounded = input.floor();

            let intervals = match self.mode {
                QuantizeMode::All => ALL_INTERVALS.as_slice().iter(),
                QuantizeMode::Major => MAJOR_INTERVALS.as_slice().iter(),
                QuantizeMode::Minor => MINOR_INTERVALS.as_slice().iter(),
            };

            let interval = intervals
                .map(|interval| interval * OrderedFloat(SEMITONE_CV))
                .min_by_key(|interval| OrderedFloat((interval - OrderedFloat(rem)).abs()))
                .unwrap();

            rounded + interval.into_inner()
        })
    }
}
