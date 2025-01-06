#![allow(non_snake_case)]

use therustymod_gen::{therustymod_lib};
use std::ffi::{CString, c_char, CStr, c_int};
use lazy_static::lazy_static;
use std::sync::{Arc};
use futures::lock::Mutex;
use std::collections::HashMap;
use tokio::sync::mpsc;
use askama::Template;

mod shader;
mod sound;
mod database;
mod embeddings;
mod llm;

struct SoundDefinition {
    pcm: Vec<u8>,
    shader: Option<shader::SoundShader>,
}

struct ModSndGenerator {
    sound_factory: Arc<Mutex<sound::SoundFactory>>,

    answers: HashMap<c_int, CString>,
    sounds: Arc<Mutex<HashMap<String, Arc<Mutex<SoundDefinition>>>>>,
}

#[derive(Clone)]
enum CommsMode {
    ASK,
    TELL,
    SAY
}

#[derive(Template)] // this will generate the code...
#[template(path = "webpage.html", escape = "none")]
struct WebpageTemplate {
    definitions: Vec<(String, String, String)>
}

struct ModSndGeneratorComms {
    chan_send: mpsc::Sender<(CommsMode, String, String)>,
    chan_recv: Mutex<mpsc::Receiver<(CommsMode, String, String)>>,

    a_chan_send: mpsc::Sender<c_int>,
    a_chan_recv: Mutex<mpsc::Receiver<c_int>>
}

impl ModSndGeneratorComms {
    pub fn new() -> ModSndGeneratorComms {
        let (sender_async, receiver_async) = mpsc::channel::<(CommsMode, String, String)>(10);
        let (a_sender_async, a_receiver_async) = mpsc::channel::<c_int>(10);
        ModSndGeneratorComms {
            chan_send: sender_async,
            chan_recv: Mutex::new(receiver_async),
            a_chan_send: a_sender_async,
            a_chan_recv: Mutex::new(a_receiver_async),
        }
    }

    pub async fn recv(&self) -> (CommsMode, String, String) {
        self.chan_recv.lock().await.recv().await.unwrap()
    }

    pub fn a_recv(&self) -> c_int {
        self.a_chan_recv.try_lock().unwrap().blocking_recv().unwrap()
    }
}

impl ModSndGenerator {
    pub fn new() -> ModSndGenerator {
        ModSndGenerator {
            sound_factory: Arc::new(Mutex::new(sound::SoundFactory::new())),
            sounds: Arc::new(Mutex::new(HashMap::new())),
            answers: HashMap::new(),
        }
    }

    pub fn init_mod_snd_generator(&mut self) -> bool {
        true
    }

    pub fn ask(query: String) -> c_int {
        println!("Going to ask: {}", query.clone());
        MOD_SND_GENERATOR_COMMS.chan_send.blocking_send((CommsMode::ASK, query.clone(), "".to_string())).unwrap();
        println!("Asking: {}", query.clone());
        let request_index = MOD_SND_GENERATOR_COMMS.a_recv();
        request_index
    }

    pub fn record_answer(&mut self, request_index: c_int, answer: String) {
        self.answers.insert(request_index, CString::new(answer).unwrap());
    }

    pub fn allocate_answer(&mut self) -> c_int {
        let request_index: c_int = self.answers.len().try_into().unwrap();
        self.answers.insert(request_index, CString::new("").unwrap());
        request_index
    }

    pub fn get_answer_if_ready(&mut self, request_index: c_int) -> &CStr {
        if let Some(answer) = self.answers.get(&request_index) {
            if answer.count_bytes() > 0 {
                println!("Got answer: {}", answer.to_string_lossy());
            }
            answer.as_ref()
        } else {
            panic!("Unknown key {}", request_index)
        }
    }

    pub fn tell(&mut self, content: String) {
        println!("Going to tell: {}", content.clone());
        MOD_SND_GENERATOR_COMMS.chan_send.blocking_send((CommsMode::TELL, content.clone(), "".to_string())).unwrap();
        println!("Telling: {}", content.clone());
    }

    pub async fn generate_sound(sound_name: String, sentence: String) {
        let sample_name = format!("fromMemory {}", sound_name);

        let sound_factory = {
            MOD_SND_GENERATOR.lock().await.sound_factory.clone()
        };
        let cur = sound_factory.lock().await.say(sentence.clone());
        let sound_shader = shader::SoundShader::new(sound_name.clone(), sample_name.clone(), sentence);
        let buffer: Vec<u8> = cur.to_owned();
        let mut buffer = unsafe { CString::from_vec_unchecked(buffer) }.into_bytes();
        let object_mem_size = buffer.len();
        let object_size = object_mem_size / std::mem::size_of::<::std::os::raw::c_short>(); // channels?
        let load_pcm_from_memory = TRM_SYSTEM.lock().unwrap().abi.unwrap().loadPCMFromMemory.unwrap();
        let pcm_successful: bool = unsafe {
            load_pcm_from_memory(
                CString::new(sample_name).unwrap().as_ptr(),
                1,
                16,
                22050,
                object_size as i32,
                object_mem_size as i32,
                CString::new(sound_name.clone()).unwrap().as_ptr(),
                buffer.as_mut_ptr()
            )
        };
        if !pcm_successful {
            panic!("Could not send sound sample to TDM: {}", sound_name.clone());
        }
        MOD_SND_GENERATOR.try_lock().unwrap().record_sound(sound_name, cur, sound_shader);
    }

    pub fn remove_sound(&mut self, sound_name: String) {
        let mut sounds = self.sounds.try_lock().unwrap();
        sounds.remove(
            &sound_name,
        );
    }

    pub fn record_sound(&mut self, sound_name: String, cur: Vec<u8>, sound_shader: shader::SoundShader) {
        let mut sounds = self.sounds.try_lock().unwrap();
        let definition = SoundDefinition {
            pcm: cur,
            shader: Some(sound_shader)
        };
        sounds.insert(
            sound_name,
            Arc::new(Mutex::new(definition))
        );
    }

    pub fn get_sound_shader_subtitle_buffer_size(&self, sound_name: String) -> c_int {
        let sounds = self.sounds.try_lock().unwrap();
        let definition = sounds.get(&sound_name).unwrap().try_lock().unwrap();
        let length = definition.shader.as_ref().unwrap().subtitle_buffer_length;
        print!("Subtitle buffer length: {}\n", length);
        length
    }

    pub fn get_sound_shader_subtitle_buffer(&self, sound_name: String) -> *mut c_char {
        let sounds = self.sounds.try_lock().unwrap();
        let definition = sounds.get(&sound_name).unwrap().try_lock().unwrap();
        let buffer: *mut c_char = definition.shader.as_ref().unwrap().subtitle_buffer;
        let buffer_chk = unsafe { CStr::from_ptr(buffer) }.to_string_lossy().clone();
        print!("Subtitle buffer size: {}\n", buffer_chk.len());
        buffer
    }

    pub fn get_sound_shader_buffer_size(&self, sound_name: String) -> c_int {
        let sounds = self.sounds.try_lock().unwrap();
        let definition = sounds.get(&sound_name).unwrap().try_lock().unwrap();
        let length = definition.shader.as_ref().unwrap().buffer_length;
        print!("Buffer length: {}\n", length);
        length
    }

    pub fn get_sound_shader_buffer(&self, sound_name: String) -> *mut c_char {
        let sounds = self.sounds.try_lock().unwrap();
        let definition = sounds.get(&sound_name).unwrap().try_lock().unwrap();
        let buffer: *mut c_char = definition.shader.as_ref().unwrap().buffer;
        let buffer_chk = unsafe { CStr::from_ptr(buffer) }.to_string_lossy().clone();
        print!("Buffer size: {}\n", buffer_chk.len());
        buffer
    }

    pub fn free_sound_shader_buffer(&self, sound_name: String) {
        let sounds = self.sounds.try_lock().unwrap();
        let mut definition = sounds.get(&sound_name).unwrap().try_lock().unwrap();
        definition.shader = None; // TODO: check this frees
    }
}

unsafe impl Send for SoundDefinition {}

lazy_static! {
    static ref MOD_SND_GENERATOR: Arc<Mutex<ModSndGenerator>> = Arc::new(Mutex::new(ModSndGenerator::new()));
    static ref MOD_SND_GENERATOR_COMMS: ModSndGeneratorComms = ModSndGeneratorComms::new();
}

#[therustymod_lib(daemon=true)]
mod mod_snd_generator {
    async fn __run() {
        let query = "Who am I?";
        println!("Starting: test ask");
        let context = database::retrieve(query).await.unwrap();
        println!("Starting: asked");
        let answer = llm::answer_with_context(&query, context).await.unwrap();
        println!("Starting: answered: {}", answer);
        loop {
            println!("Listening async");
            let (mode, query, param) = MOD_SND_GENERATOR_COMMS.recv().await;
            match mode {
                CommsMode::TELL => {
                    database::insert(&query).await.unwrap();
                    println!("Recv async channel (TELL): {:?}", query.clone());
                },
                CommsMode::SAY => {
                    ModSndGenerator::generate_sound(query.clone(), param.clone()).await;
                    println!("Recv async channel (SAY): {:?} -- {:?}", query.clone(), param.clone());
                },
                CommsMode::ASK => {
                    println!("Recv async channel (ASK): {:?}", query.clone());
                    let request_index: c_int = {
                        let mut generator = MOD_SND_GENERATOR.lock().await;
                        generator.allocate_answer()
                    };
                    MOD_SND_GENERATOR_COMMS.a_chan_send.send(request_index).await.unwrap();
                    println!("Send request index: {:?}", request_index);
                    let context = database::retrieve(&query).await.unwrap();
                    println!("Recv from DB");
                    let answer = llm::answer_with_context(&query, context).await.unwrap();
                    println!("Ready to send on async channel: {:?}", answer.clone());
                    {
                        let mut generator = MOD_SND_GENERATOR.lock().await;
                        generator.record_answer(request_index, answer.clone())
                    };
                    println!("Sent on async channel for request {}: {:?}", request_index, answer.clone());
                },
            }
        }
    }

    fn init_mod_snd_generator() -> bool {
        MOD_SND_GENERATOR.try_lock().unwrap().init_mod_snd_generator()
    }

    fn ask(sentence: *const c_char) -> c_int {
        let sentence = unsafe { CStr::from_ptr(sentence) }.to_string_lossy().clone().to_string();
        ModSndGenerator::ask(sentence)
    }

    fn get_answer_if_ready(request_index: c_int) -> *const c_char {
        MOD_SND_GENERATOR.try_lock().unwrap().get_answer_if_ready(request_index).as_ptr()
    }

    fn tell(sentence: *const c_char) {
        let sentence = unsafe { CStr::from_ptr(sentence) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().tell(sentence);
    }

    fn generate_sound(sound_name: *const c_char, sentence: *const c_char) {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        let sentence = unsafe { CStr::from_ptr(sentence) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().remove_sound(sound_name.clone());
        MOD_SND_GENERATOR_COMMS.chan_send.blocking_send((CommsMode::SAY, sound_name, sentence)).unwrap();
    }

    fn is_sound_ready(sound_name: *const c_char) -> bool {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().sounds.try_lock().unwrap().contains_key(&sound_name)
    }

    fn get_sound_shader_subtitle_buffer_size(sound_name: *const c_char) -> c_int {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().get_sound_shader_subtitle_buffer_size(sound_name)
    }

    fn get_sound_shader_subtitle_buffer(sound_name: *const c_char) -> *mut byte {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().get_sound_shader_subtitle_buffer(sound_name)
    }

    fn get_sound_shader_buffer_size(sound_name: *const c_char) -> c_int {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().get_sound_shader_buffer_size(sound_name)
    }

    fn get_webpage() -> *mut byte {
        let generator = MOD_SND_GENERATOR.try_lock().unwrap();
        let sounds = generator.sounds.try_lock().unwrap();
        let definitions = sounds.iter().map(|(sound_name, sound)| {
            if let Some(shader) = &sound.try_lock().unwrap().shader {
                (
                    sound_name.clone(),
                    unsafe { CStr::from_ptr(shader.buffer) }.to_string_lossy().clone().to_string(),
                    unsafe { CStr::from_ptr(shader.subtitle_buffer) }.to_string_lossy().clone().to_string()
                )
            } else {
                (
                    sound_name.clone(),
                    "(none)".to_string(),
                    "(none)".to_string(),
                )
            }
        }).collect();
        CString::new(
            WebpageTemplate {
                definitions
            }.render().unwrap()
        ).unwrap().into_raw()
    }

    fn get_sound_shader_buffer(sound_name: *const c_char) -> *mut byte {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().get_sound_shader_buffer(sound_name)
    }

    fn free_sound_shader_buffer(sound_name: *const c_char) {
        let sound_name = unsafe { CStr::from_ptr(sound_name) }.to_string_lossy().clone().to_string();
        MOD_SND_GENERATOR.try_lock().unwrap().free_sound_shader_buffer(sound_name)
    }
}
