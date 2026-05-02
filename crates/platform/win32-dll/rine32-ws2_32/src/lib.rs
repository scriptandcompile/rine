use rine_dlls::DllPlugin;

pub struct Ws2_32Plugin;

impl DllPlugin for Ws2_32Plugin {
    fn dll_names(&self) -> &[&str] {
        &["ws2_32.dll"]
    }

    fn exports(&self) -> Vec<rine_dlls::Export> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated.rs"))
    }

    fn stubs(&self) -> Vec<rine_dlls::StubExport> {
        include!(concat!(env!("OUT_DIR"), "/dll_plugin_generated_stubs.rs"))
    }

    fn partials(&self) -> Vec<rine_dlls::PartialExport> {
        include!(concat!(
            env!("OUT_DIR"),
            "/dll_plugin_generated_partials.rs"
        ))
    }
}

rine_dlls::export_dynamic_provider!(|| Ws2_32Plugin);
