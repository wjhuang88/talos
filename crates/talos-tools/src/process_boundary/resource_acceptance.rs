//! TEST-004: observe the existing native call sites, never own historical descriptors.
//! The closures borrow the launcher's still-live owners. No allocation/assertion/panic
//! occurs between deliberate omission and recovery. A failed native release is never retried.
use super::{
    WindowsBackgroundLauncher, WindowsLaunchFault, WindowsLaunchTestHooks,
    create_windows_job_with_hooks,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum Role {
    OutRead,
    OutWrite,
    ErrRead,
    ErrWrite,
    Job,
    Process,
    Thread,
    Attributes,
}
impl Role {
    fn bit(self) -> u8 {
        1 << self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Negative {
    Close,
    Duplicate,
    Termination,
    Attributes,
}

pub(super) struct ResourceAudit {
    fault: WindowsLaunchFault,
    generation: usize,
    negative: Option<Negative>,
    values: [usize; 8],
    acquired: u8,
    attempted: u8,
    released: u8,
    termination_attempted: bool,
    terminated: bool,
    exited: bool,
    detected: bool,
    recovered: bool,
    invalid: bool,
}
impl ResourceAudit {
    pub(super) fn new(
        fault: WindowsLaunchFault,
        generation: usize,
        negative: Option<Negative>,
    ) -> Self {
        Self {
            fault,
            generation,
            negative,
            values: [0; 8],
            acquired: 0,
            attempted: 0,
            released: 0,
            termination_attempted: false,
            terminated: false,
            exited: false,
            detected: false,
            recovered: false,
            invalid: false,
        }
    }
    pub(super) fn acquired(&mut self, role: Role, value: usize) {
        let bit = role.bit();
        if value == 0
            || value == usize::MAX
            || self.acquired & bit != 0
            || self.values.contains(&value)
        {
            self.invalid = true;
        }
        self.values[role as usize] = value;
        self.acquired |= bit;
    }
    fn matches(&mut self, role: Role, value: usize) {
        if self.acquired & role.bit() == 0 || self.values[role as usize] != value {
            self.invalid = true;
        }
    }
    // These SAME predicates reject actual missing operations in final acceptance
    // and in each negative snapshot before any recovery call.
    fn require_release(&self, role: Role) -> Result<(), Role> {
        let bit = role.bit();
        if self.acquired & bit == 0 || self.attempted & bit == 0 || self.released & bit == 0 {
            Err(role)
        } else {
            Ok(())
        }
    }
    fn require_termination(&self) -> Result<(), &'static str> {
        if self.termination_attempted && self.terminated && self.exited {
            Ok(())
        } else {
            Err("missing checked termination/exit")
        }
    }
    pub(super) fn close_pipe(&mut self, value: usize, native: impl FnMut() -> i32) -> i32 {
        // Caller iterates the four original, distinct, successful-return slots.
        let role = [Role::OutRead, Role::OutWrite, Role::ErrRead, Role::ErrWrite]
            .into_iter()
            .find(|role| self.values[*role as usize] == value);
        match role {
            Some(role) => self.close(role, value, native),
            None => {
                self.invalid = true;
                let mut native = native;
                native()
            }
        }
    }
    pub(super) fn close(
        &mut self,
        role: Role,
        value: usize,
        mut native: impl FnMut() -> i32,
    ) -> i32 {
        self.matches(role, value);
        let bit = role.bit();
        if self.attempted & bit != 0 {
            self.invalid = true;
            return 0;
        }
        // This is local release authority borrowed from the live cleanup frame,
        // not authority derived from self.values (historical observations).
        let mut owned = Some(value);
        let omit = self.negative == Some(Negative::Close) && role == Role::OutRead;
        let mut calls = 0u8;
        let mut release = |owner: &mut Option<usize>| {
            owner.take()?;
            calls += 1;
            Some(native())
        };
        // This is the operation-path injection: skip the actual normal close.
        let normal = if omit { None } else { release(&mut owned) };
        let result = if omit {
            self.detected =
                normal.is_none() && owned.is_some() && self.require_release(role) == Err(role);
            // Explicit recovery is a different call path, before returning to caller.
            let recovered = release(&mut owned).unwrap_or(0);
            self.recovered = recovered != 0;
            recovered
        } else {
            normal.unwrap_or(0)
        };
        self.attempted |= bit;
        if result != 0 {
            self.released |= bit;
        }
        if self.negative == Some(Negative::Duplicate) && role == Role::OutRead {
            // A genuine second operation request, guarded before another native call.
            let before = self.attempted;
            let duplicate = release(&mut owned);
            self.detected =
                duplicate.is_none() && self.released & bit != 0 && self.attempted == before;
            self.recovered = result != 0;
        }
        if calls != 1 {
            self.invalid = true;
        }
        result
    }
    pub(super) fn terminate(
        &mut self,
        value: usize,
        mut native: impl FnMut() -> i32,
        mut wait: impl FnMut(u32) -> u32,
    ) -> i32 {
        use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
        self.matches(Role::Process, value);
        if self.termination_attempted {
            self.invalid = true;
            return 0;
        }
        let omit = self.negative == Some(Negative::Termination);
        let normal = if omit {
            None
        } else {
            self.termination_attempted = true;
            Some(native())
        };
        let result = if omit {
            // Actual normal TerminateProcess omitted. Independent observation on
            // still-owned suspended process must report live before recovery.
            self.detected =
                normal.is_none() && wait(0) == WAIT_TIMEOUT && self.require_termination().is_err();
            self.termination_attempted = true;
            native() // explicit recovery, only one native termination attempt
        } else {
            normal.unwrap_or(0)
        };
        self.terminated = result != 0;
        self.exited = wait(5_000) == WAIT_OBJECT_0;
        if omit {
            self.recovered = self.terminated && self.exited;
        }
        result
    }
    pub(super) fn delete(&mut self, value: usize, initialized: bool, mut native: impl FnMut()) {
        self.matches(Role::Attributes, value);
        let bit = Role::Attributes.bit();
        if !initialized || self.attempted & bit != 0 {
            self.invalid = true;
            return;
        }
        let mut owned = Some(value);
        let omit = self.negative == Some(Negative::Attributes);
        if !omit {
            let _ = owned.take();
            self.attempted |= bit;
            native(); // void ABI: invocation proof only
            self.released |= bit;
        }
        if omit {
            // Real normal Delete omitted; all backing storage still lives in caller.
            self.detected =
                owned.is_some() && self.require_release(Role::Attributes) == Err(Role::Attributes);
            if owned.take().is_some() {
                self.attempted |= bit;
                native(); // explicit recovery Delete before returning to live owner
                self.released |= bit;
                self.recovered = true;
            }
        }
    }
    pub(super) fn finish(&self) -> Result<Summary, &'static str> {
        let expected = match self.fault {
            WindowsLaunchFault::Pipe => 0b0000_0011,
            WindowsLaunchFault::Job => 0b0001_1111,
            WindowsLaunchFault::AttributeList => 0b1001_1111,
            WindowsLaunchFault::Assignment | WindowsLaunchFault::Resume => 0xff,
            WindowsLaunchFault::UnsupportedHost => return Err("unsupported audited fault"),
        };
        let process = expected & Role::Process.bit() != 0;
        if self.invalid
            || self.generation == 0
            || self.acquired != expected
            || self.attempted & !expected != 0
            || self.released & !expected != 0
            || [
                Role::OutRead,
                Role::OutWrite,
                Role::ErrRead,
                Role::ErrWrite,
                Role::Job,
                Role::Process,
                Role::Thread,
                Role::Attributes,
            ]
            .into_iter()
            .any(|role| expected & role.bit() != 0 && self.require_release(role).is_err())
            || (process && self.require_termination().is_err())
            || (!process && (self.termination_attempted || self.terminated || self.exited))
            || (self.negative.is_some() && (!self.detected || !self.recovered))
        {
            return Err("native ownership/release/termination/negative evidence incomplete");
        }
        Ok(Summary {
            attempts: 1,
            resources: expected.count_ones(),
            closes: (expected & !Role::Attributes.bit()).count_ones(),
            deletes: u32::from(expected & Role::Attributes.bit() != 0),
            terminations: u32::from(self.terminated),
            exits: u32::from(self.exited),
            generation_sum: self.generation,
        })
    }
}

#[derive(Default, serde::Serialize)]
pub(super) struct Summary {
    attempts: u32,
    resources: u32,
    closes: u32,
    deletes: u32,
    terminations: u32,
    exits: u32,
    generation_sum: usize,
}
impl Summary {
    pub(super) fn add(&mut self, other: Self) {
        self.attempts += other.attempts;
        self.resources += other.resources;
        self.closes += other.closes;
        self.deletes += other.deletes;
        self.terminations += other.terminations;
        self.exits += other.exits;
        self.generation_sum += other.generation_sum;
    }
    pub(super) fn require_all40(&self) {
        assert_eq!(
            serde_json::to_value(self).expect("summary"),
            all40_receipt()
        );
    }
}
pub(super) fn all40_receipt() -> serde_json::Value {
    serde_json::json!({"attempts":40,"resources":232,"closes":208,"deletes":24,
        "terminations":16,"exits":16,"generation_sum":820})
}
pub(super) fn negative_receipt() -> serde_json::Value {
    serde_json::json!({"omitted_close":true,"duplicate_guard":true,
        "omitted_termination_live":true,"omitted_initialized_delete":true,
        "recovered":4})
}
pub(super) fn negative_controls(launcher: &WindowsBackgroundLauncher) -> serde_json::Value {
    for (index, negative) in [
        Negative::Close,
        Negative::Duplicate,
        Negative::Termination,
        Negative::Attributes,
    ]
    .into_iter()
    .enumerate()
    {
        let fault = if matches!(negative, Negative::Close | Negative::Duplicate) {
            WindowsLaunchFault::Pipe
        } else {
            WindowsLaunchFault::Assignment
        };
        let mut audit = ResourceAudit::new(fault, index + 1, Some(negative));
        let result = create_windows_job_with_hooks(
            &launcher.program,
            &launcher.args,
            &launcher.cwd,
            &launcher.env,
            WindowsLaunchTestHooks {
                fault: Some(fault),
                audit: Some(&mut audit),
                ..Default::default()
            },
        );
        // Every live owner has completed native cleanup/recovery before assertions.
        let expected = if fault == WindowsLaunchFault::Pipe {
            "injected pipe setup failure"
        } else {
            "injected AssignProcessToJobObject failure"
        };
        assert_eq!(result.err().as_deref(), Some(expected));
        audit
            .finish()
            .expect("real negative detected and native recovery succeeded");
        eprintln!(
            "WINDOWS_RESOURCE_NEGATIVE generation={} control={negative:?} detected={} recovered={}",
            index + 1,
            audit.detected,
            audit.recovered
        );
    }
    negative_receipt()
}

#[test]
fn actual_result_predicates_fail_closed_and_never_retry_release() {
    let mut audit = ResourceAudit::new(WindowsLaunchFault::Pipe, 1, None);
    audit.acquired(Role::OutRead, 1);
    audit.acquired(Role::OutWrite, 2);
    assert_eq!(audit.require_release(Role::OutRead), Err(Role::OutRead));
    assert!(audit.finish().is_err());
    assert_eq!(audit.close(Role::OutRead, 1, || 0), 0);
    assert_eq!(audit.require_release(Role::OutRead), Err(Role::OutRead));
    let mut repeated_native_calls = 0;
    assert_eq!(
        audit.close(Role::OutRead, 1, || {
            repeated_native_calls += 1;
            1
        }),
        0
    );
    assert_eq!(repeated_native_calls, 0);
    assert_eq!(audit.close(Role::OutWrite, 2, || 1), 1);
    assert!(audit.finish().is_err());
}

#[test]
fn termination_requires_real_success_and_signalled_wait_and_rejects_second_call() {
    use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
    for (status, wait_status) in [(0, WAIT_OBJECT_0), (1, WAIT_TIMEOUT), (1, u32::MAX)] {
        let mut audit = ResourceAudit::new(WindowsLaunchFault::Assignment, 1, None);
        audit.acquired(Role::Process, 6);
        audit.terminate(
            6,
            || status,
            |millis| {
                assert_eq!(millis, 5_000);
                wait_status
            },
        );
        assert!(audit.require_termination().is_err());
        let mut repeated_native_calls = 0;
        assert_eq!(
            audit.terminate(
                6,
                || {
                    repeated_native_calls += 1;
                    1
                },
                |_| WAIT_OBJECT_0
            ),
            0
        );
        assert_eq!(repeated_native_calls, 0);
        assert!(audit.invalid);
    }
}
