use super::*;

/// Per-window state tracked by the user32 implementation.
#[derive(Debug, Clone)]
pub struct WindowState {
    pub hwnd: HWND,
    pub class_name: String,
    pub title: String,
    pub style: u32,
    pub ex_style: u32,
    pub rect: crate::windows::Rect,
    pub client_rect: crate::windows::Rect,
    pub parent: HWND,
    pub visible: bool,
    pub enabled: bool,
    pub wnd_proc: usize,
    pub user_data: usize,
    pub menu: Option<HMENU>,
    pub system_menu: Option<HMENU>,
}

/// Global window state manager.
pub struct WindowManager {
    windows: Mutex<HashMap<HWND, WindowState>>,
    control_ids: Mutex<HashMap<HWND, i32>>,
    next_hwnd: Mutex<usize>,
    focused_hwnd: Mutex<HWND>,
    foreground_hwnd: Mutex<HWND>,
    last_active_popup: Mutex<HashMap<HWND, HWND>>,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(HashMap::new()),
            control_ids: Mutex::new(HashMap::new()),
            next_hwnd: Mutex::new(0x10000),
            focused_hwnd: Mutex::new(HWND::NULL),
            foreground_hwnd: Mutex::new(HWND::NULL),
            last_active_popup: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_focus(&self) -> HWND {
        *self.focused_hwnd.lock().unwrap()
    }

    pub fn set_focus(&self, hwnd: HWND) -> HWND {
        let mut focused = self.focused_hwnd.lock().unwrap();
        let prev = *focused;
        *focused = hwnd;
        prev
    }

    pub fn get_foreground(&self) -> HWND {
        *self.foreground_hwnd.lock().unwrap()
    }

    pub fn set_foreground(&self, hwnd: HWND) -> HWND {
        let mut foreground = self.foreground_hwnd.lock().unwrap();
        let prev = *foreground;
        *foreground = hwnd;
        prev
    }

    pub fn create_window(&self, state: WindowState) -> HWND {
        let mut windows = self.windows.lock().unwrap();
        let mut next_hwnd = self.next_hwnd.lock().unwrap();

        let hwnd = HWND::from_raw(*next_hwnd);
        *next_hwnd += 1;

        let mut state = state;
        state.hwnd = hwnd;
        windows.insert(hwnd, state);

        hwnd
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<WindowState> {
        let windows = self.windows.lock().unwrap();
        windows.get(&hwnd).cloned()
    }

    pub fn update_window<F>(&self, hwnd: HWND, f: F) -> bool
    where
        F: FnOnce(&mut WindowState),
    {
        let mut windows = self.windows.lock().unwrap();
        if let Some(state) = windows.get_mut(&hwnd) {
            f(state);
            true
        } else {
            false
        }
    }

    pub fn destroy_window(&self, hwnd: HWND) -> bool {
        let mut windows = self.windows.lock().unwrap();
        let removed = windows.remove(&hwnd).is_some();
        drop(windows);

        if removed {
            let mut control_ids = self.control_ids.lock().unwrap();
            control_ids.remove(&hwnd);
        }

        removed
    }

    pub fn set_control_id(&self, hwnd: HWND, control_id: i32) -> bool {
        let windows = self.windows.lock().unwrap();
        if !windows.contains_key(&hwnd) {
            return false;
        }
        drop(windows);

        let mut control_ids = self.control_ids.lock().unwrap();
        control_ids.insert(hwnd, control_id);
        true
    }

    pub fn get_control_id(&self, hwnd: HWND) -> Option<i32> {
        let control_ids = self.control_ids.lock().unwrap();
        control_ids.get(&hwnd).copied()
    }

    pub fn find_child_by_control_id(&self, parent: HWND, control_id: i32) -> HWND {
        let windows = self.windows.lock().unwrap();
        let control_ids = self.control_ids.lock().unwrap();

        let mut found = HWND::NULL;
        for (&hwnd, state) in windows.iter() {
            if state.parent != parent {
                continue;
            }

            if control_ids.get(&hwnd).copied() != Some(control_id) {
                continue;
            }

            if found.is_null() || hwnd.as_raw() > found.as_raw() {
                found = hwnd;
            }
        }

        found
    }

    pub fn record_popup_activation(&self, popup: HWND) {
        if popup.is_null() {
            return;
        }
        let windows = self.windows.lock().unwrap();
        let owner = windows.get(&popup).map(|s| s.parent).unwrap_or(HWND::NULL);
        drop(windows);

        if !owner.is_null() {
            let mut map = self.last_active_popup.lock().unwrap();
            map.insert(owner, popup);
        }
    }

    pub fn get_last_active_popup(&self, owner: HWND) -> HWND {
        let map = self.last_active_popup.lock().unwrap();
        map.get(&owner).copied().unwrap_or(HWND::NULL)
    }

    pub fn child_window_from_point(&self, parent: HWND, pt: crate::windows::Point) -> HWND {
        let windows = self.windows.lock().unwrap();

        if !windows.contains_key(&parent) {
            return HWND::NULL;
        }

        let mut found: HWND = HWND::NULL;
        for (&hwnd, state) in windows.iter() {
            if state.parent != parent {
                continue;
            }
            if !state.visible || !state.enabled {
                continue;
            }
            if !state.rect.contains_point(pt.x, pt.y) {
                continue;
            }
            if found.is_null() || hwnd.as_raw() > found.as_raw() {
                found = hwnd;
            }
        }

        if found.is_null() { parent } else { found }
    }

    pub fn child_windows(&self, parent: HWND) -> Vec<WindowState> {
        let windows = self.windows.lock().unwrap();
        let mut out = Vec::new();

        for state in windows.values() {
            if state.parent == parent {
                out.push(state.clone());
            }
        }

        out
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_window_state(class_name: &str, parent: HWND) -> WindowState {
        WindowState {
            hwnd: HWND::NULL,
            class_name: class_name.to_string(),
            title: "title".to_string(),
            style: window_style::WS_OVERLAPPED,
            ex_style: 0,
            rect: crate::windows::Rect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            client_rect: crate::windows::Rect {
                left: 0,
                top: 0,
                right: 100,
                bottom: 100,
            },
            parent,
            visible: true,
            enabled: true,
            wnd_proc: 0,
            user_data: 0,
            menu: None,
            system_menu: None,
        }
    }

    #[test]
    fn window_manager_create_get_destroy() {
        let manager = WindowManager::new();
        let hwnd = manager.create_window(sample_window_state("Main", HWND::NULL));
        assert!(!hwnd.is_null());

        let state = manager.get_window(hwnd).expect("window should exist");
        assert_eq!(state.class_name, "Main");

        assert!(manager.destroy_window(hwnd));
        assert!(manager.get_window(hwnd).is_none());
    }

    #[test]
    fn window_manager_control_id_and_find_child() {
        let manager = WindowManager::new();
        let parent = manager.create_window(sample_window_state("Parent", HWND::NULL));
        let child_a = manager.create_window(sample_window_state("Child", parent));
        let child_b = manager.create_window(sample_window_state("Child", parent));

        assert!(manager.set_control_id(child_a, 100));
        assert!(manager.set_control_id(child_b, 100));

        let found = manager.find_child_by_control_id(parent, 100);
        assert_eq!(found, child_b);
    }

    #[test]
    fn window_manager_child_window_from_point() {
        let manager = WindowManager::new();
        let parent = manager.create_window(sample_window_state("Parent", HWND::NULL));
        let child = manager.create_window(sample_window_state("Child", parent));

        manager.update_window(child, |state| {
            state.rect = crate::windows::Rect {
                left: 10,
                top: 10,
                right: 20,
                bottom: 20,
            };
        });

        let hit = manager.child_window_from_point(parent, crate::windows::Point { x: 15, y: 15 });
        assert_eq!(hit, child);

        let miss = manager.child_window_from_point(parent, crate::windows::Point { x: 90, y: 90 });
        assert_eq!(miss, parent);
    }

    #[test]
    fn window_manager_child_windows_returns_immediate_children() {
        let manager = WindowManager::new();
        let parent = manager.create_window(sample_window_state("Parent", HWND::NULL));
        let child = manager.create_window(sample_window_state("Child", parent));
        let _other_parent_child = manager.create_window(sample_window_state("Other", HWND::NULL));

        let children = manager.child_windows(parent);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].hwnd, child);
    }
}
