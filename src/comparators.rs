use crate::Block;
use crate::Operator;
use crate::SynthContext;

pub struct GreaterThan<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for GreaterThan<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs > rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}

pub struct GreaterThanOrEqualTo<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for GreaterThanOrEqualTo<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs >= rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}

pub struct LessThan<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for LessThan<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs < rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}

pub struct LessThanOrEqualTo<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for LessThanOrEqualTo<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs >= rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}

pub struct EqualTo<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for EqualTo<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs == rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}

pub struct NotEqualTo<Lhs, Rhs> {
    pub lhs: Lhs,
    pub rhs: Rhs,
}

impl<Lhs, Rhs> Operator for NotEqualTo<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| {
            let lhs = lhs[i];
            let rhs = rhs[i];

            if lhs != rhs {
                1.0
            } else {
                0.0
            }
        })
    }
}
