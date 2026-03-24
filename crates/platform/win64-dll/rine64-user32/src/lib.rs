use rine_dlls::{DllPlugin, Export};

pub struct User32Plugin;

impl DllPlugin for User32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["user32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }
}
