// use std::cell::Cell;

use std::time::Duration;

use rodio::{source, OutputStream, Sink, Source};

pub trait Tone {
    fn start_tone(&self) {}
    fn stop_tone(&self) {}
    fn is_tone_on(&self) -> bool {
        false
    }
}

pub struct Beeper {
    _stream: OutputStream,
    sink: rodio::Sink,
}

impl Beeper {
    pub fn new(freq_hz: u32) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default()
            .expect("Should be able to obtain an output stream for audio");
        let sink = Sink::try_new(&stream_handle)
            .expect("Should be able to create Sink from output stream.");
        sink.pause();

        let source = source::SineWave::new(freq_hz as f32)
            .take_duration(Duration::from_secs_f32(0.25))
            .repeat_infinite()
            .amplify(0.20);
        sink.append(source);

        Self { _stream, sink }
    }
}

impl Tone for Beeper {
    fn is_tone_on(&self) -> bool {
        !self.sink.is_paused()
    }

    fn start_tone(&self) {
        self.sink.play();
    }

    fn stop_tone(&self) {
        self.sink.pause();
    }
}
