use super::*;

/// A registered window class.
#[derive(Debug, Clone)]
pub struct WindowClass {
    pub name: String,
    pub style: u32,
    pub wnd_proc: usize,
    pub cls_extra: i32,
    pub wnd_extra: i32,
    pub instance: usize,
    pub icon: usize,
    pub cursor: usize,
    pub background: usize,
    pub menu_name: Option<String>,
    pub icon_sm: usize,
}

/// Global registry of window classes.
pub struct WindowClassRegistry {
    classes: Mutex<HashMap<String, WindowClass>>,
}

impl WindowClassRegistry {
    pub fn new() -> Self {
        Self {
            classes: Mutex::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: String, class: WindowClass) {
        let mut classes = self.classes.lock().unwrap();
        classes.insert(name, class);
    }

    pub fn get(&self, name: &str) -> Option<WindowClass> {
        let classes = self.classes.lock().unwrap();
        classes.get(name).cloned()
    }

    pub fn unregister(&self, name: &str) -> bool {
        let mut classes = self.classes.lock().unwrap();
        classes.remove(name).is_some()
    }
}

impl Default for WindowClassRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_class(name: &str) -> WindowClass {
        WindowClass {
            name: name.to_string(),
            style: class_style::CS_VREDRAW,
            wnd_proc: 0,
            cls_extra: 0,
            wnd_extra: 0,
            instance: 0,
            icon: 0,
            cursor: 0,
            background: 0,
            menu_name: None,
            icon_sm: 0,
        }
    }

    #[test]
    fn registry_register_get_unregister() {
        let registry = WindowClassRegistry::new();
        let class = sample_class("MyClass");

        registry.register("MyClass".to_string(), class.clone());
        let retrieved = registry.get("MyClass").expect("class should exist");
        assert_eq!(retrieved.name, "MyClass");
        assert_eq!(retrieved.style, class.style);

        assert!(registry.unregister("MyClass"));
        assert!(registry.get("MyClass").is_none());
    }
}
