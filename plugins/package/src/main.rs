//! Offline packaging of explicitly built language guests; never installs or activates.

use sha2::{Digest, Sha256};
use std::{error::Error, fs, io::Read, path::Path};
use wasmparser::{ExternalKind, Operator, Parser, Payload, ValType, Validator};

fn verify(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    Validator::new().validate_all(bytes)?;
    let mut version_index = None;
    let mut functions = Vec::new();
    let mut exports = Vec::new();
    let mut types = Vec::new();
    let mut function_types = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::Version { encoding, .. } if encoding != wasmparser::Encoding::Module => {
                return Err("language guest must be a core WASM module".into());
            }
            Payload::MemorySection(section) => {
                if section.count() != 1 {
                    return Err("language guest requires exactly one memory".into());
                }
                for memory in section {
                    let memory = memory?;
                    if memory.memory64 || memory.shared || memory.initial > 1024 {
                        return Err("language guest memory exceeds ABI/resource contract".into());
                    }
                }
            }
            Payload::TypeSection(section) => {
                for ty in section.into_iter_err_on_gc_types() {
                    types.push(ty?);
                }
            }
            Payload::FunctionSection(section) => {
                for ty in section {
                    function_types.push(ty? as usize);
                }
            }
            Payload::ImportSection(section) if section.count() != 0 => {
                return Err("language guests must have no imports".into());
            }
            Payload::StartSection { .. } => {
                return Err("language guests must have no start function".into());
            }
            Payload::ExportSection(section) => {
                for export in section {
                    let export = export?;
                    exports.push((export.name.to_owned(), export.kind, export.index));
                    if export.name == "talos_language_abi_version"
                        && export.kind == ExternalKind::Func
                    {
                        version_index = Some(export.index as usize);
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let mut ops = body.get_operators_reader()?;
                let constant_version = matches!(ops.read()?, Operator::I32Const { value: 2 })
                    && matches!(ops.read()?, Operator::End)
                    && ops.eof();
                functions.push(constant_version);
            }
            _ => {}
        }
    }
    if !version_index
        .and_then(|index| functions.get(index))
        .copied()
        .unwrap_or(false)
    {
        return Err("version export must return constant ABI version 2".into());
    }
    for (name, kind) in [
        ("memory", ExternalKind::Memory),
        ("talos_language_alloc", ExternalKind::Func),
        ("talos_language_run", ExternalKind::Func),
    ] {
        if !exports
            .iter()
            .any(|(export, export_kind, _)| export == name && *export_kind == kind)
        {
            return Err(format!("missing required export: {name}").into());
        }
    }
    for (name, parameters, results) in [
        ("talos_language_abi_version", &[][..], &[ValType::I32][..]),
        (
            "talos_language_alloc",
            &[ValType::I32][..],
            &[ValType::I32][..],
        ),
        (
            "talos_language_run",
            &[ValType::I32, ValType::I32][..],
            &[ValType::I64][..],
        ),
    ] {
        let signature = exports
            .iter()
            .find(|(export, _, _)| export == name)
            .and_then(|(_, _, index)| function_types.get(*index as usize))
            .and_then(|index| types.get(*index));
        if !signature.is_some_and(|ty| ty.params() == parameters && ty.results() == results) {
            return Err(format!("incorrect ABI signature: {name}").into());
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 || !matches!(args[0].as_str(), "rust" | "python") {
        return Err(
            "usage: talos-plugin-package rust|python ARTIFACT.wasm NEW_BUNDLE_DIRECTORY".into(),
        );
    }
    let language = &args[0];
    let artifact = Path::new(&args[1]);
    let input = fs::File::open(artifact)?;
    let metadata = input.metadata()?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
        return Err("artifact exceeds 16 MiB packaging limit".into());
    }
    let mut bytes = Vec::new();
    input.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("artifact grew beyond packaging limit".into());
    }
    verify(&bytes)?;
    let mut digest = String::from("sha256:");
    for byte in Sha256::digest(&bytes) {
        use std::fmt::Write;
        write!(digest, "{byte:02x}")?;
    }
    let manifest = format!(
        "schema_version = 1\n\n[bundle]\nname = \"talos-language-{language}\"\nversion = \"{}\"\ncarrier = \"wasm\"\nartifact = \"provider.wasm\"\ndigest = \"{digest}\"\ndescription = \"Source-only {language} Tree-sitter provider; requires language ABI v2 host\"\n\n[language_provider]\nlanguage = \"{language}\"\nartifact = \"provider.wasm\"\n",
        env!("CARGO_PKG_VERSION")
    );
    // Never overwrite an existing package, install destination, or user directory.
    // All artifact validation completes before creating the explicitly chosen output.
    let destination = Path::new(&args[2]);
    fs::create_dir(destination)?;
    // The existing explicit --plugin / lifecycle loader consumes the legacy
    // activation filename. Derive it from the same identity, without inventing
    // an install-implies-activation path or changing the host's loader policy.
    let activation = format!(
        "[plugin]\nname = \"talos-language-{language}\"\nversion = \"{}\"\ncarrier = \"wasm\"\nartifact = \"provider.wasm\"\n\n[language_provider]\nlanguage = \"{language}\"\nartifact = \"provider.wasm\"\n",
        env!("CARGO_PKG_VERSION")
    );
    let written = (|| -> Result<(), std::io::Error> {
        fs::write(destination.join("provider.wasm"), bytes)?;
        fs::write(destination.join("manifest.toml"), manifest)?;
        fs::write(destination.join("talos-plugin.toml"), activation)?;
        Ok(())
    })();
    if let Err(error) = written {
        return Err(format!("package incomplete at {}: {error}; nothing installed or activated; inspect and remove this incomplete output before retrying", destination.display()).into());
    }
    println!(
        "packaged {language}: {} ({digest}); not installed or activated",
        destination.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify;

    const VALID: &str = r#"(module
        (memory (export "memory") 1)
        (func (export "talos_language_abi_version") (result i32) i32.const 2)
        (func (export "talos_language_alloc") (param i32) (result i32) i32.const 1024)
        (func (export "talos_language_run") (param i32 i32) (result i64) i64.const 0)
    )"#;

    #[test]
    fn accepts_only_core_version_two_no_import_no_start_abi() {
        verify(&wat::parse_str(VALID).expect("valid module")).expect("valid contract");
        for invalid in [
            VALID.replace("i32.const 2", "i32.const 1"),
            VALID.replace("i32.const 2", "i32.const 1 i32.const 1 i32.add"),
            VALID.replace("(param i32) (result i32)", "(result i32)"),
            VALID.replace("(param i32 i32) (result i64)", "(param i32) (result i64)"),
            VALID.replace("(module", "(module (func $start) (start $start)"),
            VALID.replace("(module", "(module (import \"host\" \"call\" (func))"),
            VALID.replace(
                "(memory (export \"memory\") 1)",
                "(memory (export \"memory\") i64 1)",
            ),
            VALID.replace(
                "(memory (export \"memory\") 1)",
                "(memory (export \"memory\") 1 1 shared)",
            ),
            VALID.replace(
                "(memory (export \"memory\") 1)",
                "(memory (export \"memory\") 1025)",
            ),
        ] {
            let bytes = wat::parse_str(&invalid).expect("negative WAT syntax");
            assert!(
                verify(&bytes).is_err(),
                "accepted invalid contract: {invalid}"
            );
        }
        assert!(verify(&wat::parse_str("(component)").expect("component")).is_err());
        assert!(verify(b"not WASM").is_err());
    }
}
