//! Independent consumer of the public SQLite error conversion boundary.
fn main() {
    use rusqlite::Error::InvalidQuery;
    assert!(matches!(talos_session::PendingSubmissionError::from(InvalidQuery),
        talos_session::PendingSubmissionError::Storage(rusqlite::Error::InvalidQuery)));
    assert!(matches!(talos_memory::MemoryStoreError::from(InvalidQuery),
        talos_memory::MemoryStoreError::Database(rusqlite::Error::InvalidQuery)));
    assert!(matches!(talos_models::CatalogError::from(InvalidQuery),
        talos_models::CatalogError::Database(rusqlite::Error::InvalidQuery)));
    assert!(matches!(talos_exploration::ExplorationError::from(InvalidQuery),
        talos_exploration::ExplorationError::Database(rusqlite::Error::InvalidQuery)));
    let _: talos_session::IndexError = InvalidQuery.into();
    let _: talos_session::TodoError = InvalidQuery.into();
    let _: talos_session::sqlite::IndexStoreError = InvalidQuery.into();
    let _: talos_evolution::EvolutionError = InvalidQuery.into();
    let _: talos_evolution::StoreError = InvalidQuery.into();
    println!("external SQLite error conversions and typed payloads passed");
}
