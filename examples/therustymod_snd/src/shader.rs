use std::ffi::{CString, c_char, CStr, c_int};
use askama::Template;

#[derive(Template)] // this will generate the code...
#[template(path = "shader.tmpl", escape = "none")]
struct ShaderTemplate {
    sound_name: String,
    sample_name: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "subtitle.tmpl", escape = "none")]
struct SubtitleTemplate {
    sound_name: String,
    sample_name: String,
    sentence: String,
}

pub struct SoundShader {
    pub sound_name: String,
    pub sample_name: String,

    pub buffer: *mut c_char,
    pub buffer_length: c_int,

    pub subtitle_buffer: *mut c_char,
    pub subtitle_buffer_length: c_int
}

impl SoundShader {
    pub fn new(sound_name: String, sample_name: String, sentence: String) -> SoundShader {
        let buffer = CString::new(
            ShaderTemplate {
                sound_name: sound_name.clone(),
                sample_name: sample_name.clone()
            }.render().unwrap()
        ).unwrap();

        let subtitle_buffer = CString::new(
            SubtitleTemplate {
                sound_name: sound_name.clone(),
                sample_name: sample_name.clone(),
                sentence: sentence.clone(),
            }.render().unwrap()
        ).unwrap();

        // Assumes this is short enough that usize/i32 not relevant
        let buffer_len: i32 = buffer.count_bytes().try_into().unwrap();
        let subtitle_buffer_len: i32 = subtitle_buffer.count_bytes().try_into().unwrap();

        SoundShader {
            sound_name,
            sample_name,
            buffer: buffer.into_raw(),
            buffer_length: buffer_len,
            subtitle_buffer: subtitle_buffer.into_raw(),
            subtitle_buffer_length: subtitle_buffer_len
        }
    }
}
