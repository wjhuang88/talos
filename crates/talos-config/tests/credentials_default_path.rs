use talos_config::Credentials;

#[test]
fn credentials_default_path_uses_credentials_toml() {
    // Keep the filename contract explicit while the unit test covers the parent `.talos` directory.
    let path = Credentials::default_path();
    assert!(path.to_string_lossy().ends_with("credentials.toml"));
}
