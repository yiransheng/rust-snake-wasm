#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&( &format_args!($($t)*).to_string().into()) ))
}
