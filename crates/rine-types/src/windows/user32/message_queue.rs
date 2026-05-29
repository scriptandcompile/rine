use super::*;

/// A message queue helper for a single thread.
///
/// Runtime queue ownership lives in the shared user32 layer
/// (`rine-common-user32`), which provides the canonical thread queue used by
/// GetMessage/PeekMessage flows.
#[derive(Debug)]
pub struct MessageQueue {
    state: Mutex<MessageQueueState>,
    wake: Condvar,
}

#[derive(Debug, Default)]
struct MessageQueueState {
    messages: Vec<Msg>,
    quit: bool,
    quit_code: i32,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MessageQueueState::default()),
            wake: Condvar::new(),
        }
    }

    pub fn post_message(&self, msg: Msg) {
        {
            let mut state = self.state.lock().unwrap();
            state.messages.push(msg);
        }
        self.wake.notify_one();
    }

    pub fn post_quit(&self, exit_code: i32) {
        {
            let mut state = self.state.lock().unwrap();
            state.quit = true;
            state.quit_code = exit_code;
        }
        self.wake.notify_all();
    }

    pub fn get_message(&self, msg: &mut Msg) -> bool {
        let mut state = self.state.lock().unwrap();
        loop {
            if state.quit {
                msg.message = window_message::WM_QUIT;
                msg.w_param = state.quit_code as usize;
                return false;
            }

            if !state.messages.is_empty() {
                *msg = state.messages.remove(0);
                return true;
            }

            state = self.wake.wait(state).unwrap();
        }
    }

    pub fn peek_message(&self, msg: &mut Msg, remove: bool) -> bool {
        let mut state = self.state.lock().unwrap();

        if state.quit {
            msg.message = window_message::WM_QUIT;
            msg.w_param = state.quit_code as usize;
            return true;
        }

        if state.messages.is_empty() {
            return false;
        }

        *msg = state.messages[0];
        if remove {
            state.messages.remove(0);
        }
        true
    }
}

impl Default for MessageQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_msg() -> Msg {
        Msg {
            hwnd: HWND::NULL,
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            pt: crate::windows::Point { x: 0, y: 0 },
        }
    }

    #[test]
    fn message_queue_post_and_get() {
        let queue = MessageQueue::new();

        let msg = Msg {
            hwnd: HWND::from_raw(0x1234),
            message: window_message::WM_USER,
            w_param: 1,
            l_param: 2,
            time: 0,
            pt: crate::windows::Point { x: 10, y: 20 },
        };

        queue.post_message(msg);

        let mut out = empty_msg();
        assert!(queue.get_message(&mut out));
        assert_eq!(out.hwnd, msg.hwnd);
        assert_eq!(out.message, msg.message);
        assert_eq!(out.w_param, msg.w_param);
        assert_eq!(out.l_param, msg.l_param);
    }

    #[test]
    fn message_queue_peek_remove_and_quit() {
        let queue = MessageQueue::new();

        let msg = Msg {
            hwnd: HWND::from_raw(0x7777),
            message: window_message::WM_APP,
            w_param: 3,
            l_param: 4,
            time: 0,
            pt: crate::windows::Point { x: 1, y: 2 },
        };

        queue.post_message(msg);

        let mut out = empty_msg();
        assert!(queue.peek_message(&mut out, false));
        assert_eq!(out.message, window_message::WM_APP);

        let mut out2 = empty_msg();
        assert!(queue.peek_message(&mut out2, true));
        assert_eq!(out2.message, window_message::WM_APP);

        let mut out3 = empty_msg();
        assert!(!queue.peek_message(&mut out3, false));

        queue.post_quit(42);
        let mut quit = empty_msg();
        assert!(!queue.get_message(&mut quit));
        assert_eq!(quit.message, window_message::WM_QUIT);
        assert_eq!(quit.w_param, 42);
    }
}
