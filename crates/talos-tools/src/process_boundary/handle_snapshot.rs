//! Temporary self-only test metadata observer. Native/unsafe boundary: ADR-091.
use std::{collections::BTreeMap, io::Write, ptr};
use windows_sys::Win32::{
    Foundation::{ERROR_NO_MORE_ITEMS, ERROR_SUCCESS},
    System::{
        Diagnostics::ProcessSnapshotting::{
            HPSS, HPSSWALK, PSS_CAPTURE_HANDLE_NAME_INFORMATION, PSS_CAPTURE_HANDLES,
            PSS_HANDLE_ENTRY, PSS_HANDLE_HAVE_TYPE, PSS_WALK_HANDLES, PssCaptureSnapshot,
            PssFreeSnapshot, PssWalkMarkerCreate, PssWalkMarkerFree, PssWalkSnapshot,
        },
        Threading::GetCurrentProcess,
    },
};

pub(super) const OUTPUT_CAP: usize = 16_384;
const ENTRY_CAP: usize = 1024;
const CHANGE_CAP: usize = 32;

#[derive(Clone, PartialEq, Eq, serde::Serialize)]
pub(super) struct HandleType {
    object_type: Option<i32>,
    type_name: Option<String>,
}
pub(super) type Handles = BTreeMap<usize, HandleType>;

// Take ownership before calling release. An uncertain failed release must not retry.
fn release_pair<T, U>(
    marker: &mut Option<T>,
    snapshot: &mut Option<U>,
    free_marker: impl FnOnce(T) -> u32,
    free_snapshot: impl FnOnce(U) -> u32,
) -> (u32, u32) {
    let marker = marker.take().map_or(ERROR_SUCCESS, free_marker);
    let snapshot = snapshot.take().map_or(ERROR_SUCCESS, free_snapshot);
    (marker, snapshot)
}

#[derive(Default)]
struct Snapshot {
    snapshot: Option<HPSS>,
    marker: Option<HPSSWALK>,
}

#[derive(Clone, Copy)]
enum Fault {
    AfterCapture,
    AfterMarker,
    PanicAfterMarker,
}

impl Snapshot {
    fn release(&mut self) -> Result<(), String> {
        // SAFETY: ADR-091. Only successful initialized outputs are owned. Marker first;
        // attempt snapshot cleanup even if marker cleanup fails. Never close pseudohandle.
        let (marker, snapshot) = release_pair(
            &mut self.marker,
            &mut self.snapshot,
            |marker| unsafe { PssWalkMarkerFree(marker) },
            |snapshot| unsafe { PssFreeSnapshot(GetCurrentProcess(), snapshot) },
        );
        if marker == ERROR_SUCCESS && snapshot == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(format!("PSS cleanup marker={marker} snapshot={snapshot}"))
        }
    }

    fn collect(&mut self, fault: Option<Fault>) -> Result<Handles, String> {
        let mut snapshot = ptr::null_mut();
        // SAFETY: ADR-091. Fixed current process, handles/name flags, zero contexts,
        // initialized output; no caller-controlled PID, handles, flags or native dumps.
        let status = unsafe {
            PssCaptureSnapshot(
                GetCurrentProcess(),
                PSS_CAPTURE_HANDLES | PSS_CAPTURE_HANDLE_NAME_INFORMATION,
                0,
                &mut snapshot,
            )
        };
        if status != ERROR_SUCCESS {
            return Err(format!("PSS capture status={status}"));
        }
        if snapshot.is_null() {
            return Err("PSS null snapshot owner".to_owned());
        }
        self.snapshot = Some(snapshot);
        if matches!(fault, Some(Fault::AfterCapture)) {
            return Err("injected post-capture failure".to_owned());
        }
        let mut marker = ptr::null_mut();
        // SAFETY: ADR-091. Default allocator and initialized output for one owned marker.
        let status = unsafe { PssWalkMarkerCreate(ptr::null(), &mut marker) };
        if status != ERROR_SUCCESS {
            return Err(format!("PSS marker status={status}"));
        }
        if marker.is_null() {
            return Err("PSS null marker owner".to_owned());
        }
        self.marker = Some(marker);
        if matches!(fault, Some(Fault::AfterMarker)) {
            return Err("injected post-marker failure".to_owned());
        }
        assert!(
            !matches!(fault, Some(Fault::PanicAfterMarker)),
            "injected observer panic"
        );
        let mut handles = Handles::new();
        loop {
            let mut entry = PSS_HANDLE_ENTRY::default();
            // SAFETY: ADR-091. Exclusive live owners, correctly sized initialized ABI
            // output. Copy the returned type buffer before advancing/freeing its marker.
            let status = unsafe {
                PssWalkSnapshot(
                    snapshot,
                    PSS_WALK_HANDLES,
                    marker,
                    ptr::from_mut(&mut entry).cast(),
                    size_of::<PSS_HANDLE_ENTRY>() as u32,
                )
            };
            if status == ERROR_NO_MORE_ITEMS {
                return Ok(handles);
            }
            if status != ERROR_SUCCESS {
                return Err(format!("PSS walk status={status}"));
            }
            if handles.len() == ENTRY_CAP {
                return Err("PSS entry cap exceeded".to_owned());
            }
            let object_type = object_type(entry.Flags, entry.ObjectType)?;
            // HAVE_NAME validates ObjectName, not TypeName. Never inspect ObjectName.
            // The successful fixed NAME capture/walk supplies TypeName's marker lifetime.
            let units = type_units(entry.TypeNameLength, entry.TypeName as usize)?;
            let type_name = if units == 0 {
                None
            } else {
                // SAFETY: ADR-091. Provenance is the successful OS walk with live marker,
                // not just these null/alignment/length checks. No pointer escapes.
                let units = unsafe { std::slice::from_raw_parts(entry.TypeName, units) };
                Some(decode_type(units)?)
            };
            // Only handle value and type; no reserved fields, object names or unions.
            if handles
                .insert(
                    entry.Handle as usize,
                    HandleType {
                        object_type,
                        type_name,
                    },
                )
                .is_some()
            {
                return Err("PSS duplicate handle value".to_owned());
            }
        }
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        // Fallback only for owners whose release was never attempted; no uncertain retries.
        if let Err(error) = self.release() {
            // A broken diagnostic pipe must not cause a second panic during unwind.
            let _ = writeln!(std::io::stderr().lock(), "{error}");
        }
    }
}

fn object_type(flags: i32, value: i32) -> Result<Option<i32>, String> {
    if flags & PSS_HANDLE_HAVE_TYPE == 0 {
        return Ok(None);
    }
    if !(0..=6).contains(&value) {
        return Err("PSS invalid object type".to_owned());
    }
    Ok(Some(value)) // documented enum 0=unknown,1..6=process/thread/mutant/event/section/semaphore
}

fn type_units(bytes: u16, address: usize) -> Result<usize, String> {
    if bytes > 128 || !bytes.is_multiple_of(2) {
        return Err("PSS type length invalid".to_owned());
    }
    if bytes != 0 && (address == 0 || !address.is_multiple_of(align_of::<u16>())) {
        return Err("PSS type pointer invalid".to_owned());
    }
    Ok(usize::from(bytes) / 2)
}

fn decode_type(units: &[u16]) -> Result<String, String> {
    String::from_utf16(units).map_err(|_| "PSS invalid UTF16".to_owned())
}

pub(super) fn capture() -> Result<Handles, String> {
    capture_with_fault(None)
}

fn capture_with_fault(fault: Option<Fault>) -> Result<Handles, String> {
    let mut owner = Snapshot::default();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| owner.collect(fault)))
        .unwrap_or_else(|_| Err("PSS collection panicked; evidence incomplete".to_owned()));
    // Finish both possible cleanups before returning any collection/panic error.
    let cleanup = owner.release();
    match (result, cleanup) {
        (Ok(handles), Ok(())) => Ok(handles),
        (Err(error), Ok(())) | (Ok(_), Err(error)) => Err(error),
        (Err(error), Err(cleanup)) => Err(format!("{error}; {cleanup}")),
    }
}

pub(super) fn differences(phases: &[(&str, Handles)]) -> Result<Vec<u8>, String> {
    if phases.len() != 4 || phases.iter().any(|(_, handles)| handles.len() > ENTRY_CAP) {
        return Err("PSS snapshot/entry cap invalid".to_owned());
    }
    let mut changes = Vec::new();
    for pair in phases.windows(2) {
        for (value, kind) in &pair[1].1 {
            if pair[0].1.get(value) != Some(kind) {
                if changes.len() == CHANGE_CAP {
                    return Err("PSS change cap exceeded; incomplete evidence".to_owned());
                }
                changes.push(serde_json::json!({"phase":pair[1].0,"change":"added_or_retyped","value":value,"type":kind}));
            }
        }
        for (value, kind) in &pair[0].1 {
            if !pair[1].1.contains_key(value) {
                if changes.len() == CHANGE_CAP {
                    return Err("PSS change cap exceeded; incomplete evidence".to_owned());
                }
                changes.push(serde_json::json!({"phase":pair[1].0,"change":"removed","value":value,"type":kind}));
            }
        }
    }
    let output = serde_json::to_vec(&serde_json::json!({
        "child_pid":std::process::id(),"snapshots":4,"changes":changes,
        "phase_sizes":phases.iter().map(|(name, handles)|(*name,handles.len())).collect::<Vec<_>>(),
        "caveat":"handle reuse/observer effects; not object identity or leak proof"
    }))
    .map_err(|_| "PSS output encoding".to_owned())?;
    // Reserve 4KiB of the worker's output cap for controls/trace/test harness.
    if output.len() > OUTPUT_CAP - 4096 {
        return Err("PSS output cap exceeded".to_owned());
    }
    Ok(output)
}

// Only called in its own Job-owned fixture worker. Exactly four native captures,
// never in the concurrent shared test harness; no fake pointers enter native calls.
pub(super) fn native_fixtures() {
    let handles = capture().expect("checked native owned copies");
    assert!(!handles.is_empty());
    assert!(serde_json::to_vec(&handles).is_ok()); // usable after native owners released
    for fault in [Fault::AfterCapture, Fault::AfterMarker] {
        let mut owner = Snapshot::default();
        let expected = if matches!(fault, Fault::AfterCapture) {
            "injected post-capture failure"
        } else {
            "injected post-marker failure"
        };
        assert_eq!(owner.collect(Some(fault)).err().as_deref(), Some(expected));
        assert!(owner.snapshot.is_some());
        assert_eq!(owner.marker.is_some(), matches!(fault, Fault::AfterMarker));
        owner.release().expect("native partial-setup cleanup");
        assert!(owner.marker.is_none() && owner.snapshot.is_none());
        owner.release().expect("consumed owners are a no-op");
    }
    assert_eq!(
        capture_with_fault(Some(Fault::PanicAfterMarker))
            .err()
            .as_deref(),
        Some("PSS collection panicked; evidence incomplete")
    ); // no cleanup error suffix
}

#[test]
fn metadata_predicates_reject_without_dereferencing_fabricated_addresses() {
    for (bytes, address) in [(3, 2), (130, 2), (2, 0), (2, 1)] {
        assert!(type_units(bytes, address).is_err());
    }
    assert_eq!(type_units(0, 0), Ok(0));
    let owned = [0u16; 64];
    assert_eq!(type_units(128, owned.as_ptr() as usize), Ok(64));
    assert_eq!(object_type(0, i32::MAX), Ok(None));
    assert_eq!(object_type(PSS_HANDLE_HAVE_TYPE, 0), Ok(Some(0)));
    assert!(object_type(PSS_HANDLE_HAVE_TYPE, 7).is_err());
    assert_eq!(
        decode_type(&[69, 118, 101, 110, 116]),
        Ok("Event".to_owned())
    );
    assert!(decode_type(&[0xd800]).is_err());
}

#[test]
fn two_release_errors_are_retained_in_order_without_retry() {
    let order = std::cell::RefCell::new(Vec::new());
    let (mut marker, mut snapshot) = (Some(1usize), Some(2usize));
    assert_eq!(
        release_pair(
            &mut marker,
            &mut snapshot,
            |value| {
                order.borrow_mut().push(value);
                5
            },
            |value| {
                order.borrow_mut().push(value);
                6
            }
        ),
        (5, 6)
    );
    assert_eq!(*order.borrow(), [1, 2]);
    assert_eq!(
        release_pair(
            &mut marker,
            &mut snapshot,
            |_| panic!("marker retry"),
            |_| panic!("snapshot retry")
        ),
        (ERROR_SUCCESS, ERROR_SUCCESS)
    );
}

#[test]
fn evidence_caps_fail_instead_of_reporting_partial_success() {
    assert!(differences(&[]).is_err());
    for size in [33, ENTRY_CAP + 1] {
        let handles = (0..size)
            .map(|value| {
                (
                    value,
                    HandleType {
                        object_type: None,
                        type_name: None,
                    },
                )
            })
            .collect();
        assert!(
            differences(&[
                ("baseline", Handles::new()),
                ("control", handles),
                ("pipe", Handles::new()),
                ("final", Handles::new())
            ])
            .is_err()
        );
    }
    let huge = [(
        1,
        HandleType {
            object_type: None,
            type_name: Some("x".repeat(OUTPUT_CAP)),
        },
    )]
    .into_iter()
    .collect();
    assert!(
        differences(&[
            ("baseline", Handles::new()),
            ("control", huge),
            ("pipe", Handles::new()),
            ("final", Handles::new())
        ])
        .is_err()
    );
}

#[test]
fn unknown_type_is_not_replaced_with_an_inferred_type() {
    let handles: Handles = [(
        1,
        HandleType {
            object_type: None,
            type_name: None,
        },
    )]
    .into_iter()
    .collect();
    let output = differences(&[
        ("baseline", Handles::new()),
        ("control", handles.clone()),
        ("pipe", handles.clone()),
        ("final", handles),
    ])
    .expect("owned bounded output");
    let data: serde_json::Value = serde_json::from_slice(&output).expect("JSON");
    assert!(data["changes"][0]["type"]["object_type"].is_null());
    assert!(data["changes"][0]["type"]["type_name"].is_null());
}
