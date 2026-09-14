use token_db::TokenDb;

#[test]
fn insert_assigns_stable_ids_and_counts_occurrences() {
    let mut db = TokenDb::new();

    let hello = db.insert("hello").unwrap();
    let world = db.insert("world").unwrap();
    let hello_again = db.insert("hello").unwrap();

    assert_eq!(hello, hello_again);
    assert_ne!(hello, world);
    assert_eq!(db.len(), 2);
    assert_eq!(db.get(hello).unwrap().text(), "hello");
    assert_eq!(db.get(hello).unwrap().count(), 2);
    assert_eq!(db.get_by_token("world").unwrap().count(), 1);
    assert_eq!(db.id("missing"), None);
}

#[test]
fn get_is_safe_for_unknown_ids_from_another_database() {
    let mut first = TokenDb::new();
    let mut second = TokenDb::new();

    first.insert("one").unwrap();
    second.insert("one").unwrap();
    let foreign = second.insert("two").unwrap();

    assert!(first.get(foreign).is_none());
}

#[test]
fn iteration_is_in_id_order() {
    let mut db = TokenDb::new();
    db.insert("a").unwrap();
    db.insert("b").unwrap();
    db.insert("c").unwrap();

    let tokens: Vec<_> = db.iter().map(|(id, entry)| (id.get(), entry.text())).collect();
    assert_eq!(tokens, vec![(0, "a"), (1, "b"), (2, "c")]);
}

#[test]
fn merge_preserves_existing_ids_and_returns_mapping() {
    let mut left = TokenDb::new();
    let hello = left.insert("hello").unwrap();
    let world = left.insert("world").unwrap();

    let mut right = TokenDb::new();
    let right_hello = right.insert("hello").unwrap();
    right.insert("hello").unwrap();
    let right_rust = right.insert("rust").unwrap();
    right.insert("rust").unwrap();

    let mapping = left.merge(&right).unwrap();

    assert_eq!(mapping[right_hello.get() as usize], hello);
    assert_eq!(mapping[right_rust.get() as usize], left.id("rust").unwrap());
    assert_eq!(left.id("world"), Some(world));
    assert_eq!(left.get_by_token("hello").unwrap().count(), 3);
    assert_eq!(left.get_by_token("rust").unwrap().count(), 2);
}
