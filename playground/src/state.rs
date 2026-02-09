//! Application state management.

use dioxus::prelude::*;

/// Navigation sections in the sidebar.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Section {
    #[default]
    Counter,
    Message,
    Inspector,
}

/// Global application state using Signals.
#[derive(Clone, Copy)]
pub struct AppState {
    pub counter: Signal<i32>,
    pub message: Signal<String>,
    pub sidebar_open: Signal<bool>,
    pub active_section: Signal<Section>,
}

impl AppState {
    /// Create initial application state.
    pub fn new() -> Self {
        Self {
            counter: Signal::new(0),
            message: Signal::new("Hello from Inspector Playground!".to_string()),
            sidebar_open: Signal::new(true),
            active_section: Signal::new(Section::Counter),
        }
    }

    /// Toggle sidebar visibility.
    pub fn toggle_sidebar(&mut self) {
        let current = *self.sidebar_open.read();
        self.sidebar_open.set(!current);
    }

    /// Set active section.
    pub fn set_section(&mut self, section: Section) {
        self.active_section.set(section);
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
