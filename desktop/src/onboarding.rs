//! First use starts in the normal generation page; there is no mandatory setup flow.
use super::*;
impl Desktop {
    /// Compatibility for old menu callers: preserve the current draft and enter generation.
    pub fn open_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.setup_open = false;
        self.begin_add(window, cx);
    }
}
