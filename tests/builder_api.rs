#![allow(missing_docs)]

use rust_multer::{
    ConfigError, Limits, Multer, MulterBuilder, MulterConfig, Selector, UnknownFieldPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestStorage {
    id: u8,
}

#[test]
fn builder_default_is_permissive() {
    let builder = MulterBuilder::default();
    assert_eq!(builder.config(), &MulterConfig::default());

    let multer = Multer::builder()
        .build()
        .expect("default builder config should be valid");
    assert_eq!(multer.config(), &MulterConfig::default());
}

#[test]
fn fluent_chaining_sets_expected_configuration() {
    let limits = Limits {
        max_file_size: Some(1024),
        max_files: Some(4),
        allowed_mime_types: vec!["image/*".to_owned()],
        ..Limits::default()
    };

    let multer = Multer::builder()
        .single("avatar")
        .unknown_field_policy(UnknownFieldPolicy::Reject)
        .limits(limits.clone())
        .build()
        .expect("builder config should validate");

    assert_eq!(
        multer.config(),
        &MulterConfig {
            selector: Selector::single("avatar"),
            unknown_field_policy: UnknownFieldPolicy::Reject,
            limits,
        }
    );
}

#[test]
fn builder_supports_custom_storage() {
    let multer = Multer::builder()
        .storage(TestStorage { id: 7 })
        .any()
        .build()
        .expect("builder config should validate");

    assert_eq!(multer.storage().id, 7);
}

#[test]
fn build_surfaces_config_errors() {
    let result = Multer::builder().array("photos", 0).build();
    assert!(matches!(
        result,
        Err(ConfigError::InvalidArrayMaxCount { .. })
    ));
}
