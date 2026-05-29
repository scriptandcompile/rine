//! Internal thread message queue core shared by user32 operations.

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

use rine_types::windows::{Msg, window_message};

/// A message queue for a single thread.
#[derive(Debug)]
pub(crate) struct MessageQueue {
    state: Mutex<MessageQueueState>,
    wake: Condvar,
}

#[derive(Debug, Default)]
struct MessageQueueState {
    messages: VecDeque<Msg>,
    quit: bool,
    quit_code: i32,
}

impl MessageQueue {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(MessageQueueState::default()),
            wake: Condvar::new(),
        }
    }

    pub(crate) fn post_message(&self, msg: Msg) {
        {
            let mut state = self.state.lock().unwrap();
            state.messages.push_back(msg);
        }
        self.wake.notify_one();
    }

    pub(crate) fn post_quit(&self, exit_code: i32) {
        {
            let mut state = self.state.lock().unwrap();
            state.quit = true;
            state.quit_code = exit_code;
        }
        self.wake.notify_all();
    }

    pub(crate) fn peek_message(&self, msg: &mut Msg, remove: bool) -> bool {
        let mut state = self.state.lock().unwrap();

        if state.quit {
            msg.message = window_message::WM_QUIT;
            msg.w_param = state.quit_code as usize;
            return true;
        }

        if state.messages.is_empty() {
            return false;
        }

        if remove {
            if let Some(front) = state.messages.pop_front() {
                *msg = front;
            } else {
                return false;
            }
        } else if let Some(front) = state.messages.front() {
            *msg = *front;
        }
        true
    }

    pub(crate) fn wait_for_wake(&self, timeout: Duration) {
        let state = self.state.lock().unwrap();
        if state.quit || !state.messages.is_empty() {
            return;
        }

        let _ = self.wake.wait_timeout(state, timeout).unwrap();
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    pub(crate) static THREAD_MESSAGE_QUEUE: MessageQueue = MessageQueue::new();
}
