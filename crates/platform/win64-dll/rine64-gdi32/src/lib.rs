use rine_dlls::{DllPlugin, Export};

pub struct Gdi32Plugin;

impl DllPlugin for Gdi32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["gdi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }
}
