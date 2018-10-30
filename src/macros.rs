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

#[macro_export]
macro_rules! as_f64 {
    ($x:ident) => {
        $x.to_f64().unwrap()
    };
    ($x:ident + $t:tt) => {
        $x.to_f64().unwrap() + as_f64!($t)
    };
    ($x:ident * $t:tt) => {
        $x.to_f64().unwrap() * as_f64!($t)
    };
}

#[macro_export]
macro_rules! as_u32 {
    ($x:ident) => {
        $x.to_u32().unwrap()
    };
    ($x:ident + $t:tt) => {
        $x.to_u32().unwrap() + as_u32!($t)
    };
    ($x:ident * $t:tt) => {
        $x.to_u32().unwrap() * as_u32!($t)
    };
}
