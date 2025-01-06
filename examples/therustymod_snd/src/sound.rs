use std::f32::consts::PI;
use std::i16;
use std::path::Path;
use std::io::prelude::*;
use std::vec::Vec;
use riff_wave::WaveWriter;
use hound;

const MAX_WAV_VALUE_I16: f32 = 32767.0;

const I16MIN_F32: f32 = i16::MIN as f32;
const I16MAX_F32: f32 = i16::MAX as f32;

use piper_rs::synth::PiperSpeechSynthesizer;


pub struct SoundFactory {
}

impl SoundFactory {
    pub fn new() -> SoundFactory {
        SoundFactory { }
    }

    pub fn make_sine_wave(&self) -> std::io::Cursor<Vec<u8>> {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let vec: Vec<u8> = Vec::new();
        let mut cur = std::io::Cursor::new(vec);
        {
            let mut writer = hound::WavWriter::new(&mut cur, spec).unwrap();
            for t in (0 .. 22050).map(|x| x as f32 / 22050.0) {
                let sample = (t * 440.0 * 2.0 * PI).sin();
                let amplitude = i16::MAX as f32;
                writer.write_sample((sample * amplitude) as i16).unwrap();
            }
        }
        cur.seek(std::io::SeekFrom::Start(0)).unwrap();
        cur
    }

    pub fn say(&self, text: String) -> Vec<u8> {
        let config_path = std::env::var("PIPER_CONFIG_PATH").expect("Please specify config path");
        let sid = std::env::var("PIPER_SID").ok();
        let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
        // Set speaker ID
        if let Some(sid) = sid {
            let sid = sid.parse::<i64>().expect("Speaker ID should be number!");
            model.set_speaker(sid);
        }
        let synth = PiperSpeechSynthesizer::new(model).unwrap();
        let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();
        let mut samples: Vec<f32> = Vec::new();

        for result in synth.synthesize_parallel(text, None).unwrap() {
            match result {
                Ok(ws) => {
                    samples.append(&mut ws.into_vec());
                }
                Err(e) => {
                    panic!("Piper error: {}", e)
                }
            };
        }
        if samples.is_empty() {
            panic!(
                "{}",
                "No speech data to write",
            );
        }
        let sample_rate = model.audio_output_info().unwrap().sample_rate as u32;
        let num_channels: u32 = model.audio_output_info().unwrap().num_channels.try_into().unwrap();
        let sample_width: u32 = model.audio_output_info().unwrap().sample_width.try_into().unwrap();

        let mut buffer: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());
        {
            let Ok(mut wave_writer) = WaveWriter::new(
                    num_channels as u16,
                    sample_rate,
                    (sample_width * 8) as u16,
                    &mut buffer,
            ) else {
                panic!(
                    "{}",
                    "Failed to initialize wave writer".to_string(),
                )
            };
            let min_audio_value = samples
                .iter()
                .min_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
            let max_audio_value = samples
                .iter()
                .max_by(|x, y| x.partial_cmp(y).unwrap())
                .unwrap();
            let abs_max = max_audio_value
                .abs()
                .max(min_audio_value.abs())
                .max(f32::EPSILON);
            let audio_scale = MAX_WAV_VALUE_I16 / abs_max / 2.;
            let samples = Vec::from_iter(
                samples
                    .iter().map(|f| (f * audio_scale).clamp(I16MIN_F32, I16MAX_F32) as i16),
            );
            let any_fail = samples.iter()
                .map(|i| wave_writer.write_sample_i16(*i))
                .any(|r| r.is_err());
            if any_fail {
                panic!("{}", "Failed to write wave samples".to_string());
            }
            if wave_writer.sync_header().is_err() {
                panic!("{}", "Failed to update wave header".to_string());
            }
        }
        buffer.into_inner()
    }
}
