use crate::Block;
use crate::Operator;
use crate::SynthContext;

#[derive(Debug, Clone)]
pub struct Add<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Add<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] + rhs[i])
    }
}

#[derive(Debug, Clone)]
pub struct Sub<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Sub<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] - rhs[i])
    }
}

#[derive(Debug, Clone)]
pub struct Mul<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Mul<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] * rhs[i])
    }
}

pub type Mix<Lhs, Rhs, Cv> = Add<Lhs, Mul<Rhs, Cv>>;

#[derive(Debug, Clone)]
pub struct Min<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Min<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i].min(rhs[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Max<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Max<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i].max(rhs[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Abs<I> {
    pub input: I,
}

impl<I> Operator for Abs<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        Block::from_sample_fn(|i| input[i].abs())
    }
}

#[derive(Debug, Clone)]
pub struct Invert<I> {
    pub input: I,
}

impl<I> Operator for Invert<I>
where
    I: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);

        Block::from_sample_fn(|i| 0.0 - input[i])
    }
}

#[derive(Debug, Clone)]
pub struct Clip<I, Cv> {
    pub input: I,
    pub level: Cv,
}

impl<I, Cv> Operator for Clip<I, Cv>
where
    I: Operator,
    Cv: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let input = self.input.render(context);
        let level = self.level.render(context);

        Block::from_sample_fn(|i| {
            let input = input[i];
            let level = level[i].abs();

            if input.abs() <= level {
                input
            } else if input.is_sign_negative() {
                0.0 - level
            } else {
                level
            }
        })
    }
}
