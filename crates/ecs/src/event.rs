use std::collections::HashMap;
use std::any::TypeId;
use as_any::{AsAny, Downcast};
use flatbox_core::catch::CatchError;
use flume::{Sender, Receiver};
use hecs::Component;

pub trait Event: Component + Clone {}
impl<E: Component + Clone> Event for E {}

#[derive(Default)]
pub struct EventHandler {
    writers: HashMap<TypeId, Box<dyn GenericWriter>>,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler::default()
    }

    pub fn send<E: Event>(&self, event: E) {
        if let Some(writer_boxed) = self.writers.get(&TypeId::of::<E>()) {
            if let Some(writer) = writer_boxed.downcast_ref::<EventWriter<E>>() {
                writer.send(event);
            }
        }
    }

    pub fn push_writer<E: Event>(&mut self) -> EventReader<E> {
        let (tx, rx) = flume::unbounded();
        self.writers.insert(TypeId::of::<E>(), Box::new(EventWriter::new(tx)));
        EventReader::new(rx)
    }
}

pub trait GenericWriter: AsAny + Send + Sync + 'static {}

pub struct EventWriter<E: Event> {
    sender: Sender<E>,
}

impl<E: Event> EventWriter<E> {
    pub fn new(sender: Sender<E>) -> EventWriter<E> {
        EventWriter { sender }
    }

    pub fn send(&self, event: E) {
        self.sender.send(event).catch();
    }

    pub fn is_dropped(&self) -> bool {
        self.sender.is_disconnected()
    }
}

impl<E: Event> GenericWriter for EventWriter<E> {}

pub struct EventReader<E: Event> {
    receiver: Receiver<E>,
}

impl<E: Event> EventReader<E> {
    pub fn new(receiver: Receiver<E>) -> EventReader<E> {
        EventReader { receiver }
    }

    pub fn read(&self) -> Option<E> {
        self.receiver.try_recv().ok()
    }
}