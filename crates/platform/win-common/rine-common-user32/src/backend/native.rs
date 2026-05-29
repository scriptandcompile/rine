use std::collections::{HashMap, HashSet};
use std::time::Duration;

use rine_types::windows::*;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::platform::pump_events::EventLoopExtPumpEvents;
#[cfg(target_os = "linux")]
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowId};

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{
    ChangeWindowAttributesAux, ConnectionExt as XprotoConnectionExt, CreateGCAux, ImageFormat,
};

use crate::accelerator;
#[cfg(target_os = "linux")]
use crate::composition::compose_top_level_frame;

use super::{WindowBackend, debug_log_backend, user32_debug_enabled};

pub(super) struct NativeBackend {
    event_loop: EventLoop<()>,
    state: WinitBackendState,
}

struct WinitBackendState {
    windows: HashMap<HWND, Window>,
    reverse: HashMap<WindowId, HWND>,
    pending_windows: HashMap<HWND, PendingWindow>,
    pending_paint_messages: HashSet<HWND>,
    class_backgrounds: HashMap<HWND, ClassBackground>,
    last_unsolicited_clear: HashMap<HWND, std::time::Instant>,
    last_reported_size: HashMap<HWND, (u32, u32)>,
    last_reported_pos: HashMap<HWND, (i32, i32)>,
    ready_for_windows: bool,
}

#[derive(Clone, Copy)]
enum ClassBackground {
    Window,
}

struct PendingWindow {
    state: WindowState,
    redraw_requested: bool,
}

impl NativeBackend {
    pub(super) fn new() -> Self {
        Self {
            event_loop: EventLoop::new().expect("failed to initialize winit event loop"),
            state: WinitBackendState {
                windows: HashMap::new(),
                reverse: HashMap::new(),
                pending_windows: HashMap::new(),
                pending_paint_messages: HashSet::new(),
                class_backgrounds: HashMap::new(),
                last_unsolicited_clear: HashMap::new(),
                last_reported_size: HashMap::new(),
                last_reported_pos: HashMap::new(),
                // In pump mode, some platforms may not deliver `resumed()` before
                // we first need to materialize queued windows.
                ready_for_windows: true,
            },
        }
    }

    pub(super) fn create_window(&mut self, hwnd: HWND, state: &WindowState) -> Result<(), String> {
        if !should_create_native_host_window(state) {
            debug_log_backend(format!(
                "skip native host window for child hwnd={:#x}",
                hwnd.as_raw()
            ));
            return Ok(());
        }

        self.state.pending_windows.insert(
            hwnd,
            PendingWindow {
                state: state.clone(),
                redraw_requested: false,
            },
        );
        Ok(())
    }

    pub(super) fn destroy_window(&mut self, hwnd: HWND) {
        self.state.pending_windows.remove(&hwnd);
        self.state.pending_paint_messages.remove(&hwnd);
        self.state.class_backgrounds.remove(&hwnd);
        self.state.last_unsolicited_clear.remove(&hwnd);
        self.state.last_reported_size.remove(&hwnd);
        self.state.last_reported_pos.remove(&hwnd);
        if let Some(window) = self.state.windows.remove(&hwnd) {
            self.state.reverse.remove(&window.id());
            debug_log_backend(format!("destroy_window hwnd={:#x}", hwnd.as_raw()));
        }
    }

    pub(super) fn set_visible(&mut self, hwnd: HWND, visible: bool) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_visible(visible);
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.state.visible = visible;
        }
    }

    pub(super) fn set_title(&mut self, hwnd: HWND, title: &str) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_title(title);
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.state.title = title.to_owned();
        }
    }

    pub(super) fn request_redraw(&mut self, hwnd: HWND) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            self.state.pending_paint_messages.insert(hwnd);
            window.request_redraw();
        }
        if let Some(pending) = self.state.pending_windows.get_mut(&hwnd) {
            pending.redraw_requested = true;
        }
    }

    pub(super) fn present_window(&mut self, hwnd: HWND) {
        // Milestone 3 present hook: while software composition is being
        // introduced, presenting maps to the existing redraw path.
        self.request_redraw(hwnd);
    }

    pub(super) fn set_window_rect(&self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
        if let Some(window) = self.state.windows.get(&hwnd) {
            window.set_outer_position(LogicalPosition::new(x as f64, y as f64));
            let _ = window
                .request_inner_size(LogicalSize::new(width.max(1) as f64, height.max(1) as f64));
        }
    }

    pub(super) fn pump_messages(&mut self) -> Vec<Msg> {
        let mut out = Vec::new();
        let mut app = WinitPumpApp {
            state: &mut self.state,
            out: &mut out,
        };
        self.event_loop
            .pump_app_events(Some(Duration::ZERO), &mut app);

        out
    }
}

fn should_create_native_host_window(state: &WindowState) -> bool {
    (state.style & window_style::WS_CHILD) == 0
}

impl WindowBackend for NativeBackend {
    fn create_window(&mut self, hwnd: HWND, state: &WindowState) {
        let _ = NativeBackend::create_window(self, hwnd, state);
    }

    fn destroy_window(&mut self, hwnd: HWND) {
        NativeBackend::destroy_window(self, hwnd);
    }

    fn set_visible(&mut self, hwnd: HWND, visible: bool) {
        NativeBackend::set_visible(self, hwnd, visible);
    }

    fn set_title(&mut self, hwnd: HWND, title: &str) {
        NativeBackend::set_title(self, hwnd, title);
    }

    fn request_redraw(&mut self, hwnd: HWND) {
        NativeBackend::request_redraw(self, hwnd);
    }

    fn present_window(&mut self, hwnd: HWND) {
        NativeBackend::present_window(self, hwnd);
    }

    fn set_window_rect(&mut self, hwnd: HWND, x: i32, y: i32, width: i32, height: i32) {
        NativeBackend::set_window_rect(self, hwnd, x, y, width, height);
    }

    fn pump_messages(&mut self) -> Vec<Msg> {
        NativeBackend::pump_messages(self)
    }
}

impl WinitBackendState {
    fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        hwnd: HWND,
        state: &WindowState,
    ) -> Result<(), String> {
        let width = state.rect.right.saturating_sub(state.rect.left).max(1) as f64;
        let height = state.rect.bottom.saturating_sub(state.rect.top).max(1) as f64;

        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title(state.title.clone())
                    .with_visible(state.visible)
                    .with_inner_size(LogicalSize::new(width, height))
                    .with_position(LogicalPosition::new(
                        state.rect.left as f64,
                        state.rect.top as f64,
                    )),
            )
            .map_err(|e| format!("failed to create window: {e}"))?;

        let id = window.id();
        self.reverse.insert(id, hwnd);
        self.windows.insert(hwnd, window);

        if let Some(background) = class_background_for_name(&state.class_name) {
            self.class_backgrounds.insert(hwnd, background);
        }

        debug_log_backend(format!(
            "create_window hwnd={:#x} title=\"{}\" visible={}",
            hwnd.as_raw(),
            state.title,
            state.visible
        ));
        Ok(())
    }

    fn flush_pending_windows(&mut self, event_loop: &ActiveEventLoop) {
        if !self.ready_for_windows || self.pending_windows.is_empty() {
            return;
        }

        let pending = std::mem::take(&mut self.pending_windows);
        for (hwnd, pending_window) in pending {
            match self.create_window(event_loop, hwnd, &pending_window.state) {
                Ok(()) => {
                    if pending_window.redraw_requested
                        && let Some(window) = self.windows.get(&hwnd)
                    {
                        self.pending_paint_messages.insert(hwnd);
                        window.request_redraw();
                    }
                }
                Err(error) => {
                    eprintln!(
                        "warning: failed to create native user32 window {:#x}: {error}",
                        hwnd.as_raw()
                    );
                }
            }
        }
    }

    fn take_pending_paint(&mut self, hwnd: HWND) -> bool {
        self.pending_paint_messages.remove(&hwnd)
    }
}

struct WinitPumpApp<'a> {
    state: &'a mut WinitBackendState,
    out: &'a mut Vec<Msg>,
}

impl ApplicationHandler for WinitPumpApp<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state.ready_for_windows = true;
        self.state.flush_pending_windows(event_loop);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.state.ready_for_windows = false;
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.state.flush_pending_windows(event_loop);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(hwnd) = self.state.reverse.get(&window_id).copied() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                debug_log_backend(format!(
                    "winit CloseRequested hwnd={:#x} -> WM_CLOSE",
                    hwnd.as_raw()
                ));
                self.out.push(Msg {
                    hwnd,
                    message: window_message::WM_CLOSE,
                    w_param: 0,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                });
            }
            WindowEvent::Destroyed => self.out.push(Msg {
                hwnd,
                message: window_message::WM_DESTROY,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::Resized(size) => {
                let dims = (size.width, size.height);
                if self.state.last_reported_size.get(&hwnd) == Some(&dims) {
                    return;
                }
                self.state.last_reported_size.insert(hwnd, dims);

                let l_param = ((size.height as isize) << 16) | ((size.width as isize) & 0xFFFF);
                self.out.push(Msg {
                    hwnd,
                    message: window_message::WM_SIZE,
                    w_param: 0,
                    l_param,
                    time: 0,
                    pt: Point::default(),
                });
            }
            WindowEvent::Moved(pos) => {
                let coords = (pos.x, pos.y);
                if self.state.last_reported_pos.get(&hwnd) == Some(&coords) {
                    return;
                }
                self.state.last_reported_pos.insert(hwnd, coords);

                let x = (pos.x as i16) as u16 as usize;
                let y = (pos.y as i16) as u16 as usize;
                self.out.push(Msg {
                    hwnd,
                    message: window_message::WM_MOVE,
                    w_param: 0,
                    l_param: ((y << 16) | x) as isize,
                    time: 0,
                    pt: Point { x: pos.x, y: pos.y },
                });
            }
            WindowEvent::Focused(true) => self.out.push(Msg {
                hwnd,
                message: window_message::WM_SETFOCUS,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::Focused(false) => self.out.push(Msg {
                hwnd,
                message: window_message::WM_KILLFOCUS,
                w_param: 0,
                l_param: 0,
                time: 0,
                pt: Point::default(),
            }),
            WindowEvent::ModifiersChanged(modifiers) => {
                let state = modifiers.state();
                accelerator::update_modifier_state(
                    state.shift_key(),
                    state.control_key(),
                    state.alt_key(),
                );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state != ElementState::Pressed {
                    return;
                }

                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };

                let Some(vk) = key_code_to_virtual_key(code) else {
                    return;
                };

                accelerator::update_modifier_key(vk, true);

                let message = if accelerator::is_alt_down() {
                    window_message::WM_SYSKEYDOWN
                } else {
                    window_message::WM_KEYDOWN
                };

                self.out.push(Msg {
                    hwnd,
                    message,
                    w_param: vk as usize,
                    l_param: 0,
                    time: 0,
                    pt: Point::default(),
                });
            }
            WindowEvent::RedrawRequested => {
                if self.state.take_pending_paint(hwnd) {
                    if let Some(background) = self.state.class_backgrounds.get(&hwnd).copied()
                        && let Some(window) = self.state.windows.get(&hwnd)
                        && !present_composed_frame(hwnd, window)
                    {
                        paint_native_background(window, background);
                    }

                    self.out.push(Msg {
                        hwnd,
                        message: window_message::WM_PAINT,
                        w_param: 0,
                        l_param: 0,
                        time: 0,
                        pt: Point::default(),
                    });
                } else {
                    let should_clear = {
                        let now = std::time::Instant::now();
                        let interval = Duration::from_millis(30);
                        let allowed = self
                            .state
                            .last_unsolicited_clear
                            .get(&hwnd)
                            .map(|last| now.duration_since(*last) >= interval)
                            .unwrap_or(true);
                        if allowed {
                            self.state.last_unsolicited_clear.insert(hwnd, now);
                        }
                        allowed
                    };

                    if should_clear
                        && let Some(background) = self.state.class_backgrounds.get(&hwnd).copied()
                        && let Some(window) = self.state.windows.get(&hwnd)
                        && !present_composed_frame(hwnd, window)
                    {
                        paint_native_background(window, background);
                    }

                    if user32_debug_enabled() {
                        debug_log_backend(format!(
                            "unsolicited redraw hwnd={:#x} -> native background clear only (throttled={})",
                            hwnd.as_raw(),
                            !should_clear
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn key_code_to_virtual_key(code: KeyCode) -> Option<u16> {
    let vk = match code {
        KeyCode::Backspace => virtual_key::VK_BACK,
        KeyCode::Tab => virtual_key::VK_TAB,
        KeyCode::Enter => virtual_key::VK_RETURN,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => virtual_key::VK_SHIFT,
        KeyCode::ControlLeft | KeyCode::ControlRight => virtual_key::VK_CONTROL,
        KeyCode::AltLeft | KeyCode::AltRight => virtual_key::VK_MENU,
        KeyCode::Pause => virtual_key::VK_PAUSE,
        KeyCode::CapsLock => virtual_key::VK_CAPITAL,
        KeyCode::Escape => virtual_key::VK_ESCAPE,
        KeyCode::Space => virtual_key::VK_SPACE,
        KeyCode::PageUp => virtual_key::VK_PRIOR,
        KeyCode::PageDown => virtual_key::VK_NEXT,
        KeyCode::End => virtual_key::VK_END,
        KeyCode::Home => virtual_key::VK_HOME,
        KeyCode::ArrowLeft => virtual_key::VK_LEFT,
        KeyCode::ArrowUp => virtual_key::VK_UP,
        KeyCode::ArrowRight => virtual_key::VK_RIGHT,
        KeyCode::ArrowDown => virtual_key::VK_DOWN,
        KeyCode::Insert => virtual_key::VK_INSERT,
        KeyCode::Delete => virtual_key::VK_DELETE,
        KeyCode::Digit0 => virtual_key::VK_0,
        KeyCode::Digit1 => virtual_key::VK_1,
        KeyCode::Digit2 => virtual_key::VK_2,
        KeyCode::Digit3 => virtual_key::VK_3,
        KeyCode::Digit4 => virtual_key::VK_4,
        KeyCode::Digit5 => virtual_key::VK_5,
        KeyCode::Digit6 => virtual_key::VK_6,
        KeyCode::Digit7 => virtual_key::VK_7,
        KeyCode::Digit8 => virtual_key::VK_8,
        KeyCode::Digit9 => virtual_key::VK_9,
        KeyCode::KeyA => virtual_key::VK_A,
        KeyCode::KeyB => virtual_key::VK_B,
        KeyCode::KeyC => virtual_key::VK_C,
        KeyCode::KeyD => virtual_key::VK_D,
        KeyCode::KeyE => virtual_key::VK_E,
        KeyCode::KeyF => virtual_key::VK_F,
        KeyCode::KeyG => virtual_key::VK_G,
        KeyCode::KeyH => virtual_key::VK_H,
        KeyCode::KeyI => virtual_key::VK_I,
        KeyCode::KeyJ => virtual_key::VK_J,
        KeyCode::KeyK => virtual_key::VK_K,
        KeyCode::KeyL => virtual_key::VK_L,
        KeyCode::KeyM => virtual_key::VK_M,
        KeyCode::KeyN => virtual_key::VK_N,
        KeyCode::KeyO => virtual_key::VK_O,
        KeyCode::KeyP => virtual_key::VK_P,
        KeyCode::KeyQ => virtual_key::VK_Q,
        KeyCode::KeyR => virtual_key::VK_R,
        KeyCode::KeyS => virtual_key::VK_S,
        KeyCode::KeyT => virtual_key::VK_T,
        KeyCode::KeyU => virtual_key::VK_U,
        KeyCode::KeyV => virtual_key::VK_V,
        KeyCode::KeyW => virtual_key::VK_W,
        KeyCode::KeyX => virtual_key::VK_X,
        KeyCode::KeyY => virtual_key::VK_Y,
        KeyCode::KeyZ => virtual_key::VK_Z,
        KeyCode::F1 => virtual_key::VK_F1,
        KeyCode::F2 => virtual_key::VK_F2,
        KeyCode::F3 => virtual_key::VK_F3,
        KeyCode::F4 => virtual_key::VK_F4,
        KeyCode::F5 => virtual_key::VK_F5,
        KeyCode::F6 => virtual_key::VK_F6,
        KeyCode::F7 => virtual_key::VK_F7,
        KeyCode::F8 => virtual_key::VK_F8,
        KeyCode::F9 => virtual_key::VK_F9,
        KeyCode::F10 => virtual_key::VK_F10,
        KeyCode::F11 => virtual_key::VK_F11,
        KeyCode::F12 => virtual_key::VK_F12,
        _ => return None,
    };

    Some(vk)
}

fn class_background_for_name(class_name: &str) -> Option<ClassBackground> {
    let class = WINDOW_CLASS_REGISTRY.get(class_name)?;
    let sys_color_index = class.background.checked_sub(1)?;

    match sys_color_index {
        // COLOR_WINDOW
        5 => Some(ClassBackground::Window),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn paint_native_background(window: &Window, background: ClassBackground) {
    let Ok(window_handle) = window.window_handle() else {
        return;
    };

    let x11_window = match window_handle.as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window as u32,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        _ => return,
    };

    if x11_window == 0 {
        return;
    }

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return;
    };

    let screen = &conn.setup().roots[screen_num];
    let pixel = match background {
        ClassBackground::Window => screen.white_pixel,
    };

    if user32_debug_enabled() {
        debug_log_backend(format!(
            "native background clear via X11 pixel={:#x}",
            pixel
        ));
    }

    let attributes = ChangeWindowAttributesAux::new().background_pixel(pixel);

    let _ = conn.change_window_attributes(x11_window, &attributes);
    let _ = conn.clear_area(false, x11_window, 0, 0, 0, 0);
    let _ = conn.flush();
}

#[cfg(target_os = "linux")]
fn present_composed_frame(parent: HWND, window: &Window) -> bool {
    let Some(frame) = compose_top_level_frame(parent) else {
        return false;
    };

    let Ok(window_handle) = window.window_handle() else {
        return false;
    };

    let x11_window = match window_handle.as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window as u32,
        RawWindowHandle::Xcb(handle) => handle.window.get(),
        _ => return false,
    };

    if x11_window == 0 {
        return false;
    }

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return false;
    };

    let screen = &conn.setup().roots[screen_num];
    let Ok(width) = u16::try_from(frame.width) else {
        return false;
    };
    let Ok(height) = u16::try_from(frame.height) else {
        return false;
    };

    let Ok(gc) = conn.generate_id() else {
        return false;
    };

    let _ = conn.create_gc(gc, x11_window, &CreateGCAux::new());

    let mut image = Vec::with_capacity(frame.pixels_argb.len() * 4);
    for pixel in frame.pixels_argb {
        let a = ((pixel >> 24) & 0xFF) as u8;
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        image.extend_from_slice(&[b, g, r, a]);
    }

    let put_ok = conn
        .put_image(
            ImageFormat::Z_PIXMAP,
            x11_window,
            gc,
            width,
            height,
            0,
            0,
            0,
            screen.root_depth,
            &image,
        )
        .is_ok();

    let _ = conn.free_gc(gc);
    let _ = conn.flush();
    put_ok
}

#[cfg(not(target_os = "linux"))]
fn paint_native_background(_window: &Window, _background: ClassBackground) {}

#[cfg(not(target_os = "linux"))]
fn present_composed_frame(_parent: HWND, _window: &Window) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rine_types::windows::{Rect, WindowState, window_style};
    use winit::keyboard::KeyCode;

    fn test_window_state(style: u32) -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: "test".to_owned(),
            title: "test".to_owned(),
            style,
            ex_style: 0,
            rect: Rect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            client_rect: Rect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            parent: HWND::NULL,
            visible: true,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
            menu: None,
            system_menu: None,
        }
    }

    #[test]
    fn child_windows_do_not_create_native_host_windows() {
        let state = test_window_state(window_style::WS_CHILD);
        assert!(!should_create_native_host_window(&state));
    }

    #[test]
    fn top_level_windows_create_native_host_windows() {
        let state = test_window_state(window_style::WS_OVERLAPPEDWINDOW);
        assert!(should_create_native_host_window(&state));
    }

    #[test]
    fn maps_backspace() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Backspace), Some(0x08));
    }

    #[test]
    fn maps_tab() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Tab), Some(0x09));
    }

    #[test]
    fn maps_enter() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Enter), Some(0x0D));
    }

    #[test]
    fn maps_escape() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Escape), Some(0x1B));
    }

    #[test]
    fn maps_space() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Space), Some(0x20));
    }

    #[test]
    fn maps_caps_lock() {
        assert_eq!(key_code_to_virtual_key(KeyCode::CapsLock), Some(0x14));
    }

    #[test]
    fn maps_modifier_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::ShiftLeft), Some(0x10));
        assert_eq!(key_code_to_virtual_key(KeyCode::ShiftRight), Some(0x10));
        assert_eq!(key_code_to_virtual_key(KeyCode::ControlLeft), Some(0x11));
        assert_eq!(key_code_to_virtual_key(KeyCode::ControlRight), Some(0x11));
        assert_eq!(key_code_to_virtual_key(KeyCode::AltLeft), Some(0x12));
        assert_eq!(key_code_to_virtual_key(KeyCode::AltRight), Some(0x12));
    }

    #[test]
    fn maps_navigation_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::PageUp), Some(0x21));
        assert_eq!(key_code_to_virtual_key(KeyCode::PageDown), Some(0x22));
        assert_eq!(key_code_to_virtual_key(KeyCode::End), Some(0x23));
        assert_eq!(key_code_to_virtual_key(KeyCode::Home), Some(0x24));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowLeft), Some(0x25));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowUp), Some(0x26));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowRight), Some(0x27));
        assert_eq!(key_code_to_virtual_key(KeyCode::ArrowDown), Some(0x28));
    }

    #[test]
    fn maps_insert_delete() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Insert), Some(0x2D));
        assert_eq!(key_code_to_virtual_key(KeyCode::Delete), Some(0x2E));
    }

    #[test]
    fn maps_pause() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Pause), Some(0x13));
    }

    #[test]
    fn maps_digit_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit0), Some(0x30));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit1), Some(0x31));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit2), Some(0x32));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit3), Some(0x33));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit4), Some(0x34));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit5), Some(0x35));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit6), Some(0x36));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit7), Some(0x37));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit8), Some(0x38));
        assert_eq!(key_code_to_virtual_key(KeyCode::Digit9), Some(0x39));
    }

    #[test]
    fn maps_letter_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyA), Some(0x41));
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyB), Some(0x42));
        assert_eq!(key_code_to_virtual_key(KeyCode::KeyZ), Some(0x5A));
    }

    #[test]
    fn maps_function_keys() {
        assert_eq!(key_code_to_virtual_key(KeyCode::F1), Some(0x70));
        assert_eq!(key_code_to_virtual_key(KeyCode::F2), Some(0x71));
        assert_eq!(key_code_to_virtual_key(KeyCode::F3), Some(0x72));
        assert_eq!(key_code_to_virtual_key(KeyCode::F4), Some(0x73));
        assert_eq!(key_code_to_virtual_key(KeyCode::F5), Some(0x74));
        assert_eq!(key_code_to_virtual_key(KeyCode::F6), Some(0x75));
        assert_eq!(key_code_to_virtual_key(KeyCode::F7), Some(0x76));
        assert_eq!(key_code_to_virtual_key(KeyCode::F8), Some(0x77));
        assert_eq!(key_code_to_virtual_key(KeyCode::F9), Some(0x78));
        assert_eq!(key_code_to_virtual_key(KeyCode::F10), Some(0x79));
        assert_eq!(key_code_to_virtual_key(KeyCode::F11), Some(0x7A));
        assert_eq!(key_code_to_virtual_key(KeyCode::F12), Some(0x7B));
    }

    #[test]
    fn unmapped_key_returns_none() {
        assert_eq!(key_code_to_virtual_key(KeyCode::F13), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::AudioVolumeUp), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::BrowserHome), None);
        assert_eq!(key_code_to_virtual_key(KeyCode::MediaPlayPause), None);
    }
}
