#![allow(missing_docs)]

use bytes::Bytes;
use futures::stream;
use rust_multer::{
    Limits, MemoryStorage, Multer, MulterConfig, MulterError, SelectedField, Selector,
    UnknownFieldPolicy,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let config = MulterConfig {
        selector: Selector::fields([
            SelectedField::new("avatar").with_max_count(1),
            SelectedField::new("gallery").with_max_count(2),
        ]),
        unknown_field_policy: UnknownFieldPolicy::Reject,
        limits: Limits {
            allowed_mime_types: vec!["image/*".to_owned()],
            max_files: Some(3),
            ..Limits::default()
        },
    };

    let multer = Multer::with_config(MemoryStorage::new(), config).expect("valid config");
    let body = concat!(
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"avatar\"; filename=\"a.png\"\r\n",
        "Content-Type: image/png\r\n",
        "\r\n",
        "a\r\n",
        "--BOUND\r\n",
        "Content-Disposition: form-data; name=\"unknown\"; filename=\"b.png\"\r\n",
        "Content-Type: image/png\r\n",
        "\r\n",
        "b\r\n",
        "--BOUND--\r\n"
    );

    let result = multer
        .parse_and_store(
            "BOUND",
            stream::iter([Ok::<Bytes, MulterError>(Bytes::from_static(body.as_bytes()))]),
        )
        .await;

    match result {
        Ok(output) => println!("stored {} files", output.stored_files.len()),
        Err(err) => println!("validation error: {err}"),
    }
}
