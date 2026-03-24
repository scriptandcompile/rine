use rine_dlls::{DllPlugin, Export};

pub struct Ws2_32Plugin;

impl DllPlugin for Ws2_32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["ws2_32.dll"]
    }

    fn exports(&self) -> Vec<Export> {
        vec![]
    }
}
