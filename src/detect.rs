use crate::Block;
use crate::Operator;
use crate::SynthContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerState {
    Low,
    High,
}

impl From<f32> for TriggerState {
    fn from(sample: f32) -> Self {
        if sample < 1.0 {
            Self::Low
        } else {
            Self::High
        }
    }
}

impl Into<f32> for TriggerState {
    fn into(self) -> f32 {
        match self {
            Self::Low => 0.0,
            Self::High => 1.0,
        }
    }
}

pub struct Trigger<I> {
    pub input: I,
    pub previous_sample: TriggerState,
}

impl<I> Operator for Trigger<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        Block::from_sample_fn(|i| {
            let input = input[i].into();

            let state = if self.previous_sample == TriggerState::Low && input == TriggerState::High
            {
                TriggerState::High
            } else {
                TriggerState::Low
            };

            self.previous_sample = input;
            state.into()
        })
    }
}
