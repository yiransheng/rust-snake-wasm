#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&( &format_args!($($t)*).to_string().into()) ))
}

#[macro_export]
macro_rules! yield_from {
    ($e:expr) => {{
        use std::ops::{Generator, GeneratorState};

        let mut gen = $e;
        loop {
            match unsafe { gen.resume() } {
                GeneratorState::Complete(_) => break,
                GeneratorState::Yielded(x) => yield x,
            }
        }
    }};
}
