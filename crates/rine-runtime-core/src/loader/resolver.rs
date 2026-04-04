//! Import resolution - resolves PE imports against rine-dlls Rust implementations
//! and writes function pointers into the loaded image's Import Address Table (IAT).

use std::ptr;

use goblin::pe::PE;
use goblin::pe::import::SyntheticImportLookupTableEntry;
use rine_dlls::{DllRegistry, LookupResult};
use rine_types::memory::{RelativeVirtualAddress, VirtualAddress};
use thiserror::Error;
use tracing::{debug, info, warn};

use super::memory::LoadedImage;
use crate::pe::parser::PeFormat;

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("unimplemented imports detected: {imports:?}")]
    UnimplementedImports { imports: Vec<String> },

    #[error("IAT write at {va} is outside the loaded image bounds")]
    IatOutOfBounds { va: VirtualAddress },

    #[error("no import data found in PE")]
    NoImportData,

    #[error("failed to make IAT section writable: {0}")]
    MprotectFailed(#[from] nix::Error),
}

/// Summary of import resolution for one DLL.
#[derive(Debug)]
pub struct DllResolutionSummary {
    pub dll_name: String,
    pub resolved: usize,
    pub stubbed: usize,
    pub resolved_names: Vec<String>,
    pub stubbed_names: Vec<String>,
}

/// Summary of the entire import resolution pass.
#[derive(Debug)]
pub struct ResolutionReport {
    pub dll_summaries: Vec<DllResolutionSummary>,
    pub total_resolved: usize,
    pub total_stubbed: usize,
}

/// Resolve all imports in a loaded PE image, writing function pointers into the IAT.
///
/// This function:
/// 1. Iterates the PE's import directory entries
/// 2. For each imported DLL, looks up each function by name or ordinal in the `DllRegistry`
/// 3. Writes the resolved function pointer (or stub) into the IAT slot in mapped memory
///
/// The IAT must be writable when this is called. Typically this runs before
/// `set_section_protections` finalizes memory permissions, or the caller
/// temporarily makes the IAT writable.
pub fn resolve_imports(
    image: &LoadedImage,
    pe: &PE,
    pe_format: PeFormat,
    registry: &DllRegistry,
) -> Result<ResolutionReport, ResolverError> {
    let import_data = pe.import_data.as_ref().ok_or(ResolverError::NoImportData)?;

    let image_size = image.size();
    let base = image.base();
    let mut report = ResolutionReport {
        dll_summaries: Vec::new(),
        total_resolved: 0,
        total_stubbed: 0,
    };
    let mut unimplemented_imports = Vec::new();

    for entry in &import_data.import_data {
        let dll_name = entry.name;
        let iat_rva = entry.import_directory_entry.import_address_table_rva;

        debug!(
            dll = dll_name,
            iat_rva = format_args!("{iat_rva:#x}"),
            "resolving imports"
        );

        let mut summary = DllResolutionSummary {
            dll_name: dll_name.to_string(),
            resolved: 0,
            stubbed: 0,
            resolved_names: Vec::new(),
            stubbed_names: Vec::new(),
        };

        if !registry.has_dll(dll_name) {
            warn!(dll = dll_name, "unknown DLL - all imports will be stubbed");
        }

        let lookup_table = match &entry.import_lookup_table {
            Some(table) => table,
            None => {
                warn!(dll = dll_name, "no import lookup table, skipping");
                continue;
            }
        };

        let entry_size = iat_entry_size(pe_format);

        for (i, lookup_entry) in lookup_table.iter().enumerate() {
            let iat_slot_rva = RelativeVirtualAddress::new(iat_rva + (i as u32) * entry_size);
            let iat_slot_va = base.offset(iat_slot_rva);

            // Bounds check: ensure the entry write is within the image.
            if iat_slot_rva.as_usize() + entry_size as usize > image_size {
                return Err(ResolverError::IatOutOfBounds { va: iat_slot_va });
            }

            let (func_name, result) = match lookup_entry {
                SyntheticImportLookupTableEntry::HintNameTableRVA((_rva, hint_entry)) => {
                    let name = hint_entry.name;
                    let result = registry.resolve_by_name(dll_name, name);
                    (name.to_string(), result)
                }
                SyntheticImportLookupTableEntry::OrdinalNumber(ordinal) => {
                    let result = registry.resolve_by_ordinal(dll_name, *ordinal);
                    (format!("#{ordinal}"), result)
                }
            };

            match result {
                LookupResult::Found(func) => {
                    debug!(
                        dll = dll_name,
                        func = func_name,
                        addr = format_args!("{iat_slot_va}"),
                        "resolved import"
                    );
                    summary.resolved += 1;
                    summary.resolved_names.push(func_name);
                    write_iat_entry(iat_slot_va, func as usize, pe_format);
                }
                LookupResult::Stub(func) => {
                    debug!(
                        dll = dll_name,
                        func = func_name,
                        addr = format_args!("{iat_slot_va}"),
                        "unimplemented import"
                    );
                    summary.stubbed += 1;
                    summary.stubbed_names.push(func_name.clone());
                    unimplemented_imports.push(format!("{dll_name}!{func_name}"));
                    write_iat_entry(iat_slot_va, func as usize, pe_format);
                }
            }
        }

        info!(
            dll = dll_name,
            resolved = summary.resolved,
            stubbed = summary.stubbed,
            "import resolution complete"
        );

        report.total_resolved += summary.resolved;
        report.total_stubbed += summary.stubbed;
        report.dll_summaries.push(summary);
    }

    if !unimplemented_imports.is_empty() {
        return Err(ResolverError::UnimplementedImports {
            imports: unimplemented_imports,
        });
    }

    info!(
        total_resolved = report.total_resolved,
        total_stubbed = report.total_stubbed,
        dlls = report.dll_summaries.len(),
        "all imports resolved"
    );

    Ok(report)
}

/// Resolve delay-loaded imports, if any.
///
/// Delay-load imports are similar to regular imports but use a separate
/// directory. The PE calls a helper (`__delayLoadHelper2`) that resolves
/// the import on first use. We pre-resolve them at load time instead,
/// which is simpler and avoids needing the delay-load helper stub.
pub fn resolve_delay_imports(
    _image: &LoadedImage,
    pe: &PE,
    _registry: &DllRegistry,
) -> Result<ResolutionReport, ResolverError> {
    // goblin 0.10 does not expose a fully parsed delay-load import table
    // in the same way as regular imports. We check the data directory entry
    // and, if present, log a warning. Full delay-load support will be added
    // when goblin exposes the parsed data or we parse it manually.
    if let Some(ref optional_header) = pe.header.optional_header
        && let Some(delay_dd) = optional_header
            .data_directories
            .get_delay_import_descriptor()
        && delay_dd.virtual_address != 0
    {
        warn!(
            rva = format_args!("{:#x}", delay_dd.virtual_address),
            size = format_args!("{:#x}", delay_dd.size),
            "PE has delay-load imports - not yet resolved (will be resolved on demand)"
        );
    }

    Ok(ResolutionReport {
        dll_summaries: Vec::new(),
        total_resolved: 0,
        total_stubbed: 0,
    })
}

/// Write a function pointer into an IAT slot in the loaded image.
///
/// # Safety
/// The caller must ensure `va` points to a valid, writable IAT slot
/// (4 bytes for PE32, 8 bytes for PE32+) within the loaded image's memory region.
fn write_iat_entry(va: VirtualAddress, func_addr: usize, pe_format: PeFormat) {
    match pe_format {
        PeFormat::Pe32Plus => unsafe {
            let slot = va.as_mut_ptr() as *mut u64;
            ptr::write_unaligned(slot, func_addr as u64);
        },
        PeFormat::Pe32 => unsafe {
            let slot = va.as_mut_ptr() as *mut u32;
            ptr::write_unaligned(slot, func_addr as u32);
        },
    }
}

fn iat_entry_size(pe_format: PeFormat) -> u32 {
    match pe_format {
        PeFormat::Pe32 => 4,
        PeFormat::Pe32Plus => 8,
    }
}
