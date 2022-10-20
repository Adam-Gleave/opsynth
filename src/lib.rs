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

pub trait PhasedOscillator {
    fn sample(&mut self, phase: f32) -> f32;
}

pub trait Operator {
    fn render(&mut self, context: &mut SynthContext) -> Block;
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

    fn mix<Rhs, Cv>(self, rhs: Rhs, level: Cv) -> Mix<Self, Rhs, Cv>
    where
        Rhs: Operator,
    {
        Mix {
            lhs: self,
            rhs: rhs.mul(level),
        }
    }
}

impl<T> OperatorExt for T where T: Operator {}

pub fn volt_octave(frequency: f32, volt_octave: f32) -> f32 {
    frequency * 2_f32.powf(volt_octave)
}

#[derive(Debug, Clone, Copy)]
pub struct Sine;

impl PhasedOscillator for Sine {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }
}

impl Sine {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            frequency,
            phase: 0.0,
            v_oct: Silence,
            inner: Sine,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Saw;

impl PhasedOscillator for Saw {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0) - 1.0
    }
}

impl Saw {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            frequency,
            phase: 0.0,
            v_oct: Silence,
            inner: Saw,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle;

impl PhasedOscillator for Triangle {
    fn sample(&mut self, phase: f32) -> f32 {
        if phase < 0.25 {
            phase * 4.0
        } else if phase < 0.75 {
            2.0 - (phase * 4.0)
        } else {
            phase * 4.0 - 4.0
        }
    }
}

impl Triangle {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            frequency,
            phase: 0.0,
            v_oct: Silence,
            inner: Triangle,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Square;

impl PhasedOscillator for Square {
    fn sample(&mut self, phase: f32) -> f32 {
        if phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

impl Square {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            frequency,
            phase: 0.0,
            v_oct: Silence,
            inner: Square,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Silence;

impl Operator for Silence {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block::silence()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Const(pub f32);

impl Operator for Const {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block([self.0; BLOCK_SIZE])
    }
}

#[derive(Debug, Clone)]
pub struct VoltageOscillator<Cv, S> {
    frequency: f32,
    phase: f32,
    v_oct: Cv,
    inner: S,
}

impl<Cv, S> VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhasedOscillator,
{
    pub fn v_oct<I>(self, input: I) -> VoltageOscillator<I, S>
    where
        I: Operator,
    {
        VoltageOscillator {
            frequency: self.frequency,
            phase: self.phase,
            v_oct: input,
            inner: self.inner,
        }
    }
}

impl<Cv, S> Operator for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhasedOscillator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let v_oct = self.v_oct.render(context);
        let sample_t = context.sample_time();

        Block::from_sample_fn(|i| {
            let frequency = volt_octave(self.frequency, v_oct[i]);
            self.phase = (self.phase + frequency * sample_t) % 1.0;
            self.inner.sample(self.phase)
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
    fn render(&mut self, context: &mut SynthContext) -> Block {
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
    fn render(&mut self, context: &mut SynthContext) -> Block {
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
    fn render(&mut self, context: &mut SynthContext) -> Block {
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
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let lhs = self.lhs.render(context);
        let rhs = self.rhs.render(context);

        Block::from_sample_fn(|i| lhs[i] / rhs[i])
    }
}

pub type Mix<Lhs, Rhs, Cv> = Add<Lhs, Mul<Rhs, Cv>>;

pub struct Switch<const N: usize> {
    pub choice: usize,
    pub inputs: [BoxedOperator; N],
}

impl<const N: usize> Operator for Switch<N> {
    fn render(&mut self, context: &mut SynthContext) -> Block {
        if let Some(input) = self.inputs.get_mut(self.choice).as_deref_mut() {
            input.render(context)
        } else {
            Silence.render(context)
        }
    }
}

pub struct BoxedOperator(Box<dyn Operator>);

impl Deref for BoxedOperator {
    type Target = Box<dyn Operator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoxedOperator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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

pub trait AudioOut {
    fn write(&mut self, input: Block);
}

use std::fmt::Display;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::Sample;
use cpal::SampleFormat;
use rtrb::Consumer;
use rtrb::Producer;
use rtrb::RingBuffer;

const RING_BUFFER_CAPACITY: usize = 4096;

pub struct Sink<I, O> {
    input: I,
    inner: O,
}

impl<I> Sink<I, CpalMono>
where
    I: Operator,
{
    pub fn cpal_mono(input: I, output: CpalMono) -> Self {
        Self {
            input,
            inner: output,
        }
    }
}

impl<I, O> Operator for Sink<I, O>
where
    I: Operator,
    O: AudioOut,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        self.inner.write(self.input.render(context));
        Block::silence()
    }
}

pub struct CpalMono {
    buffer: Producer<f32>,
    _stream: cpal::Stream,
}

impl CpalMono {
    pub fn new(device: &cpal::Device, config: &cpal::SupportedStreamConfig) -> Self {
        let channels = config.channels() as usize;
        let sample_format = config.sample_format();
        let config = config.clone().into();

        let (producer, mut consumer) = RingBuffer::<f32>::new(RING_BUFFER_CAPACITY);

        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], info: &cpal::OutputCallbackInfo| {
                    data_callback(&mut consumer, data, info, channels);
                },
                error_callback,
            ),
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], info: &cpal::OutputCallbackInfo| {
                    data_callback(&mut consumer, data, info, channels);
                },
                error_callback,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], info: &cpal::OutputCallbackInfo| {
                    data_callback(&mut consumer, data, info, channels);
                },
                error_callback,
            ),
        }
        .expect("error building output stream");

        stream.play().unwrap();

        Self {
            buffer: producer,
            _stream: stream,
        }
    }
}

impl AudioOut for CpalMono {
    fn write(&mut self, input: Block) {
        for sample in input {
            while self.buffer.is_full() {}
            self.buffer.push(sample).ok();
        }
    }
}

fn data_callback<T: Sample + Display>(
    input: &mut Consumer<f32>,
    data: &mut [T],
    _: &cpal::OutputCallbackInfo,
    channels: usize,
) {
    for frame in data.chunks_mut(channels) {
        let sample = T::from(&input.pop().unwrap_or_default());
        frame.fill(sample);
    }
}

fn error_callback(err: cpal::StreamError) {
    // TODO
    eprintln!("error in output stream: {}", err);
}
