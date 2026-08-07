use std::fs;
use std::path::Path;

#[test]
fn todo_responsibilities_stay_private_behind_the_facade() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = fs::read_to_string(crate_root.join("src/todo.rs")).expect("read todo facade");
    let model = fs::read_to_string(crate_root.join("src/todo/model.rs")).expect("read todo model");
    let repository = fs::read_to_string(crate_root.join("src/todo/repository.rs"))
        .expect("read todo repository");
    let formatting = fs::read_to_string(crate_root.join("src/todo/formatting.rs"))
        .expect("read todo formatting");
    let tools = fs::read_to_string(crate_root.join("src/todo/tools.rs")).expect("read todo tools");

    for module in ["formatting", "model", "repository", "tools"] {
        assert!(facade.contains(&format!("mod {module};")));
    }
    assert!(facade.contains("pub use model::"));
    assert!(facade.contains("pub use repository::TodoRepository;"));
    assert!(facade.contains("pub use tools::"));
    assert!(facade.contains("pub use formatting::status_icon;"));

    assert!(model.contains("pub enum TodoStatus"));
    assert!(repository.contains("pub struct TodoRepository"));
    assert!(formatting.contains("pub fn status_icon"));
    assert!(tools.contains("impl AgentTool for TodoCreateTool"));
    assert!(tools.contains("impl AgentTool for TodoUpdateBatchTool"));

    assert!(!facade.contains("impl TodoRepository"));
    assert!(!facade.contains("impl AgentTool"));
}

#[test]
fn todo_public_paths_compile_through_the_existing_facades() {
    macro_rules! assert_same_type {
        ($name:ident) => {
            assert_eq!(
                std::any::TypeId::of::<talos_session::$name>(),
                std::any::TypeId::of::<talos_session::todo::$name>()
            );
        };
    }

    assert_same_type!(CreateTodo);
    assert_same_type!(TodoAddDependencyTool);
    assert_same_type!(TodoCreateBatchInput);
    assert_same_type!(TodoCreateBatchTool);
    assert_same_type!(TodoCreateInput);
    assert_same_type!(TodoCreateTool);
    assert_same_type!(TodoDeleteInput);
    assert_same_type!(TodoDeleteTool);
    assert_same_type!(TodoDependency);
    assert_same_type!(TodoDependencyInput);
    assert_same_type!(TodoError);
    assert_same_type!(TodoItem);
    assert_same_type!(TodoPriority);
    assert_same_type!(TodoQuery);
    assert_same_type!(TodoQueryInput);
    assert_same_type!(TodoQueryTool);
    assert_same_type!(TodoRemoveDependencyTool);
    assert_same_type!(TodoRepository);
    assert_same_type!(TodoStatus);
    assert_same_type!(TodoUpdate);
    assert_same_type!(TodoUpdateBatchInput);
    assert_same_type!(TodoUpdateBatchTool);
    assert_same_type!(TodoUpdateInput);
    assert_same_type!(TodoUpdateStatusInput);
    assert_same_type!(TodoUpdateStatusTool);
    assert_same_type!(TodoUpdateTool);

    let root_status_icon: fn(talos_session::TodoStatus) -> &'static str =
        talos_session::status_icon;
    let module_status_icon: fn(talos_session::todo::TodoStatus) -> &'static str =
        talos_session::todo::status_icon;
    assert_eq!(root_status_icon as usize, module_status_icon as usize);
}
