use std::cell::Cell;
use std::rc::Rc;

/// Create single-threaded mpsc channels for sending `Copy` data
pub fn channel<T: Copy>() -> (Sender<T>, Receiver<T>) {
    let c1 = Rc::new(Cell::new(None));
    let c2 = c1.clone();

    (Sender { shared: c1 }, Receiver { shared: c2 })
}

#[derive(Debug, Copy, Clone)]
pub struct SendError;

#[derive(Clone)]
pub struct Sender<T> {
    shared: Rc<Cell<Option<T>>>,
}
pub struct Receiver<T> {
    shared: Rc<Cell<Option<T>>>,
}

impl<T: Copy> Sender<T> {
    pub fn send(&self, t: T) -> Result<(), SendError> {
        match self.shared.get() {
            None => {
                self.shared.set(Some(t));
                Ok(())
            }
            _ => Err(SendError),
        }
    }
}
impl<T: Copy> Receiver<T> {
    pub fn recv(&self) -> Option<T> {
        self.shared.take()
    }
}
