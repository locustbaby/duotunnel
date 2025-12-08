use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ConnectionState {
    Disconnected = 0,
    Connecting = 1,
    Connected = 2,
    Reconnecting = 3,
    ShuttingDown = 4,
    Shutdown = 5,
}

impl From<u8> for ConnectionState {
    fn from(v: u8) -> Self {
        match v {
            0 => ConnectionState::Disconnected,
            1 => ConnectionState::Connecting,
            2 => ConnectionState::Connected,
            3 => ConnectionState::Reconnecting,
            4 => ConnectionState::ShuttingDown,
            5 => ConnectionState::Shutdown,
            _ => ConnectionState::Disconnected,
        }
    }
}

pub struct ConnectionStateMachine {
    state: Arc<AtomicU8>,
}

impl ConnectionStateMachine {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(ConnectionState::Disconnected as u8)),
        }
    }

    pub fn current(&self) -> ConnectionState {
        self.state.load(Ordering::SeqCst).into()
    }

    pub fn is_active(&self) -> bool {
        matches!(self.current(), 
            ConnectionState::Connecting | 
            ConnectionState::Connected | 
            ConnectionState::Reconnecting
        )
    }

    pub fn is_shutting_down(&self) -> bool {
        matches!(self.current(), 
            ConnectionState::ShuttingDown | 
            ConnectionState::Shutdown
        )
    }

    pub fn should_reconnect(&self) -> bool {
        matches!(self.current(), 
            ConnectionState::Disconnected | 
            ConnectionState::Reconnecting
        )
    }

    pub fn transition_to_connecting(&self) -> bool {
        let current = self.current();
        if matches!(current, ConnectionState::Disconnected | ConnectionState::Reconnecting) {
            self.state.store(ConnectionState::Connecting as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_connected(&self) -> bool {
        let current = self.current();
        if current == ConnectionState::Connecting {
            self.state.store(ConnectionState::Connected as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_reconnecting(&self) -> bool {
        let current = self.current();
        if matches!(current, ConnectionState::Connected | ConnectionState::Connecting) {
            self.state.store(ConnectionState::Reconnecting as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_shutting_down(&self) -> bool {
        let current = self.current();
        if !matches!(current, ConnectionState::Shutdown) {
            self.state.store(ConnectionState::ShuttingDown as u8, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn transition_to_shutdown(&self) -> bool {
        self.state.store(ConnectionState::Shutdown as u8, Ordering::SeqCst);
        true
    }

    pub fn clone_state(&self) -> Arc<AtomicU8> {
        self.state.clone()
    }
}

impl Default for ConnectionStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_transitions() {
        let sm = ConnectionStateMachine::new();
        
        assert_eq!(sm.current(), ConnectionState::Disconnected);
        assert!(sm.should_reconnect());
        
        assert!(sm.transition_to_connecting());
        assert_eq!(sm.current(), ConnectionState::Connecting);
        
        assert!(sm.transition_to_connected());
        assert_eq!(sm.current(), ConnectionState::Connected);
        assert!(sm.is_active());
        
        assert!(sm.transition_to_reconnecting());
        assert_eq!(sm.current(), ConnectionState::Reconnecting);
        
        assert!(sm.transition_to_shutting_down());
        assert!(sm.is_shutting_down());
        
        assert!(sm.transition_to_shutdown());
        assert_eq!(sm.current(), ConnectionState::Shutdown);
    }
}
