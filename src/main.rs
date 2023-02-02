use pomodoro::start;

fn main() {
    if let Err(err) = start() {
        eprintln!("{err}");
    }
}
