use crate::detect::Trigger;
use crate::detect::TriggerState;
use crate::Block;
use crate::Lerp;
use crate::Operator;
use crate::OperatorExt;
use crate::SynthContext;

#[derive(Debug, Clone)]
pub struct Ad<A, D, T> {
    attack: A,
    decay: D,
    trigger: Trigger<T>,
    time: Option<f32>,
}

pub fn ad<A, D, T>(trigger: T, attack: A, decay: D) -> Ad<A, D, T>
where
    A: Operator,
    D: Operator,
    T: Operator,
{
    Ad {
        attack,
        decay,
        trigger: trigger.trigger(),
        time: None,
    }
}

impl<A, D, T> Operator for Ad<A, D, T>
where
    A: Operator,
    D: Operator,
    T: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let attack = self.attack.render(context);
        let decay = self.decay.render(context);
        let trigger = self.trigger.render(context);

        Block::from_sample_fn(|i| {
            let attack = attack[i].max(0.0);
            let decay = decay[i].max(0.0);
            let trigger: TriggerState = trigger[i].max(0.0).into();

            if trigger == TriggerState::High {
                self.time = Some(0.0)
            }

            if let Some(time) = self.time.as_mut() {
                if *time < attack {
                    let progress = *time / attack;
                    let sample = 0.0.lerp(1.0, progress);

                    *time += context.sample_time();

                    sample
                } else if *time < attack + decay {
                    let progress = (*time - attack) / decay;
                    let sample = 1.0.lerp(0.0, progress);

                    *time += context.sample_time();

                    sample
                } else {
                    self.time = None;
                    0.0
                }
            } else {
                0.0
            }
        })
    }
}
