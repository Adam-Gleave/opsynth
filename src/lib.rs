use std::f32::consts::PI;
use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

pub const BLOCK_SIZE: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct Block([f32; BLOCK_SIZE]);

pub const SILENCE: [f32; BLOCK_SIZE] = [0f32; BLOCK_SIZE];

impl Deref for Block {
    type Target = [f32; BLOCK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Block {
    fn silence() -> Self {
        Self(SILENCE)
    }

    fn from_sample_fn<F>(mut f: F) -> Self
    where
        F: FnMut(usize) -> f32,
    {
        let mut samples = SILENCE;

        for (i, sample) in samples.iter_mut().enumerate() {
            *sample = f(i)
        }

        Self(samples)
    }
}

pub struct SynthContext {
    sample_rate: u32,
    sample_count: u32,
}

impl SynthContext {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            sample_count: 0,
        }
    }

    pub fn time(&self) -> f32 {
        self.sample_count as f32 * self.sample_time()
    }

    pub fn sample_time(&self) -> f32 {
        1.0 / self.sample_rate as f32
    }

    pub fn update(&mut self) {
        self.sample_count += BLOCK_SIZE as u32;
    }
}

pub trait Source: Debug {
    fn sample(&self, phase: f32) -> f32;
}

pub trait Operator: Debug {
    fn render(&self, context: &mut SynthContext) -> Block;
}

pub trait OperatorExt
where
    Self: Sized,
{
    fn add<Rhs>(self, rhs: Rhs) -> Add<Self, Rhs> {
        Add { lhs: self, rhs }
    }

    fn sub<Rhs>(self, rhs: Rhs) -> Sub<Self, Rhs> {
        Sub { lhs: self, rhs }
    }

    fn mul<Rhs>(self, rhs: Rhs) -> Mul<Self, Rhs> {
        Mul { lhs: self, rhs }
    }

    fn div<Rhs>(self, rhs: Rhs) -> Div<Self, Rhs> {
        Div { lhs: self, rhs }
    }
}

impl<T> OperatorExt for T where T: Operator {}

pub fn volt_octave(frequency: f32, volt_octave: f32) -> f32 {
    frequency * 2_f32.powf(volt_octave)
}

const MIDDLE_C: f32 = 256.0;

#[derive(Debug, Clone, Copy)]
pub struct Sine;

impl Source for Sine {
    fn sample(&self, phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }
}

impl Sine {
    pub fn vco(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            frequency,
            v_oct: Silence,
            inner: Sine,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Silence;

impl Operator for Silence {
    fn render(&self, _: &mut SynthContext) -> Block {
        Block::silence()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Const(f32);

impl Operator for Const {
    fn render(&self, _: &mut SynthContext) -> Block {
        Block([self.0; BLOCK_SIZE])
    }
}

#[derive(Debug, Clone)]
pub struct VoltageOscillator<Cv, S> {
    frequency: f32,
    v_oct: Cv,
    inner: S,
}

impl<Cv, S> Operator for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: Source,
{
    fn render(&self, context: &mut SynthContext) -> Block {
        let v_oct = self.v_oct.render(context);

        let block_t = context.time();
        let sample_t = context.sample_time();

        let mut phase = (self.frequency * block_t) % 1.0;

        Block::from_sample_fn(|i| {
            let frequency = volt_octave(self.frequency, v_oct[i]);
            phase = (phase + frequency * sample_t) % 1.0;
            self.inner.sample(phase)
        })
    }
}

#[derive(Debug, Clone)]
pub struct Add<Lhs, Rhs> {
    lhs: Lhs,
    rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Add<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] + rhs[i])
    }
}

#[derive(Debug, Clone)]
pub struct Sub<Lhs, Rhs> {
    lhs: Lhs,
    rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Sub<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] - rhs[i])
    }
}

#[derive(Debug, Clone)]
pub struct Mul<Lhs, Rhs> {
    lhs: Lhs,
    rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Mul<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] * rhs[i])
    }
}

#[derive(Debug, Clone)]
pub struct Div<Lhs, Rhs> {
    lhs: Lhs,
    rhs: Rhs,
}

impl<Lhs, Rhs> Operator for Div<Lhs, Rhs>
where
    Lhs: Operator,
    Rhs: Operator,
{
    fn render(&self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] / rhs[i])
    }
}

#[derive(Debug)]
pub struct Switch<const N: usize> {
    choice: usize,
    inputs: [BoxedOperator; N],
}

impl<const N: usize> Operator for Switch<N> {
    fn render(&self, context: &mut SynthContext) -> Block {
        if let Some(input) = self.inputs.get(self.choice).as_ref() {
            input.render(context)
        } else {
            Silence.render(context)
        }
    }
}

#[derive(Debug)]
pub struct BoxedOperator(Box<dyn Operator>);

impl Deref for BoxedOperator {
    type Target = Box<dyn Operator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for BoxedOperator
where
    T: Sized + Operator + 'static,
{
    fn from(operator: T) -> Self {
        Self(Box::new(operator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine() {
        let mut context = SynthContext::new(44_100);

        let a = Sine::vco(MIDDLE_C);
        let block = a.render(&mut context);
        println!("{:?}\n", block);

        let b = Sine::vco(MIDDLE_C).add(Const(1.0));
        let block = b.render(&mut context);
        println!("{:?}\n", block);

        let c = Switch {
            choice: 0,
            inputs: [a.clone().into(), b.clone().into()],
        };
        let block = c.render(&mut context);
        println!("{:?}\n", block);

        let d = Switch {
            choice: 1,
            inputs: [a.clone().into(), b.clone().into()],
        };
        let block = d.render(&mut context);
        println!("{:?}\n", block);

        context.update();

        let block = Sine::vco(MIDDLE_C).render(&mut context);
        println!("{:?}\n", block);
    }
}
