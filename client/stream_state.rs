use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StreamState {
    Idle = 0,
    Opening = 1,
    Active = 2,
    Reconnecting = 3,
    Closing = 4,
    Closed = 5,
}

impl From<u8> for StreamState {
    fn from(v: u8) -> Self {
        match v {
            0 => StreamState::Idle,
            1 => StreamState::Opening,
            2 => StreamState::Active,
            3 => StreamState::Reconnecting,
            4 => StreamState::Closing,
            5 => StreamState::Closed,
            _ => StreamState::Idle,
        }
    }
}

pub struct StreamStateMachine {
    state: Arc<AtomicU8>,
}

impl StreamStateMachine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(StreamState::Idle as u8)),
        }
    }

    pub fn current(&self) -> StreamState {
        self.state.load(Ordering::SeqCst).into()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.current(), StreamState::Active)
    }

    pub fn should_reconnect(&self) -> bool {
        matches!(self.current(), 
            StreamState::Idle | 
            StreamState::Reconnecting
        )
    }

    pub fn is_closing(&self) -> bool {
        matches!(self.current(), 
            StreamState::Closing | 
            StreamState::Closed
        )
    }

    pub fn transition_to_opening(&self) -> bool {
        let current = self.current();
        if matches!(current, StreamState::Idle | StreamState::Reconnecting) {
            self.state.store(StreamState::Opening as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_active(&self) -> bool {
        let current = self.current();
        if current == StreamState::Opening {
            self.state.store(StreamState::Active as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_reconnecting(&self) -> bool {
        let current = self.current();
        if matches!(current, StreamState::Active | StreamState::Opening) {
            self.state.store(StreamState::Reconnecting as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_closing(&self) -> bool {
        let current = self.current();
        if !matches!(current, StreamState::Closed) {
            self.state.store(StreamState::Closing as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_closed(&self) -> bool {
        self.state.store(StreamState::Closed as u8, Ordering::SeqCst);
        true
    }
}

impl Default for StreamStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
