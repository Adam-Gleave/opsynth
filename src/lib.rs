use std::f32::consts::PI;
use std::fmt::Debug;
use std::fs::File;
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

pub trait PhaseSampler {
    fn sample(&mut self, phase: f32) -> f32;
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
}

impl<T> OperatorExt for T where T: Operator {}

pub fn volt_octave(frequency: f32, volt_octave: f32) -> f32 {
    frequency * 2_f32.powf(volt_octave)
}

#[derive(Debug, Clone, Copy)]
pub struct Sine;

impl PhaseSampler for Sine {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }
}

impl Sine {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Sine,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Saw;

impl PhaseSampler for Saw {
    fn sample(&mut self, phase: f32) -> f32 {
        (phase * 2.0) - 1.0
    }
}

impl Saw {
    pub fn oscillator(frequency: f32) -> VoltageOscillator<Silence, Self> {
        VoltageOscillator {
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Saw,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle;

impl PhaseSampler for Triangle {
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
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Triangle,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Square;

impl PhaseSampler for Square {
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
            v_oct: Silence,
            inner: Oscillator {
                frequency,
                phase: 0.0,
                inner: Square,
            },
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

impl Operator for f32 {
    fn render(&mut self, _: &mut SynthContext) -> Block {
        Block([*self; BLOCK_SIZE])
    }
}

#[derive(Debug, Clone)]
pub struct Oscillator<S> {
    frequency: f32,
    phase: f32,
    inner: S,
}

impl<S> Oscillator<S>
where
    S: PhaseSampler,
{
    pub fn shift_phase(self, offset: f32) -> Self {
        Self {
            frequency: self.frequency,
            phase: self.phase + offset,
            inner: self.inner,
        }
    }
}

impl<S> PhaseSampler for Oscillator<S>
where
    S: PhaseSampler,
{
    fn sample(&mut self, phase: f32) -> f32 {
        self.inner.sample(phase)
    }
}

#[derive(Debug, Clone)]
pub struct VoltageOscillator<Cv, S> {
    v_oct: Cv,
    inner: Oscillator<S>,
}

impl<Cv, S> VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    pub fn v_oct<I>(self, input: I) -> VoltageOscillator<I, S>
    where
        I: Operator,
    {
        VoltageOscillator {
            v_oct: input,
            inner: self.inner,
        }
    }
}

impl<Cv, S> PhaseSampler for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    fn sample(&mut self, phase: f32) -> f32 {
        self.inner.sample(phase)
    }
}

impl<Cv, S> Operator for VoltageOscillator<Cv, S>
where
    Cv: Operator,
    S: PhaseSampler,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let v_oct = self.v_oct.render(context);
        let sample_t = context.sample_time();

        let frequency = self.inner.frequency;

        Block::from_sample_fn(|i| {
            let frequency = volt_octave(frequency, v_oct[i]);

            let phase = self.inner.phase;
            let phase = (phase + frequency * sample_t) % 1.0;
            self.inner.phase = phase;

            self.sample(phase)
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

pub type Mix<Lhs, Rhs, Cv> = Add<Lhs, Mul<Rhs, Cv>>;

#[derive(Debug, Clone)]
pub struct Min<Lhs, Rhs> {
    lhs: Lhs,
    rhs: Rhs,
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
    lhs: Lhs,
    rhs: Rhs,
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
    input: I,
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
    input: I,
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

pub struct Clip<I, Cv> {
    input: I,
    level: Cv,
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

pub struct Clock {
    interval_sec: f32,
    completed: u32,
}

impl Clock {
    pub fn bpm(bpm: f32) -> Self {
        Self {
            interval_sec: 60.0 / bpm,
            completed: 0,
        }
    }
}

impl Operator for Clock {
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let interval = (self.interval_sec * context.sample_rate as f32).ceil() as u32;

        Block::from_sample_fn(|_| {
            if self.completed == interval {
                self.completed = 0;
                1.0
            } else {
                self.completed += 1;
                0.0
            }
        })
    }
}

pub struct Gate<Cv> {
    interval_sec: f32,
    completed: u32,
    width: Cv,
}

impl Gate<Const> {
    pub fn bpm(bpm: f32) -> Self {
        Self {
            interval_sec: 60.0 / bpm,
            completed: 0,
            width: Const(0.5),
        }
    }
}

impl<Cv> Gate<Cv>
where
    Cv: Operator,
{
    pub fn width(self, input: Cv) -> Self {
        Self {
            interval_sec: self.interval_sec,
            completed: self.completed,
            width: input,
        }
    }
}

impl<Cv> Operator for Gate<Cv>
where
    Cv: Operator,
{
    fn render(&mut self, context: &mut SynthContext) -> Block {
        let interval = (self.interval_sec * context.sample_rate as f32).ceil() as u32;
        let width = self.width.render(context);

        Block::from_sample_fn(|i| {
            let width_sec = width[i] * self.interval_sec;
            let width = (width_sec * context.sample_rate as f32).ceil() as u32;

            let sample = if self.completed == interval {
                self.completed = 0;
                1.0
            } else if self.completed < width {
                1.0
            } else {
                0.0
            };

            self.completed += 1;
            sample
        })
    }
}

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
    input: I,
    previous_sample: TriggerState,
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

impl<I> Sink<I, WavFile>
where
    I: Operator,
{
    pub fn wav(input: I, output: WavFile) -> Self {
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

use std::io::BufWriter;
use std::path::Path;

use hound::WavSpec;
use hound::WavWriter;

pub struct WavFile {
    writer: WavWriter<BufWriter<File>>,
}

impl WavFile {
    pub fn from_path<P: AsRef<Path>>(path: P, sample_rate: u32) -> hound::Result<Self> {
        let spec = WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };

        Ok(Self {
            writer: WavWriter::create(path, spec)?,
        })
    }
}

impl AudioOut for WavFile {
    fn write(&mut self, input: Block) {
        for sample in input {
            self.writer.write_sample(sample).ok();
        }
    }
}
