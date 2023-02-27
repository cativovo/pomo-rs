use notify_rust::Notification as NotifyRust;
use rodio::{Decoder, OutputStream, Sink};
use std::io::BufReader;
use std::{fs, thread};

fn play_sound(path: &str) {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();

    let file = fs::File::open(path).unwrap();
    sink.append(Decoder::new(BufReader::new(file)).unwrap());
    sink.sleep_until_end();
}

pub struct Notification {
    notifier: NotifyRust,
}

impl Notification {
    pub fn new() -> Notification {
        let notifier = NotifyRust::new();

        Notification { notifier }
    }

    fn update_body_and_summary(&mut self, body: &str, summary: &str) {
        self.notifier.body(body).summary(summary);
    }

    fn notify(&self, sound_path: &'static str) {
        let notify_clone = self.notifier.clone();

        thread::spawn(move || {
            notify_clone.show().unwrap();
            play_sound(sound_path);
        });
    }

    pub fn notify_work(&mut self) {
        self.update_body_and_summary("Work time", "It's time to work");
        self.notify("assets/OGG_Polite.ogg");
    }

    pub fn notify_break(&mut self) {
        self.update_body_and_summary("Break time", "Take a break");
        self.notify("assets/OGG_Calm.ogg");
    }
}
