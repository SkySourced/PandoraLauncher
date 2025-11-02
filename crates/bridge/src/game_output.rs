#[derive(Clone, Copy, Debug)]
pub enum GameOutputLogLevel {
    Fatal,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
    Other,
}
