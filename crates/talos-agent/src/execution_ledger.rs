//! Per-turn tool execution ledger.
//!
//! A provider may resend a tool call after a transport/protocol failure.  Once
//! admission has completed, the runtime must assume that execution can have
//! reached the tool even when no result was observed.  The ledger therefore
//! reserves a logical call before invocation and never releases a reservation
//! on cancellation or error.  This is deliberately in-memory and agent-local;
//! durable replay custody belongs to the session layer.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use talos_core::message::ToolCall;
use talos_plugin::TurnId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Started,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Key {
    turn: TurnId,
    call_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Record {
    name: String,
    input: String,
    state: State,
}

/// The result of attempting to reserve a logical tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReservationError {
    /// The same call identity was already admitted or executed.
    AlreadyReserved,
    /// A provider reused an id for a different call payload.
    IdentityConflict,
}

/// Agent-local ledger that prevents one logical call from being executed twice.
#[derive(Clone, Default)]
pub(crate) struct ExecutionLedger {
    records: Arc<Mutex<HashMap<Key, Record>>>,
}

impl ExecutionLedger {
    pub(crate) fn reserve(&self, turn: TurnId, call: &ToolCall) -> Result<(), ReservationError> {
        let key = Key {
            turn,
            call_id: call.id.clone(),
        };
        let record = Record {
            name: call.name.clone(),
            input: call.input.to_string(),
            state: State::Started,
        };
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(previous) = records.get(&key) {
            if previous.name != record.name || previous.input != record.input {
                return Err(ReservationError::IdentityConflict);
            }
            return Err(ReservationError::AlreadyReserved);
        }
        records.insert(key, record);
        Ok(())
    }

    pub(crate) fn complete(&self, turn: TurnId, call: &ToolCall) {
        let key = Key {
            turn,
            call_id: call.id.clone(),
        };
        let mut records = self
            .records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(record) = records.get_mut(&key) {
            record.state = State::Completed;
        }
    }

    #[cfg(test)]
    fn state(&self, turn: TurnId, call: &ToolCall) -> Option<State> {
        let key = Key {
            turn,
            call_id: call.id.clone(),
        };
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&key)
            .map(|record| record.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn call(id: &str, input: serde_json::Value) -> ToolCall {
        ToolCall {
            id: id.into(),
            name: "write_file".into(),
            input,
        }
    }

    #[test]
    fn reservation_is_not_released_before_result() {
        let ledger = ExecutionLedger::default();
        let turn = TurnId::new();
        let tool = call("call-1", json!({"path":"a","content":"x"}));
        assert_eq!(ledger.reserve(turn, &tool), Ok(()));
        assert_eq!(ledger.state(turn, &tool), Some(State::Started));
        assert_eq!(
            ledger.reserve(turn, &tool),
            Err(ReservationError::AlreadyReserved)
        );
    }

    #[test]
    fn completed_call_cannot_be_replayed() {
        let ledger = ExecutionLedger::default();
        let turn = TurnId::new();
        let tool = call("call-1", json!({"path":"a"}));
        ledger.reserve(turn, &tool).unwrap();
        ledger.complete(turn, &tool);
        assert_eq!(ledger.state(turn, &tool), Some(State::Completed));
        assert_eq!(
            ledger.reserve(turn, &tool),
            Err(ReservationError::AlreadyReserved)
        );
    }

    #[test]
    fn reused_id_with_different_payload_is_rejected() {
        let ledger = ExecutionLedger::default();
        let turn = TurnId::new();
        let first = call("call-1", json!({"path":"a"}));
        let second = call("call-1", json!({"path":"b"}));
        ledger.reserve(turn, &first).unwrap();
        assert_eq!(
            ledger.reserve(turn, &second),
            Err(ReservationError::IdentityConflict)
        );
    }

    #[test]
    fn same_id_in_a_new_turn_is_a_new_logical_call() {
        let ledger = ExecutionLedger::default();
        let first_turn = TurnId::new();
        let second_turn = TurnId::new();
        let tool = call("tc_0", json!({"path":"a"}));
        ledger.reserve(first_turn, &tool).unwrap();
        assert_eq!(ledger.reserve(second_turn, &tool), Ok(()));
    }
}
