use rine_dlls::{DllPlugin, Export};

pub struct Advapi32Plugin;

impl DllPlugin for Advapi32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["advapi32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }
}
