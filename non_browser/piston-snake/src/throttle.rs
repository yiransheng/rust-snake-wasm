use std::marker::PhantomData;

use piston_window::GenericEvent;

pub fn throttle<E, F>(threshold: f64, f: F) -> Throttled<E, F>
where
    E: GenericEvent,
    F: FnMut(&E),
{
    Throttled {
        threshold,
        accumulated: 0.0,
        _run: f,
        _event_type: PhantomData,
    }
}

pub struct Throttled<E, F> {
    threshold: f64,
    accumulated: f64,
    _run: F,
    _event_type: PhantomData<E>,
}

impl<E, F> Throttled<E, F>
where
    E: GenericEvent,
    F: FnMut(&E) -> (),
{
    pub fn run(&mut self, event: &E) {
        event.update(|arg| {
            self.accumulated += arg.dt;
        });

        if self.accumulated >= self.threshold {
            self.accumulated = 0.0;
            (self._run)(event);
        }
    }
}
