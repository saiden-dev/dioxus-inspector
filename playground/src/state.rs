//! Application state management.

use dioxus::prelude::*;

/// Global application state using Signals.
#[derive(Clone, Copy)]
pub struct AppState {
    pub counter: Signal<i32>,
    pub message: Signal<String>,
}

impl AppState {
    /// Create initial application state.
    pub fn new() -> Self {
        Self {
            counter: Signal::new(0),
            message: Signal::new("Hello from Inspector Playground!".to_string()),
        }
    }

    /// Increment the counter.
    pub fn increment(&mut self) {
        *self.counter.write() += 1;
    }

    /// Decrement the counter.
    pub fn decrement(&mut self) {
        *self.counter.write() -= 1;
    }

    /// Reset the counter.
    pub fn reset(&mut self) {
        *self.counter.write() = 0;
    }

    /// Update the message.
    pub fn set_message(&mut self, msg: impl Into<String>) {
        *self.message.write() = msg.into();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let runtime = dioxus::dioxus_core::Runtime::new();
        let _guard = runtime.enter();

        let state = AppState::new();
        assert_eq!(*state.counter.read(), 0);
        assert!(state.message.read().contains("Hello"));
    }
}
