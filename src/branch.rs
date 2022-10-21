use crate::detect::Trigger;
use crate::detect::TriggerState;
use crate::Block;
use crate::Operator;
use crate::SynthContext;

pub struct SequentialSwitch<I> {
    trigger: Trigger<I>,
    signals: Vec<Box<dyn Operator>>,
    index: usize,
}

impl<I> SequentialSwitch<I>
where
    I: Operator,
{
    pub fn new(trigger: Trigger<I>, signals: impl IntoIterator<Item = Box<dyn Operator>>) -> Self {
        Self {
            trigger,
            signals: signals.into_iter().collect(),
            index: 0,
        }
    }

    fn render_current_block(&mut self, context: &mut SynthContext) -> Block {
        self.signals[self.index].render(context)
    }

    fn next_index(&self) -> usize {
        (self.index + 1) % self.signals.len()
    }
}

impl<I> Operator for SequentialSwitch<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let trigger = self.trigger.render(context);
        let mut block = self.render_current_block(context);

        Block::from_sample_fn(|i| {
            let trigger: TriggerState = trigger[i].into();

            if trigger == TriggerState::High {
                self.index = self.next_index();
                block = self.render_current_block(context);
            }

            block[i]
        })
    }
}
