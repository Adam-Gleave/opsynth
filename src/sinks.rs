use crate::Block;
use crate::Operator;
use crate::SynthContext;

use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use cpal::traits::DeviceTrait;
use cpal::traits::StreamTrait;
use cpal::Sample;
use cpal::SampleFormat;
use hound::WavSpec;
use hound::WavWriter;
use rtrb::Consumer;
use rtrb::Producer;
use rtrb::RingBuffer;

pub trait AudioOut {
    fn write(&mut self, input: Block);
}

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
