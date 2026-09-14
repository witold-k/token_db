use std::fs;

use tempfile::tempdir;
use token_db::{Error, TokenDb};

#[test]
fn bytes_round_trip_preserves_ids_and_counts() {
    let mut db = TokenDb::new();
    let hello = db.insert("hello").unwrap();
    db.insert("world").unwrap();
    db.insert("hello").unwrap();

    let restored = TokenDb::from_bytes(&db.to_bytes().unwrap()).unwrap();

    assert_eq!(restored.id("hello"), Some(hello));
    assert_eq!(restored.get_by_token("hello").unwrap().count(), 2);
    assert_eq!(restored.get_by_token("world").unwrap().count(), 1);
}

#[test]
fn file_round_trip_works() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("tokens.tdb");

    let mut db = TokenDb::new();
    db.insert("hello").unwrap();
    db.save(&path).unwrap();

    let restored = TokenDb::load(&path).unwrap();
    assert_eq!(restored.get_by_token("hello").unwrap().count(), 1);
}

#[test]
fn unknown_format_is_rejected() {
    let error = TokenDb::from_bytes(b"not-a-token-db").unwrap_err();
    assert!(matches!(error, Error::InvalidFormat(_)));
}

#[test]
fn truncated_payload_is_rejected() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("broken.tdb");
    fs::write(&path, b"TDB1\xff").unwrap();

    assert!(TokenDb::load(path).is_err());
}
