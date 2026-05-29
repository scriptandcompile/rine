use rine_types::windows::*;

pub(super) trait WindowBackend {
    fn create_window(&mut self, hwnd: HWND, state: &WindowState);
    fn destroy_window(&mut self, hwnd: HWND);
    fn set_visible(&mut self, hwnd: HWND, visible: bool);
    fn set_title(&mut self, hwnd: HWND, title: &str);
    fn request_redraw(&mut self, hwnd: HWND);
    fn set_window_rect(&mut self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32);
    fn pump_messages(&mut self) -> Vec<Msg>;
}
