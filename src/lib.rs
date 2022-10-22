pub mod branch;
pub mod comparators;
pub mod delay;
pub mod detect;
pub mod envelope;
pub mod math;
pub mod scales;
pub mod sinks;
pub mod sources;

pub use sinks::AudioOut;
pub use sinks::CpalMono;
pub use sinks::Sink;
pub use sinks::WavFile;
pub use sources::Clock;
pub use sources::Const;
pub use sources::Gate;
pub use sources::Saw;
pub use sources::Silence;
pub use sources::Sine;
pub use sources::Square;
pub use sources::Triangle;
pub use sources::WhiteNoise;

use branch::*;
use comparators::*;
use delay::*;
use detect::*;
use envelope::*;
use math::*;
use scales::*;

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

impl IntoIterator for Block {
    type Item = f32;
    type IntoIter = std::array::IntoIter<Self::Item, BLOCK_SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn render_to_sink<I, O>(&mut self, sink: &mut Sink<I, O>)
    where
        I: Operator,
        O: AudioOut,
    {
        sink.render(self);
        self.update();
    }

    fn update(&mut self) {
        self.sample_count += BLOCK_SIZE as u32;
    }
}

pub trait Operator {
    fn render(&mut self, context: &mut SynthContext) -> Block;
}

pub trait OperatorExt
where
    Self: Sized,
{
    fn boxed(self) -> Box<dyn Operator>
    where
        Self: Operator + 'static,
    {
        Box::new(self)
    }

    fn add<Rhs>(self, rhs: Rhs) -> Add<Self, Rhs> {
        Add { lhs: self, rhs }
    }

    fn sub<Rhs>(self, rhs: Rhs) -> Sub<Self, Rhs> {
        Sub { lhs: self, rhs }
    }

    fn mul<Rhs>(self, rhs: Rhs) -> Mul<Self, Rhs> {
        Mul { lhs: self, rhs }
    }

    fn clip<Cv>(self, level: Cv) -> Clip<Self, Cv> {
        Clip { input: self, level }
    }

    fn mix<Rhs, Cv>(self, rhs: Rhs, level: Cv) -> Mix<Self, Rhs, Cv>
    where
        Rhs: Operator,
    {
        Mix {
            lhs: self,
            rhs: rhs.mul(level),
        }
    }

    fn min<Rhs>(self, rhs: Rhs) -> Min<Self, Rhs> {
        Min { lhs: self, rhs }
    }

    fn max<Rhs>(self, rhs: Rhs) -> Max<Self, Rhs> {
        Max { lhs: self, rhs }
    }

    fn abs(self) -> Abs<Self> {
        Abs { input: self }
    }

    fn invert(self) -> Invert<Self> {
        Invert { input: self }
    }

    fn trigger(self) -> Trigger<Self>
    where
        Self: Operator,
    {
        Trigger {
            input: self,
            previous_sample: TriggerState::Low,
        }
    }

    fn sequential_switch(
        self,
        signals: impl IntoIterator<Item = Box<dyn Operator>>,
    ) -> SequentialSwitch<Self>
    where
        Self: Operator,
    {
        SequentialSwitch::new(self.trigger(), signals)
    }

    fn ad_envelope<A, D>(self, attack: A, decay: D) -> Ad<A, D, Self>
    where
        A: Operator,
        D: Operator,
        Self: Operator,
    {
        envelope::ad(self, attack, decay)
    }

    fn delay(self, time: f32, sample_rate: u32) -> Delay<Self>
    where
        Self: Operator,
    {
        Delay::delay(self, time, sample_rate)
    }

    fn tap(self) -> Tap<Self>
    where
        Self: Operator,
    {
        Tap::tap(self)
    }

    fn quantize(self, mode: QuantizeMode) -> Quantizer<Self> {
        Quantizer { input: self, mode }
    }

    fn greater_than<Rhs>(self, rhs: Rhs) -> GreaterThan<Self, Rhs> {
        GreaterThan { lhs: self, rhs }
    }

    fn greater_than_or_equal_to<Rhs>(self, rhs: Rhs) -> GreaterThanOrEqualTo<Self, Rhs> {
        GreaterThanOrEqualTo { lhs: self, rhs }
    }

    fn less_than<Rhs>(self, rhs: Rhs) -> LessThan<Self, Rhs> {
        LessThan { lhs: self, rhs }
    }

    fn less_than_or_equal_to<Rhs>(self, rhs: Rhs) -> LessThanOrEqualTo<Self, Rhs> {
        LessThanOrEqualTo { lhs: self, rhs }
    }

    fn equal_to<Rhs>(self, rhs: Rhs) -> EqualTo<Self, Rhs> {
        EqualTo { lhs: self, rhs }
    }

    fn not_equal_to<Rhs>(self, rhs: Rhs) -> NotEqualTo<Self, Rhs> {
        NotEqualTo { lhs: self, rhs }
    }
}

impl<T> OperatorExt for T where T: Operator {}

pub fn volt_octave(frequency: f32, volt_octave: f32) -> f32 {
    frequency * 2_f32.powf(volt_octave)
}

pub trait Lerp
where
    Self: Sized,
{
    fn lerp(self, other: f32, factor: f32) -> f32;
}

impl Lerp for f32 {
    fn lerp(self, other: f32, factor: f32) -> f32 {
        let factor = factor.clamp(0.0, 1.0);
        (self * (1.0 - factor)) + (other * factor)
    }
}
