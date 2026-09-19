//! Opt-in real Bundle acceptance: TALOS_LANGUAGE_BUNDLE_ROOT must contain rust/ and python/.

use std::sync::{Arc, Mutex};
use talos_core::{capability::CapabilityRegistry, tool::AgentTool};
use talos_plugin::{install_bundle, lifecycle::PluginLifecycle, wasm::WasmRuntime};
use talos_tools::symbol::{FindReferencesTool, FindSymbolTool, ListImportsTool, ListSymbolsTool};

#[tokio::test]
async fn installed_plugins_drive_all_four_symbol_tools() {
    let bundles = std::path::PathBuf::from(
        std::env::var_os("TALOS_LANGUAGE_BUNDLE_ROOT")
            .expect("provide real packaged Bundles for plugin-acceptance"),
    );
    let temp = tempfile::tempdir().expect("test directory");
    for (language, extension, source) in [
        (
            "rust",
            "rs",
            "use std::fmt;\n// fn fake() {}\nfn greet() {}\nfn main() { greet(); }\n",
        ),
        (
            "python",
            "py",
            "import os\n# def fake():\ndef greet():\n    pass\ngreet()\n",
        ),
    ] {
        let package = temp.path().join(format!("package-{language}"));
        install_bundle(&bundles.join(language), &package).expect("verified install");
        let mut lifecycle =
            PluginLifecycle::new(Arc::new(Mutex::new(CapabilityRegistry::default())));
        lifecycle.load(&package).expect("load");
        lifecycle
            .initialize(Arc::new(WasmRuntime::new(100_000, 250).expect("runtime")))
            .expect("initialize");
        lifecycle.activate().expect("activate");
        let context = lifecycle
            .take_language_provider_context()
            .expect("context state")
            .expect("provider");
        let workspace = temp.path().join(language);
        std::fs::create_dir(&workspace).expect("workspace");
        let file = format!("sample.{extension}");
        std::fs::write(workspace.join(&file), source).expect("source");
        let list = ListSymbolsTool::with_provider(workspace.clone(), context.clone());
        let result = list
            .execute(serde_json::json!({"path":".", "kind":"function"}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        let symbols: serde_json::Value =
            serde_json::from_str(&result.content).expect("symbols JSON");
        assert!(
            symbols
                .as_array()
                .expect("array")
                .iter()
                .any(|symbol| symbol["name"] == "greet" && symbol["file"] == file)
        );
        assert!(
            !symbols
                .as_array()
                .expect("array")
                .iter()
                .any(|symbol| symbol["name"] == "fake")
        );

        let find = FindSymbolTool::with_provider(workspace.clone(), context.clone());
        let result = find
            .execute(serde_json::json!({"name":"greet", "path":"."}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        let definitions: serde_json::Value =
            serde_json::from_str(&result.content).expect("definitions");
        assert_eq!(definitions[0]["definition"]["line"], 3);
        assert_eq!(definitions[0]["definition"]["file"], file);

        let references = FindReferencesTool::with_provider(workspace.clone(), context.clone());
        let result = references
            .execute(serde_json::json!({"name":"greet", "file":file}))
            .await;
        assert!(!result.is_error, "{}", result.content);
        let references: serde_json::Value =
            serde_json::from_str(&result.content).expect("references");
        assert_eq!(references.as_array().expect("array").len(), 2);

        let imports = ListImportsTool::with_provider(workspace.clone(), context);
        let result = imports.execute(serde_json::json!({"file":file})).await;
        assert!(!result.is_error, "{}", result.content);
        let imports: serde_json::Value = serde_json::from_str(&result.content).expect("imports");
        assert_eq!(imports.as_array().expect("array").len(), 1);

        std::fs::write(workspace.join(&file), source.replace("greet", "changed"))
            .expect("second source");
        let result = list
            .execute(serde_json::json!({"path":".", "kind":"function"}))
            .await;
        assert!(
            !result.is_error
                && result.content.contains("changed")
                && !result.content.contains("greet")
        );
        lifecycle.stop().expect("stop");
        assert!(
            list.execute(serde_json::json!({"path":".", "kind":"function"}))
                .await
                .is_error
        );
    }
}
