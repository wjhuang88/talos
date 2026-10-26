use talos_config::Credentials;

#[test]
fn credentials_default_path_uses_credentials_toml() {
    let path = Credentials::default_path();
    assert!(path.to_string_lossy().ends_with("credentials.toml"));
}
