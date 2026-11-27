//! I324 D2 candidate: enumerate declared PE32+ DLL names from supplied bytes.
//! A supplied-image graph records declared edges, not a Windows loader closure or attestation.
//! The parser uses no filesystem, environment, process, unsafe code or authority.
//! The ignored package-fixture adapter reads only its explicit test input.

use std::collections::{BTreeMap, BTreeSet};

const MAX_IMAGE: usize = 64 * 1024 * 1024;
const MAX_SECTIONS: usize = 96;
const MAX_IMPORTS: usize = 256;
const MAX_NAME: usize = 128;

#[derive(Debug, PartialEq, Eq)]
enum PeError {
    Unsupported,
    Malformed,
    Budget,
}

type Result<T> = std::result::Result<T, PeError>;

#[derive(Debug, PartialEq, Eq)]
struct DeclaredImports {
    ordinary: BTreeSet<String>,
    delayed: BTreeSet<String>,
}

struct Section {
    rva: u32,
    virtual_size: u32,
    raw_start: usize,
    raw_size: u32,
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16> {
    let end = offset.checked_add(2).ok_or(PeError::Malformed)?;
    let raw = bytes.get(offset..end).ok_or(PeError::Malformed)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset.checked_add(4).ok_or(PeError::Malformed)?;
    let raw = bytes.get(offset..end).ok_or(PeError::Malformed)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

// Return only the contiguous, file-backed part of exactly one section.
// Header RVAs, overlapping ranges and zero-filled virtual tails refuse.
fn at_rva<'a>(bytes: &'a [u8], sections: &[Section], rva: u32) -> Result<&'a [u8]> {
    let mut found = None;
    for section in sections {
        let span = section.virtual_size.max(section.raw_size);
        let end = section.rva.checked_add(span).ok_or(PeError::Malformed)?;
        if rva < section.rva || rva >= end {
            continue;
        }
        if found.is_some() {
            return Err(PeError::Malformed);
        }
        let delta = rva - section.rva;
        if delta >= section.raw_size || delta >= section.virtual_size {
            return Err(PeError::Malformed);
        }
        let start = section
            .raw_start
            .checked_add(delta as usize)
            .ok_or(PeError::Malformed)?;
        let len = (section.raw_size.min(section.virtual_size) - delta) as usize;
        let end = start.checked_add(len).ok_or(PeError::Malformed)?;
        found = Some(bytes.get(start..end).ok_or(PeError::Malformed)?);
    }
    found.ok_or(PeError::Malformed)
}

fn dll_name(bytes: &[u8], sections: &[Section], rva: u32) -> Result<String> {
    let data = at_rva(bytes, sections, rva)?;
    let prefix = &data[..data.len().min(MAX_NAME + 1)];
    let end = prefix
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(PeError::Malformed)?;
    let name = &prefix[..end];
    // Narrow ordinary basename grammar only. No name is a path or authority.
    if name.len() <= 4
        || !name
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'-' | b'_' | b'.'))
        || name.starts_with(b".")
        || name.windows(2).any(|pair| pair == b"..")
    {
        return Err(PeError::Unsupported);
    }
    let lower = std::str::from_utf8(name)
        .map_err(|_| PeError::Unsupported)?
        .to_ascii_lowercase();
    if !lower.ends_with(".dll") {
        return Err(PeError::Unsupported);
    }
    Ok(lower)
}

fn directory(
    bytes: &[u8],
    sections: &[Section],
    rva: u32,
    size: u32,
    delayed: bool,
) -> Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    if rva == 0 && size == 0 {
        return Ok(names);
    }
    if rva == 0 || size == 0 {
        return Err(PeError::Malformed);
    }
    let stride = if delayed { 32 } else { 20 };
    let size = size as usize;
    if size > 1024 * 1024 {
        return Err(PeError::Budget);
    }
    let data = at_rva(bytes, sections, rva)?;
    let table = data.get(..size).ok_or(PeError::Malformed)?;
    // PE ordinary imports can declare a span covering more than the
    // descriptor array (e.g. GNU BFD's .idata directory). A referenced,
    // file-backed lookup/address table or DLL name bounds the descriptor
    // prefix. It is not another descriptor, nor proof of loader identity.
    let mut descriptor_limit = size;
    for (index, descriptor) in table.chunks_exact(stride).enumerate() {
        let descriptor_end = (index + 1) * stride;
        if descriptor_end > descriptor_limit {
            return Err(PeError::Malformed);
        }
        if descriptor.iter().all(|byte| *byte == 0) {
            if descriptor_limit == size && !size.is_multiple_of(stride) {
                return Err(PeError::Malformed);
            }
            let trailing = &table[descriptor_end..descriptor_limit];
            if trailing.iter().any(|byte| *byte != 0) {
                return Err(PeError::Malformed);
            }
            return Ok(names);
        }
        if index >= MAX_IMPORTS {
            return Err(PeError::Budget);
        }
        if delayed && read_u32(descriptor, 0)? != 1 {
            // Only RVA-based delay imports are understood.
            return Err(PeError::Unsupported);
        }
        let name_rva = read_u32(descriptor, if delayed { 4 } else { 12 })?;
        if name_rva == 0 {
            return Err(PeError::Malformed);
        }
        names.insert(dll_name(bytes, sections, name_rva)?);
        if !delayed {
            for (field, minimum) in [(0, 8), (16, 8), (12, 1)] {
                let referenced = read_u32(descriptor, field)?;
                if referenced == 0 {
                    continue;
                }
                if let Some(delta) = referenced.checked_sub(rva) {
                    let delta = delta as usize;
                    if delta < size {
                        let data = at_rva(bytes, sections, referenced)?;
                        if data.len() < minimum {
                            return Err(PeError::Malformed);
                        }
                        descriptor_limit = descriptor_limit.min(delta);
                    }
                }
            }
            if descriptor_end > descriptor_limit {
                return Err(PeError::Malformed);
            }
        }
    }
    Err(PeError::Malformed)
}

fn declared_imports(bytes: &[u8]) -> Result<DeclaredImports> {
    if bytes.len() > MAX_IMAGE {
        return Err(PeError::Budget);
    }
    if !bytes.starts_with(b"MZ") {
        return Err(PeError::Unsupported);
    }
    let pe = read_u32(bytes, 0x3c)? as usize;
    // The input-size bound also keeps these fixed header offsets in range.
    if pe > bytes.len() || bytes.get(pe..pe + 4) != Some(b"PE\0\0") {
        return Err(PeError::Malformed);
    }
    if read_u16(bytes, pe + 4)? != 0x8664 {
        return Err(PeError::Unsupported);
    }
    let count = read_u16(bytes, pe + 6)? as usize;
    if count == 0 || count > MAX_SECTIONS {
        return Err(PeError::Budget);
    }
    let optional_size = read_u16(bytes, pe + 20)? as usize;
    let optional = pe + 24;
    let section_table = optional + optional_size;
    if section_table > bytes.len() || optional_size < 112 {
        return Err(PeError::Malformed);
    }
    if read_u16(bytes, optional)? != 0x20b {
        return Err(PeError::Unsupported);
    }
    let directory_count = read_u32(bytes, optional + 108)? as usize;
    if directory_count < 14 || directory_count > (optional_size - 112) / 8 {
        return Err(PeError::Unsupported);
    }
    let header_size = read_u32(bytes, optional + 60)? as usize;
    let header_end = section_table + count * 40;
    if header_size < header_end || header_size > bytes.len() {
        return Err(PeError::Malformed);
    }
    let mut sections: Vec<Section> = Vec::with_capacity(count);
    for index in 0..count {
        let offset = section_table + index * 40;
        bytes.get(offset..offset + 40).ok_or(PeError::Malformed)?;
        let section = Section {
            virtual_size: read_u32(bytes, offset + 8)?,
            rva: read_u32(bytes, offset + 12)?,
            raw_size: read_u32(bytes, offset + 16)?,
            raw_start: read_u32(bytes, offset + 20)? as usize,
        };
        let end = section
            .raw_start
            .checked_add(section.raw_size as usize)
            .ok_or(PeError::Malformed)?;
        if end > bytes.len()
            || (section.raw_size != 0 && section.raw_start < header_size)
            || (section.virtual_size.max(section.raw_size) != 0
                && (section.rva as usize) < header_size)
        {
            return Err(PeError::Malformed);
        }
        let virtual_end = section
            .rva
            .checked_add(section.virtual_size.max(section.raw_size))
            .ok_or(PeError::Malformed)?;
        for previous in &sections {
            let previous_end = previous.rva + previous.virtual_size.max(previous.raw_size);
            let previous_raw_end = previous.raw_start + previous.raw_size as usize;
            if (section.rva < previous_end && previous.rva < virtual_end)
                || (section.raw_size != 0
                    && previous.raw_size != 0
                    && section.raw_start < previous_raw_end
                    && previous.raw_start < end)
            {
                return Err(PeError::Malformed);
            }
        }
        sections.push(section);
    }
    let imports = optional + 112 + 8;
    let delay = optional + 112 + 13 * 8;
    Ok(DeclaredImports {
        ordinary: directory(
            bytes,
            &sections,
            read_u32(bytes, imports)?,
            read_u32(bytes, imports + 4)?,
            false,
        )?,
        delayed: directory(
            bytes,
            &sections,
            read_u32(bytes, delay)?,
            read_u32(bytes, delay + 4)?,
            true,
        )?,
    })
}

fn put16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 0x600];
    bytes[..2].copy_from_slice(b"MZ");
    put32(&mut bytes, 0x3c, 0x80);
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
    put16(&mut bytes, 0x84, 0x8664);
    put16(&mut bytes, 0x86, 1);
    put16(&mut bytes, 0x94, 240);
    let optional = 0x98;
    put16(&mut bytes, optional, 0x20b);
    put32(&mut bytes, optional + 60, 0x200);
    put32(&mut bytes, optional + 108, 16);
    put32(&mut bytes, optional + 120, 0x1000);
    put32(&mut bytes, optional + 124, 40);
    put32(&mut bytes, optional + 216, 0x1080);
    put32(&mut bytes, optional + 220, 64);
    let section = optional + 240;
    put32(&mut bytes, section + 8, 0x400);
    put32(&mut bytes, section + 12, 0x1000);
    put32(&mut bytes, section + 16, 0x400);
    put32(&mut bytes, section + 20, 0x200);
    put32(&mut bytes, 0x20c, 0x1100);
    put32(&mut bytes, 0x280, 1);
    put32(&mut bytes, 0x284, 0x1120);
    bytes[0x300..0x30d].copy_from_slice(b"KERNEL32.dll\0");
    bytes[0x320..0x32a].copy_from_slice(b"later.dll\0");
    bytes
}

#[test]
fn pe_import_frontier_reads_ordinary_and_delay_names() {
    let imports = declared_imports(&fixture()).expect("synthetic PE");
    assert_eq!(imports.ordinary, BTreeSet::from(["kernel32.dll".into()]));
    assert_eq!(imports.delayed, BTreeSet::from(["later.dll".into()]));
}

#[test]
fn pe_import_frontier_refuses_unmapped_or_truncated_data() {
    let original = fixture();
    for end in [0, 0x3c, 0x84, 0x180, 0x200, 0x300, 0x5ff] {
        assert!(declared_imports(&original[..end]).is_err());
    }
    let mut bytes = fixture();
    put32(&mut bytes, 0x20c, 0x2000);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
    let mut bytes = fixture();
    put32(&mut bytes, 0x98 + 124, 20);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
}

#[test]
fn pe_import_frontier_refuses_path_names_and_non_rva_delay_imports() {
    let mut bytes = fixture();
    bytes[0x300..0x30d].copy_from_slice(b"C:/evil.dll\0\0");
    assert_eq!(declared_imports(&bytes), Err(PeError::Unsupported));
    let mut bytes = fixture();
    put32(&mut bytes, 0x280, 0);
    assert_eq!(declared_imports(&bytes), Err(PeError::Unsupported));
}

#[test]
fn pe_import_frontier_refuses_ambiguous_sections_and_excess_descriptors() {
    let mut bytes = fixture();
    put16(&mut bytes, 0x86, 2);
    let first = 0x98 + 240;
    bytes.copy_within(first..first + 40, first + 40);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
    let mut bytes = fixture();
    put32(&mut bytes, 0x98 + 124, 1024 * 1024 + 1);
    assert_eq!(declared_imports(&bytes), Err(PeError::Budget));
}

#[test]
fn pe_import_frontier_refuses_unterminated_names_and_virtual_tail() {
    let mut bytes = fixture();
    bytes[0x300..0x381].fill(b'a');
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
    let mut bytes = fixture();
    put32(&mut bytes, 0x98 + 240 + 8, 0x100);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
}

#[test]
fn pe_import_frontier_empty_directories_do_not_imply_loader_safety() {
    let mut bytes = fixture();
    for offset in [120, 124, 216, 220] {
        put32(&mut bytes, 0x98 + offset, 0);
    }
    let imports = declared_imports(&bytes).expect("no declared imports");
    assert!(imports.ordinary.is_empty());
    assert!(imports.delayed.is_empty());
    // No return type from this parser can establish trusted load identity.
}

#[test]
fn pe_import_frontier_refuses_sections_overlapping_declared_headers() {
    let mut bytes = fixture();
    // Section bytes still exist and contain a valid import table, but
    // the declared headers cover part of the section's raw extent.
    put32(&mut bytes, 0x98 + 60, 0x300);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));

    let mut bytes = fixture();
    // Move the virtual section and all corresponding directory/name
    // RVAs into the header range without moving the raw fixture bytes.
    put32(&mut bytes, 0x98 + 240 + 12, 0x100);
    put32(&mut bytes, 0x98 + 120, 0x100);
    put32(&mut bytes, 0x98 + 216, 0x180);
    put32(&mut bytes, 0x20c, 0x200);
    put32(&mut bytes, 0x284, 0x220);
    assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));

    for size in [0, 0x1af, 0x601] {
        let mut bytes = fixture();
        put32(&mut bytes, 0x98 + 60, size);
        assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
    }
}

#[test]
fn pe_import_frontier_enforces_descriptor_count_with_duplicate_names() {
    // Repeated names cannot bypass the budget through set deduplication.
    for delayed in [false, true] {
        for count in [MAX_IMPORTS, MAX_IMPORTS + 1] {
            let mut bytes = fixture();
            bytes.resize(0x3000, 0);
            bytes[0x200..].fill(0);
            put32(&mut bytes, 0x98 + 240 + 8, 0x2e00);
            put32(&mut bytes, 0x98 + 240 + 16, 0x2e00);
            for offset in [120, 124, 216, 220] {
                put32(&mut bytes, 0x98 + offset, 0);
            }
            let stride = if delayed { 32 } else { 20 };
            let directory_offset = if delayed { 216 } else { 120 };
            put32(&mut bytes, 0x98 + directory_offset, 0x1000);
            put32(
                &mut bytes,
                0x98 + directory_offset + 4,
                ((count + 1) * stride) as u32,
            );
            for index in 0..count {
                let descriptor = 0x200 + index * stride;
                if delayed {
                    put32(&mut bytes, descriptor, 1);
                }
                put32(
                    &mut bytes,
                    descriptor + if delayed { 4 } else { 12 },
                    0x3d00,
                );
            }
            bytes[0x2f00..0x2f0a].copy_from_slice(b"later.dll\0");
            let result = declared_imports(&bytes);
            if count == MAX_IMPORTS {
                let imports = result.expect("exact descriptor budget");
                let names = if delayed {
                    imports.delayed
                } else {
                    imports.ordinary
                };
                assert_eq!(names, BTreeSet::from(["later.dll".into()]));
            } else {
                assert_eq!(result, Err(PeError::Budget));
            }
        }
    }
}

#[test]
fn pe_import_frontier_rejects_trailing_descriptors_and_partial_entries() {
    for delayed in [false, true] {
        let stride = if delayed { 32 } else { 20 };
        let offset = if delayed { 220 } else { 124 };
        let raw_start = if delayed { 0x280 } else { 0x200 };
        let after_terminator = raw_start + stride * 2;
        // Extra complete zero descriptors are harmless padding.
        let mut bytes = fixture();
        put32(&mut bytes, 0x98 + offset, (stride * 3) as u32);
        assert!(declared_imports(&bytes).is_ok());
        // A well-formed-looking descriptor after the terminator is ambiguous.
        if delayed {
            put32(&mut bytes, after_terminator, 1);
        }
        put32(
            &mut bytes,
            after_terminator + if delayed { 4 } else { 12 },
            if delayed { 0x1120 } else { 0x1100 },
        );
        assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));

        // Partial descriptors are rejected even when their bytes are zero.
        let mut bytes = fixture();
        put32(&mut bytes, 0x98 + offset, (stride * 2 + 1) as u32);
        assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
        bytes[after_terminator] = 1;
        assert_eq!(declared_imports(&bytes), Err(PeError::Malformed));
    }
}

#[test]
fn pe_import_frontier_distinguishes_gnu_idata_payload_from_descriptors() {
    let mut bytes = fixture();
    // A section-sized import directory includes the lookup/address data
    // and DLL name, not just the null-terminated descriptor prefix.
    put32(&mut bytes, 0x98 + 124, 0x10d);
    put32(&mut bytes, 0x98 + 216, 0);
    put32(&mut bytes, 0x98 + 220, 0);
    bytes[0x280..0x2c0].fill(0);
    put32(&mut bytes, 0x200, 0x1040);
    put32(&mut bytes, 0x210, 0x1060);
    put32(&mut bytes, 0x240, 0x1100);
    put32(&mut bytes, 0x260, 0x1100);
    let imports = declared_imports(&bytes).expect("descriptor prefix and referenced payload");
    assert_eq!(imports.ordinary, BTreeSet::from(["kernel32.dll".into()]));
    assert!(imports.delayed.is_empty());

    // Unreferenced nonzero bytes before the first referenced payload
    // still refuse, even in a section-sized ordinary import directory.
    let mut gap = bytes.clone();
    gap[0x228] = 1;
    assert_eq!(declared_imports(&gap), Err(PeError::Malformed));

    // A payload reference cannot shorten the array into a descriptor
    // or its required terminator.
    for rva in [0x1004, 0x1014, 0x1020] {
        let mut overlap = bytes.clone();
        put32(&mut overlap, 0x200, rva);
        assert_eq!(declared_imports(&overlap), Err(PeError::Malformed));
    }
}

#[cfg(windows)]
#[test]
#[ignore = "explicit checksum-verified disposable package fixture only"]
fn pe_import_frontier_fixture_from_file() {
    use std::io::Read;

    // Only the isolated test adapter reads an explicitly supplied fixture.
    // The actual parser remains pure, with no ambient access.
    let path = std::env::var_os("TALOS_I324_PE_FIXTURE").expect("fixture path");
    let file = std::fs::File::open(path).expect("open fixture image");
    let mut bytes = Vec::new();
    file.take(MAX_IMAGE as u64 + 1)
        .read_to_end(&mut bytes)
        .expect("bounded fixture read");
    let imports = declared_imports(&bytes).expect("declared imports in pinned fixture");
    for name in imports.ordinary {
        println!("I324_PE_DECLARED kind=ordinary name={name}");
    }
    for name in imports.delayed {
        println!("I324_PE_DECLARED kind=delayed name={name}");
    }
}

const MAX_GRAPH_IMAGES: usize = 32;
const MAX_GRAPH_BYTES: usize = 128 * 1024 * 1024;
const MAX_GRAPH_EDGES: usize = 2048;

#[derive(Debug, PartialEq, Eq)]
struct ImportGraph {
    // A root and reachable supplied images only. Each node retains the
    // ordinary/delay distinction. Keys are labels, not load identities.
    nodes: BTreeMap<String, DeclaredImports>,
    // Missing from this supplied catalog, not absent from the host.
    unresolved: BTreeSet<String>,
}

// Candidate-only: exact caller-supplied labels and bytes, never name-to-path
// resolution. A catalog match does not establish what Windows would load.
fn declared_graph(
    root: &[u8],
    supplied: &[(&str, &[u8])],
    byte_budget: usize,
    edge_budget: usize,
) -> Result<ImportGraph> {
    if supplied.len() >= MAX_GRAPH_IMAGES {
        return Err(PeError::Budget);
    }
    let mut total_bytes = root.len();
    let mut catalog = BTreeMap::from([("git.exe", root)]);
    for &(name, bytes) in supplied {
        if name.len() <= 4
            || name.len() > MAX_NAME
            || !name.ends_with(".dll")
            || name.starts_with('.')
            || name.contains("..")
            || !name.bytes().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'-' | b'_' | b'.')
            })
        {
            return Err(PeError::Unsupported);
        }
        if catalog.insert(name, bytes).is_some() {
            return Err(PeError::Malformed);
        }
        total_bytes = total_bytes
            .checked_add(bytes.len())
            .ok_or(PeError::Budget)?;
        if bytes.len() > MAX_IMAGE {
            return Err(PeError::Budget);
        }
    }
    if root.len() > MAX_IMAGE || total_bytes > byte_budget.min(MAX_GRAPH_BYTES) {
        return Err(PeError::Budget);
    }

    let mut nodes = BTreeMap::new();
    let mut unresolved = BTreeSet::new();
    let mut pending = BTreeSet::from(["git.exe".to_owned()]);
    let mut edges = 0;
    while let Some(label) = pending.pop_first() {
        if nodes.contains_key(&label) {
            continue;
        }
        let bytes = catalog.get(label.as_str()).ok_or(PeError::Malformed)?;
        let imports = declared_imports(bytes)?;
        edges += imports.ordinary.len() + imports.delayed.len();
        if edges > edge_budget.min(MAX_GRAPH_EDGES) {
            return Err(PeError::Budget);
        }
        for name in imports.ordinary.iter().chain(&imports.delayed) {
            if catalog.contains_key(name.as_str()) {
                if !nodes.contains_key(name) {
                    pending.insert(name.clone());
                }
            } else {
                unresolved.insert(name.clone());
            }
        }
        nodes.insert(label, imports);
    }
    Ok(ImportGraph { nodes, unresolved })
}

fn graph_fixture(ordinary: Option<&str>, delayed: Option<&str>) -> Vec<u8> {
    let mut bytes = fixture();
    bytes[0x300..0x380].fill(0);
    for (name, raw, dir) in [(ordinary, 0x300, 120), (delayed, 0x320, 216)] {
        if let Some(name) = name {
            bytes[raw..raw + name.len()].copy_from_slice(name.as_bytes());
        } else {
            put32(&mut bytes, 0x98 + dir, 0);
            put32(&mut bytes, 0x98 + dir + 4, 0);
        }
    }
    bytes
}

#[test]
fn pe_graph_preserves_cycles_shared_edges_and_unresolved_names() {
    let root = graph_fixture(Some("alpha.dll"), Some("beta.dll"));
    let alpha = graph_fixture(Some("beta.dll"), Some("kernel32.dll"));
    let beta = graph_fixture(Some("alpha.dll"), Some("kernel32.dll"));
    let supplied: &[(&str, &[u8])] = &[
        ("alpha.dll", &alpha),
        ("beta.dll", &beta),
        ("unused.dll", b"not a reachable image"),
    ];
    let graph = declared_graph(&root, supplied, MAX_GRAPH_BYTES, MAX_GRAPH_EDGES)
        .expect("bounded declared graph");
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.unresolved, BTreeSet::from(["kernel32.dll".into()]));
    assert_eq!(
        graph.nodes["git.exe"].ordinary,
        BTreeSet::from(["alpha.dll".into()])
    );
    assert_eq!(
        graph.nodes["git.exe"].delayed,
        BTreeSet::from(["beta.dll".into()])
    );
    assert_eq!(
        graph.nodes["alpha.dll"].ordinary,
        BTreeSet::from(["beta.dll".into()])
    );
    assert_eq!(
        graph.nodes["beta.dll"].ordinary,
        BTreeSet::from(["alpha.dll".into()])
    );
    for name in ["alpha.dll", "beta.dll"] {
        assert_eq!(
            graph.nodes[name].delayed,
            BTreeSet::from(["kernel32.dll".into()])
        );
    }
}

#[test]
fn pe_graph_rejects_ambiguous_catalog_and_malformed_reachable_image() {
    let root = graph_fixture(Some("alpha.dll"), None);
    let alpha = graph_fixture(None, None);
    for name in ["ALPHA.dll", "../alpha.dll", "alpha.exe"] {
        assert_eq!(
            declared_graph(&root, &[(name, &alpha)], MAX_GRAPH_BYTES, MAX_GRAPH_EDGES),
            Err(PeError::Unsupported)
        );
    }
    assert_eq!(
        declared_graph(
            &root,
            &[("alpha.dll", &alpha), ("alpha.dll", &alpha)],
            MAX_GRAPH_BYTES,
            MAX_GRAPH_EDGES,
        ),
        Err(PeError::Malformed)
    );
    assert_eq!(
        declared_graph(
            &root,
            &[("alpha.dll", b"invalid PE")],
            MAX_GRAPH_BYTES,
            MAX_GRAPH_EDGES,
        ),
        Err(PeError::Unsupported)
    );
    let graph = declared_graph(&root, &[], MAX_GRAPH_BYTES, MAX_GRAPH_EDGES)
        .expect("missing catalog entries remain unresolved");
    assert_eq!(graph.unresolved, BTreeSet::from(["alpha.dll".into()]));
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn pe_graph_enforces_aggregate_budgets_before_returning_evidence() {
    let root = graph_fixture(Some("alpha.dll"), Some("beta.dll"));
    let leaf = graph_fixture(None, None);
    assert_eq!(
        declared_graph(
            &root,
            &[("unused.dll", &leaf)],
            root.len() + leaf.len() - 1,
            MAX_GRAPH_EDGES,
        ),
        Err(PeError::Budget)
    );
    assert_eq!(
        declared_graph(&root, &[], MAX_GRAPH_BYTES, 1),
        Err(PeError::Budget)
    );
    let names: Vec<_> = (0..MAX_GRAPH_IMAGES)
        .map(|index| format!("leaf{index}.dll"))
        .collect();
    let supplied: Vec<_> = names
        .iter()
        .map(|name| (name.as_str(), leaf.as_slice()))
        .collect();
    assert_eq!(
        declared_graph(&root, &supplied, MAX_GRAPH_BYTES, MAX_GRAPH_EDGES),
        Err(PeError::Budget)
    );
}

#[cfg(windows)]
#[test]
#[ignore = "explicit checksum-verified disposable PortableGit fixture only"]
fn pe_graph_pinned_portablegit_rebound() {
    use std::io::Read;
    use std::path::PathBuf;

    let root = PathBuf::from(
        std::env::var_os("TALOS_I324_PE_GRAPH_ROOT").expect("explicit extracted fixture directory"),
    );
    let mut remaining = MAX_GRAPH_BYTES;
    let mut read_image = |name: &str| {
        let file = std::fs::File::open(root.join(name)).expect("explicit fixture image");
        let limit = MAX_IMAGE.min(remaining);
        let mut bytes = Vec::new();
        file.take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .expect("bounded fixture read");
        assert!(bytes.len() <= limit);
        remaining -= bytes.len();
        bytes
    };
    let writer = read_image("git.exe");
    let names = [
        "libpcre2-8-0.dll",
        "libintl-8.dll",
        "libiconv-2.dll",
        "libssp-0.dll",
        "zlib1.dll",
    ];
    let images: Vec<_> = names.iter().map(|name| read_image(name)).collect();
    let supplied: Vec<_> = names
        .iter()
        .zip(&images)
        .map(|(&name, bytes)| (name, bytes.as_slice()))
        .collect();
    let graph = declared_graph(&writer, &supplied, MAX_GRAPH_BYTES, MAX_GRAPH_EDGES)
        .expect("pinned supplied-image graph");
    assert_eq!(graph.nodes.len(), 6);
    assert_eq!(
        graph.unresolved,
        BTreeSet::from([
            "advapi32.dll".into(),
            "kernel32.dll".into(),
            "msvcrt.dll".into(),
            "ntdll.dll".into(),
            "user32.dll".into(),
            "ws2_32.dll".into(),
        ])
    );
    for (image, imports) in &graph.nodes {
        for name in &imports.ordinary {
            println!("I324_PE_GRAPH_EDGE image={image} kind=ordinary name={name}");
        }
        for name in &imports.delayed {
            println!("I324_PE_GRAPH_EDGE image={image} kind=delayed name={name}");
        }
    }
    for name in &graph.unresolved {
        println!("I324_PE_GRAPH_UNRESOLVED name={name}");
    }
    println!("I324_PE_GRAPH nodes=6 unresolved=6 static_only=true");
}
