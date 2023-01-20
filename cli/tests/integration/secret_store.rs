use crate::common::{Test, TestResult};
use hyper::{body::to_bytes, StatusCode};
use viceroy_lib::config::FastlyConfig;
use viceroy_lib::error::{FastlyConfigError, SecretStoreConfigError};

#[tokio::test(flavor = "multi_thread")]
async fn secret_store_works() -> TestResult {
    const FASTLY_TOML: &str = r#"
        name = "secret-store"
        description = "secret store test"
        authors = ["Jill Bryson <jbryson@fastly.com>", "Rose McDowall <rmcdowall@fastly.com>"]
        language = "rust"
        [local_server]
        secret_store.store_one = [{key = "first", data = "This is some data"},{key = "second", path = "../test-fixtures/data/object-store.txt"}]
    "#;

    let resp = Test::using_fixture("secret-store.wasm")
        .using_fastly_toml(FASTLY_TOML)?
        .against_empty()
        .await;

    assert_eq!(resp.status(), StatusCode::OK);
    assert!(to_bytes(resp.into_body())
        .await
        .expect("can read body")
        .to_vec()
        .is_empty());

    Ok(())
}

fn bad_config_test(toml_fragment: &str) -> Result<FastlyConfig, FastlyConfigError> {
    let toml = format!(
        r#"
        name = "secret-store"
        description = "secret store test"
        authors = ["Jill Bryson <jbryson@fastly.com>", "Rose McDowall <rmcdowall@fastly.com>"]
        language = "rust"
        [local_server]
        {}
    "#,
        toml_fragment
    );

    println!("TOML: {}", toml);
    toml.parse::<FastlyConfig>()
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_store_not_array() -> TestResult {
    const TOML_FRAGMENT: &str = "secret_store.store_one = 1";
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::NotAnArray,
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::NotAnArray"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_store_not_table() -> TestResult {
    const TOML_FRAGMENT: &str = "secret_store.store_one = [1]";
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::NotATable,
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::NotATable"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_no_key() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::NoKey,
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::NoKey"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_key_not_string() -> TestResult {
    const TOML_FRAGMENT: &str =
        r#"secret_store.store_one = [{key = 1, data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::KeyNotAString,
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::KeyNotAString"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_no_data_or_path() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{key = "first"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::NoPathOrData(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::NoPathOrData"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_both_data_and_path() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{key = "first", path = "file.txt", data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::PathAndData(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::PathAndData"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_data_not_string() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{key = "first", data = 1}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::DataNotAString(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::DataNotAString"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_path_not_string() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{key = "first", path = 1}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::PathNotAString(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::PathNotAString"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_path_nonexistent() -> TestResult {
    const TOML_FRAGMENT: &str =
        r#"secret_store.store_one = [{key = "first", path = "nonexistent.txt"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::IoError(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::IoError"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_invalid_store_name() -> TestResult {
    const TOML_FRAGMENT: &str =
        r#"secret_store.store*one = [{key = "first", data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidFastlyToml(_)) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidFastlyToml"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_invalid_secret_name() -> TestResult {
    const TOML_FRAGMENT: &str =
        r#"secret_store.store_one = [{key = "first*", data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::InvalidSecretName(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::InvalidSecretName"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_config_secret_name_too_long() -> TestResult {
    const TOML_FRAGMENT: &str = r#"secret_store.store_one = [{key = "firstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirstfirst", data = "This is some data"}]"#;
    match bad_config_test(TOML_FRAGMENT) {
        Err(FastlyConfigError::InvalidSecretStoreDefinition {
            err: SecretStoreConfigError::InvalidSecretName(_),
            ..
        }) => (),
        Err(_) => panic!("Expected a FastlyConfigError::InvalidSecretStoreDefinition with SecretStoreConfigError::InvalidSecretName"),
        _ => panic!("Expected an error"),
    }
    Ok(())
}