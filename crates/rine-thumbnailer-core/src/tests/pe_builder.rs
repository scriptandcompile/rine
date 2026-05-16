//! Minimal synthetic PE builder for unit tests.
//!
//! Constructs a valid-enough PE32 binary containing a resource section with
//! one RT_GROUP_ICON entry pointing to one RT_ICON entry, so the extraction
//! pipeline can be exercised end-to-end without relying on real EXE fixtures.
//!
//! The generated PE is not executable; it is only structurally valid for the
//! purposes of resource-table parsing.

// ── Constants ────────────────────────────────────────────────────────────────

const IMAGE_FILE_MACHINE_I386: u16 = 0x014C;
const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
const IMAGE_NT_OPTIONAL_HDR32_MAGIC: u16 = 0x010B;
const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x0000_0040;
const IMAGE_SCN_MEM_READ: u32 = 0x4000_0000;

// ── Public builders ──────────────────────────────────────────────────────────

/// Build a minimal PE32 with a resource section containing exactly one icon
/// group referencing one RT_ICON entry with the given `icon_data` payload.
pub fn build_pe_with_single_icon(
    width: u8,
    height: u8,
    bit_count: u16,
    icon_data: &[u8],
) -> Vec<u8> {
    // The resource section layout (all offsets relative to section start):
    //
    // [0]   Root IMAGE_RESOURCE_DIRECTORY
    // [16]  Entry for RT_GROUP_ICON (type=14) → sub-dir at offset of L2_GRP
    // [24]  Entry for RT_ICON      (type=3)  → sub-dir at offset of L2_ICO
    // [32]  L2_GRP: directory for group icon name-level
    // [48]  L2_GRP entry id=1 → L3_GRP
    // [56]  L3_GRP: directory for language level
    // [72]  L3_GRP entry lang=0 → data entry for GRPICONDIR
    // [80]  L2_ICO: directory for RT_ICON name-level
    // [96]  L2_ICO entry id=1 → L3_ICO
    // [104] L3_ICO: directory for language level
    // [120] L3_ICO entry lang=0 → data entry for icon payload
    // [128] DATA_ENTRY for GRPICONDIR (IMAGE_RESOURCE_DATA_ENTRY)
    // [144] DATA_ENTRY for icon payload
    // [160] GRPICONDIR data
    // [160 + grp_size] icon payload data

    let grp_data = build_grpicondir(width, height, bit_count, icon_data.len() as u32, 1u16);
    let grp_size = grp_data.len();
    let ico_size = icon_data.len();

    const SECTION_ALIGN: usize = 0x1000;
    const FILE_ALIGN: usize = 0x200;

    // Resource section raw data: directory tree + raw data blobs
    let dir_tree_size = 160_usize;
    let raw_section_content_size =
        align_up(dir_tree_size + grp_size + ico_size, FILE_ALIGN);

    // Layout in the PE file:
    //   MZ header (64 bytes, with e_lfanew at offset 60)
    //   PE signature + COFF header + optional header + section table
    //   raw section content

    let dos_header_size = 64_usize;
    let pe_sig_size = 4_usize;
    let coff_size = 20_usize;
    // PE32 optional header: 28 standard + 68 windows-specific + 16×8 data dirs = 224
    let opt_size = 224_usize;
    let num_sections = 1_usize;
    let section_header_size = num_sections * 40;
    let headers_total =
        align_up(dos_header_size + pe_sig_size + coff_size + opt_size + section_header_size, FILE_ALIGN);

    let section_raw_offset = headers_total;
    let section_virtual_address = SECTION_ALIGN; // first section at 0x1000

    let rsrc_rva = section_virtual_address as u32;
    let image_size = align_up(section_virtual_address + raw_section_content_size, SECTION_ALIGN);

    // Build resource section bytes
    let mut rsrc = vec![0u8; raw_section_content_size];

    // Offsets within the resource directory tree:
    let off_root = 0_usize;
    let off_entry_grp = off_root + 16;  // root has 0 named, 2 id entries
    let off_entry_ico = off_entry_grp + 8;
    let off_l2_grp = off_entry_ico + 8; // = 32
    let off_l2_grp_entry = off_l2_grp + 16;
    let off_l3_grp = off_l2_grp_entry + 8; // = 56
    let off_l3_grp_entry = off_l3_grp + 16;
    let off_l2_ico = off_l3_grp_entry + 8; // = 80
    let off_l2_ico_entry = off_l2_ico + 16;
    let off_l3_ico = off_l2_ico_entry + 8; // = 104
    let off_l3_ico_entry = off_l3_ico + 16;
    let off_data_entry_grp = off_l3_ico_entry + 8; // = 128
    let off_data_entry_ico = off_data_entry_grp + 16; // = 144
    let off_grp_data = off_data_entry_ico + 16; // = 160
    let off_ico_data = off_grp_data + grp_size;

    // Root directory: 0 named, 2 id entries
    write_res_dir(&mut rsrc, off_root, 0, 2);
    // Root entry: type RT_GROUP_ICON=14 → subdirectory flag | off_l2_grp
    write_res_entry(&mut rsrc, off_entry_grp, 14, 0x8000_0000 | off_l2_grp as u32);
    // Root entry: type RT_ICON=3 → subdirectory flag | off_l2_ico
    write_res_entry(&mut rsrc, off_entry_ico, 3, 0x8000_0000 | off_l2_ico as u32);

    // L2 for group icon: 0 named, 1 id entry
    write_res_dir(&mut rsrc, off_l2_grp, 0, 1);
    write_res_entry(&mut rsrc, off_l2_grp_entry, 1, 0x8000_0000 | off_l3_grp as u32);

    // L3 for group icon: 0 named, 1 id entry (lang=0)
    write_res_dir(&mut rsrc, off_l3_grp, 0, 1);
    write_res_entry(&mut rsrc, off_l3_grp_entry, 0, off_data_entry_grp as u32);

    // L2 for RT_ICON: 0 named, 1 id entry
    write_res_dir(&mut rsrc, off_l2_ico, 0, 1);
    write_res_entry(&mut rsrc, off_l2_ico_entry, 1, 0x8000_0000 | off_l3_ico as u32);

    // L3 for RT_ICON: 0 named, 1 id entry (lang=0)
    write_res_dir(&mut rsrc, off_l3_ico, 0, 1);
    write_res_entry(&mut rsrc, off_l3_ico_entry, 0, off_data_entry_ico as u32);

    // Data entry for GRPICONDIR
    let grp_rva = rsrc_rva + off_grp_data as u32;
    write_data_entry(&mut rsrc, off_data_entry_grp, grp_rva, grp_size as u32);
    // Data entry for icon payload
    let ico_rva = rsrc_rva + off_ico_data as u32;
    write_data_entry(&mut rsrc, off_data_entry_ico, ico_rva, ico_size as u32);

    // Copy blobs
    rsrc[off_grp_data..off_grp_data + grp_size].copy_from_slice(&grp_data);
    rsrc[off_ico_data..off_ico_data + ico_size].copy_from_slice(icon_data);

    // Build full PE
    let total_size = section_raw_offset + raw_section_content_size;
    let mut pe = vec![0u8; total_size];

    // DOS header
    pe[0] = b'M';
    pe[1] = b'Z';
    write_u32_le(&mut pe, 60, dos_header_size as u32); // e_lfanew

    let pe_off = dos_header_size;
    // PE signature
    pe[pe_off..pe_off + 4].copy_from_slice(b"PE\0\0");

    // COFF header
    let coff_off = pe_off + 4;
    write_u16_le(&mut pe, coff_off, IMAGE_FILE_MACHINE_I386);
    write_u16_le(&mut pe, coff_off + 2, num_sections as u16);
    // timestamp (4), symbol table ptr (4), symbol count (4) = 0
    write_u16_le(&mut pe, coff_off + 16, opt_size as u16); // SizeOfOptionalHeader
    write_u16_le(&mut pe, coff_off + 18, IMAGE_FILE_EXECUTABLE_IMAGE);

    // Optional header (PE32)
    let opt_off = coff_off + coff_size;
    write_u16_le(&mut pe, opt_off, IMAGE_NT_OPTIONAL_HDR32_MAGIC);
    // MajorLinkerVersion, MinorLinkerVersion = 0
    // SizeOfCode = 0
    write_u32_le(&mut pe, opt_off + 16, 1); // AddressOfEntryPoint (non-zero)
    // ImageBase = 0x00400000
    write_u32_le(&mut pe, opt_off + 28, 0x0040_0000u32);
    // SectionAlignment
    write_u32_le(&mut pe, opt_off + 32, SECTION_ALIGN as u32);
    // FileAlignment
    write_u32_le(&mut pe, opt_off + 36, FILE_ALIGN as u32);
    // SizeOfImage
    write_u32_le(&mut pe, opt_off + 56, image_size as u32);
    // SizeOfHeaders
    write_u32_le(&mut pe, opt_off + 60, headers_total as u32);
    // NumberOfRvaAndSizes = 16
    write_u32_le(&mut pe, opt_off + 92, 16);

    // Data directory [2] = resource table (offset = 96 for standard + windows fields)
    // data_dirs start at opt_off + 96
    let dd_rsrc_off = opt_off + 96 + 2 * 8; // index 2
    write_u32_le(&mut pe, dd_rsrc_off, rsrc_rva); // VirtualAddress
    write_u32_le(&mut pe, dd_rsrc_off + 4, dir_tree_size as u32); // Size

    // Section header for .rsrc
    let sh_off = opt_off + opt_size;
    // Name: ".rsrc\0\0\0" (8 bytes)
    pe[sh_off..sh_off + 8].copy_from_slice(b".rsrc\0\0\0");
    write_u32_le(&mut pe, sh_off + 8, raw_section_content_size as u32); // VirtualSize
    write_u32_le(&mut pe, sh_off + 12, section_virtual_address as u32); // VirtualAddress
    write_u32_le(&mut pe, sh_off + 16, raw_section_content_size as u32); // SizeOfRawData
    write_u32_le(&mut pe, sh_off + 20, section_raw_offset as u32); // PointerToRawData
    write_u32_le(
        &mut pe,
        sh_off + 36,
        IMAGE_SCN_CNT_INITIALIZED_DATA | IMAGE_SCN_MEM_READ,
    ); // Characteristics

    // Copy resource section
    pe[section_raw_offset..section_raw_offset + rsrc.len()].copy_from_slice(&rsrc);

    pe
}

/// Build a minimal PE32 with no resource section (no RT_GROUP_ICON).
pub fn build_pe_no_resources() -> Vec<u8> {
    const FILE_ALIGN: usize = 0x200;

    let dos_header_size = 64_usize;
    let pe_sig_size = 4_usize;
    let coff_size = 20_usize;
    // PE32 optional header: 28 standard + 68 windows-specific + 16×8 data dirs = 224
    let opt_size = 224_usize;
    let headers_total = align_up(
        dos_header_size + pe_sig_size + coff_size + opt_size,
        FILE_ALIGN,
    );

    let mut pe = vec![0u8; headers_total];

    pe[0] = b'M';
    pe[1] = b'Z';
    write_u32_le(&mut pe, 60, dos_header_size as u32);

    let pe_off = dos_header_size;
    pe[pe_off..pe_off + 4].copy_from_slice(b"PE\0\0");

    let coff_off = pe_off + 4;
    write_u16_le(&mut pe, coff_off, IMAGE_FILE_MACHINE_I386);
    write_u16_le(&mut pe, coff_off + 2, 0u16); // no sections
    write_u16_le(&mut pe, coff_off + 16, opt_size as u16);
    write_u16_le(&mut pe, coff_off + 18, IMAGE_FILE_EXECUTABLE_IMAGE);

    let opt_off = coff_off + coff_size;
    write_u16_le(&mut pe, opt_off, IMAGE_NT_OPTIONAL_HDR32_MAGIC);
    write_u32_le(&mut pe, opt_off + 16, 1); // entry point
    write_u32_le(&mut pe, opt_off + 28, 0x0040_0000u32);
    write_u32_le(&mut pe, opt_off + 32, 0x1000u32);
    write_u32_le(&mut pe, opt_off + 36, FILE_ALIGN as u32);
    write_u32_le(&mut pe, opt_off + 56, 0x1000u32);
    write_u32_le(&mut pe, opt_off + 60, headers_total as u32);
    write_u32_le(&mut pe, opt_off + 92, 16);
    // resource data directory = all zeros → no resource section

    pe
}

// ── Utilities ────────────────────────────────────────────────────────────────

fn align_up(v: usize, align: usize) -> usize {
    (v + align - 1) & !(align - 1)
}

fn write_u16_le(buf: &mut [u8], off: usize, v: u16) {
    buf[off..off + 2].copy_from_slice(&v.to_le_bytes());
}

fn write_u32_le(buf: &mut [u8], off: usize, v: u32) {
    buf[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn write_res_dir(buf: &mut [u8], off: usize, num_named: u16, num_id: u16) {
    // IMAGE_RESOURCE_DIRECTORY (16 bytes): characteristics(4), timestamp(4),
    // majorVer(2), minorVer(2), namedEntries(2), idEntries(2)
    write_u16_le(buf, off + 12, num_named);
    write_u16_le(buf, off + 14, num_id);
}

fn write_res_entry(buf: &mut [u8], off: usize, name_or_id: u32, data_or_dir: u32) {
    write_u32_le(buf, off, name_or_id);
    write_u32_le(buf, off + 4, data_or_dir);
}

fn write_data_entry(buf: &mut [u8], off: usize, rva: u32, size: u32) {
    // IMAGE_RESOURCE_DATA_ENTRY: rva(4), size(4), codepage(4), reserved(4)
    write_u32_le(buf, off, rva);
    write_u32_le(buf, off + 4, size);
}

/// Build a GRPICONDIR with one entry referencing RT_ICON id=1.
fn build_grpicondir(
    width: u8,
    height: u8,
    bit_count: u16,
    bytes_in_res: u32,
    icon_id: u16,
) -> Vec<u8> {
    let mut buf = Vec::new();
    // GRPICONDIR header (6 bytes): reserved(2), type=1(2), count(2)
    buf.extend_from_slice(&0u16.to_le_bytes()); // reserved
    buf.extend_from_slice(&1u16.to_le_bytes()); // type = icon
    buf.extend_from_slice(&1u16.to_le_bytes()); // count = 1

    // GRPICONDIRENTRY (14 bytes): width(1), height(1), colorCount(1), reserved(1),
    //   planes(2), bitCount(2), bytesInRes(4), id(2)
    buf.push(width);
    buf.push(height);
    buf.push(0); // colorCount
    buf.push(0); // reserved
    buf.extend_from_slice(&1u16.to_le_bytes()); // planes
    buf.extend_from_slice(&bit_count.to_le_bytes());
    buf.extend_from_slice(&bytes_in_res.to_le_bytes());
    buf.extend_from_slice(&icon_id.to_le_bytes());
    buf
}
