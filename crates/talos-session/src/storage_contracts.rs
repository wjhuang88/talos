//! Domain-owned operational contracts for the existing Session sidecar stores.
//!
//! STORE-002 / I322 C does not create a provider registry or a storage engine.
//! These interfaces forward to existing concrete methods without moving SQL,
//! altering runtime ownership, or changing SessionStore (TLOG/JSONL).
//! Metadata capabilities in talos-core are not operational bindings.

use crate::{
    CreateTodo, PendingSubmissionError, PendingSubmissionRecord, PendingSubmissionStore,
    SessionRuntimeActivation, SessionRuntimeState, TodoDependency, TodoError, TodoItem, TodoQuery,
    TodoRepository, TodoUpdate,
};
use talos_core::submission::{StructuredSubmission, SubmissionReceiptDisposition};
use talos_core::work::WorkGraph;
use uuid::Uuid;

/// Durable custody and generation fencing for structured session submissions.
///
/// The atomic staging/commit and recovery meanings belong to Session. This
/// interface is not interchangeable with a generic relational CRUD provider.
pub trait PendingSubmissionPersistence {
    /// Return the current durable session runtime generation.
    fn runtime_generation(&self) -> Result<u64, PendingSubmissionError>;

    /// Stage activation against the expected generation without changing its ownership.
    fn stage_runtime_activation(
        &self,
        expected: u64,
        activation: &SessionRuntimeActivation,
    ) -> Result<u64, PendingSubmissionError>;

    /// Commit a previously staged runtime activation.
    fn commit_runtime_activation(
        &self,
        activation_id: &str,
    ) -> Result<SessionRuntimeState, PendingSubmissionError>;

    /// Advance the generation only when the expected generation still matches.
    fn advance_runtime_generation(&self, expected: u64) -> Result<u64, PendingSubmissionError>;

    /// Accept a structured submission with existing idempotency and custody semantics.
    fn accept(
        &self,
        submission: &StructuredSubmission,
    ) -> Result<(String, SubmissionReceiptDisposition), PendingSubmissionError>;

    /// Mark a committed turn without dropping its durable receipt.
    fn mark_committed(&self, submission_id: &str, turn_id: &str)
        -> Result<(), PendingSubmissionError>;

    /// Recover running submissions without inventing a new resume policy.
    fn recover_running(&self) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError>;

    /// Read the existing durable custody record.
    fn get(&self, submission_id: &str)
        -> Result<Option<PendingSubmissionRecord>, PendingSubmissionError>;
}

impl PendingSubmissionPersistence for PendingSubmissionStore {
    fn runtime_generation(&self) -> Result<u64, PendingSubmissionError> {
        PendingSubmissionStore::runtime_generation(self)
    }

    fn stage_runtime_activation(
        &self,
        expected: u64,
        activation: &SessionRuntimeActivation,
    ) -> Result<u64, PendingSubmissionError> {
        PendingSubmissionStore::stage_runtime_activation(self, expected, activation)
    }

    fn commit_runtime_activation(
        &self,
        activation_id: &str,
    ) -> Result<SessionRuntimeState, PendingSubmissionError> {
        PendingSubmissionStore::commit_runtime_activation(self, activation_id)
    }

    fn advance_runtime_generation(&self, expected: u64) -> Result<u64, PendingSubmissionError> {
        PendingSubmissionStore::advance_runtime_generation(self, expected)
    }

    fn accept(
        &self,
        submission: &StructuredSubmission,
    ) -> Result<(String, SubmissionReceiptDisposition), PendingSubmissionError> {
        PendingSubmissionStore::accept(self, submission)
    }

    fn mark_committed(
        &self,
        submission_id: &str,
        turn_id: &str,
    ) -> Result<(), PendingSubmissionError> {
        PendingSubmissionStore::mark_committed(self, submission_id, turn_id)
    }

    fn recover_running(&self) -> Result<Vec<PendingSubmissionRecord>, PendingSubmissionError> {
        PendingSubmissionStore::recover_running(self)
    }

    fn get(
        &self,
        submission_id: &str,
    ) -> Result<Option<PendingSubmissionRecord>, PendingSubmissionError> {
        PendingSubmissionStore::get(self, submission_id)
    }
}

/// Session-owned Todo transaction and dependency graph operations.
///
/// The concrete repository continues to own revision/CAS, rollback, scope
/// checks and SQLite transactions. No caller is moved to this trait in C.
pub trait TodoPersistence {
    /// Create a session-scoped Todo item.
    fn create(&self, input: CreateTodo) -> Result<TodoItem, TodoError>;

    /// Read one Todo in its session scope.
    fn get(&self, session_id: Uuid, id: Uuid) -> Result<Option<TodoItem>, TodoError>;

    /// Query Todo items with the unchanged ordering and filter policy.
    fn list(&self, session_id: Uuid, query: TodoQuery) -> Result<Vec<TodoItem>, TodoError>;

    /// Apply the existing atomic batch update and revision checks.
    fn update_batch(
        &self,
        session_id: Uuid,
        updates: Vec<(Uuid, TodoUpdate)>,
    ) -> Result<Vec<TodoItem>, TodoError>;

    /// Preserve the existing dependency validity and cycle checks.
    fn add_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<TodoDependency, TodoError>;

    /// Load the current session work graph without changing edge identity.
    fn load_work_graph(&self, session_id: Uuid) -> Result<WorkGraph, TodoError>;
}

impl TodoPersistence for TodoRepository {
    fn create(&self, input: CreateTodo) -> Result<TodoItem, TodoError> {
        TodoRepository::create(self, input)
    }

    fn get(&self, session_id: Uuid, id: Uuid) -> Result<Option<TodoItem>, TodoError> {
        TodoRepository::get(self, session_id, id)
    }

    fn list(&self, session_id: Uuid, query: TodoQuery) -> Result<Vec<TodoItem>, TodoError> {
        TodoRepository::list(self, session_id, query)
    }

    fn update_batch(
        &self,
        session_id: Uuid,
        updates: Vec<(Uuid, TodoUpdate)>,
    ) -> Result<Vec<TodoItem>, TodoError> {
        TodoRepository::update_batch(self, session_id, updates)
    }

    fn add_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<TodoDependency, TodoError> {
        TodoRepository::add_dependency(self, session_id, parent_id, child_id)
    }

    fn load_work_graph(&self, session_id: Uuid) -> Result<WorkGraph, TodoError> {
        TodoRepository::load_work_graph(self, session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_stores_satisfy_domain_contracts_without_rebinding() {
        fn pending_contract<T: PendingSubmissionPersistence>() {}
        fn todo_contract<T: TodoPersistence>() {}
        pending_contract::<PendingSubmissionStore>();
        todo_contract::<TodoRepository>();
    }
}
