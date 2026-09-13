// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

#[cfg(test)]
mod tests {
    use token_db::token_db::TokenDB;

    #[test]
    fn test_push_and_lookup() {
        let mut db = TokenDB::default();

        // Erstes Einfügen
        let idx_apple1 = db.push("apple");
        assert_eq!(idx_apple1, 0);

        // Zweites Einfügen erhöht den Count, behält den Index bei
        let idx_apple2 = db.push("apple");
        assert_eq!(idx_apple2, 0);

        // Neues Token einfügen
        let idx_banana = db.push("banana");
        assert_eq!(idx_banana, 1);

        // Lookup über Index
        let entry_0 = db.get_by_index(0);
        assert_eq!(&*entry_0.text, "apple");
        assert_eq!(entry_0.count, 2);
        assert_eq!(entry_0.index, 0);

        // Lookup über Namen
        let entry_banana = db.get_by_name("banana").unwrap();
        assert_eq!(entry_banana.count, 1);
        assert_eq!(entry_banana.index, 1);

        // Lookup für nicht existierendes Token
        assert!(db.get_by_name("cherry").is_none());
    }

    #[test]
    fn test_join() {
        let mut db1 = TokenDB::default();
        db1.push("apple");  // count: 1, idx: 0
        db1.push("apple");  // count: 2, idx: 0
        db1.push("banana"); // count: 1, idx: 1

        let mut db2 = TokenDB::default();
        db2.push("banana"); // count: 1, idx: 0
        db2.push("cherry"); // count: 1, idx: 1

        db1.join(&db2);

        // "apple" bleibt unverändert
        assert_eq!(db1.get_by_name("apple").unwrap().count, 2);

        // "banana" addiert sich (1 + 1)
        assert_eq!(db1.get_by_name("banana").unwrap().count, 2);
        assert_eq!(db1.get_by_name("banana").unwrap().index, 1);

        // "cherry" wurde neu hinzugefügt
        let cherry = db1.get_by_name("cherry").unwrap();
        assert_eq!(cherry.count, 1);
        assert_eq!(cherry.index, 2);
        assert_eq!(db1.get_ref_list().len(), 3);
    }

    #[test]
    fn test_join_with_map() {
        let mut db1 = TokenDB::default();
        db1.push("apple");  // idx: 0, count: 1
        db1.push("banana"); // idx: 1, count: 1

        let mut db2 = TokenDB::default();
        db2.push("banana"); // idx: 0 in db2
        db2.push("cherry"); // idx: 1 in db2
        db2.push("apple");  // idx: 2 in db2

        // Mapping generieren
        let map = db1.join_with_map(&db2);

        // Überprüfung der Mapping-Integrität
        // "banana": Index 0 in db2 -> gemappt auf Index 1 in db1
        assert_eq!(map.first(), Some(&1));
        // "cherry": Index 1 in db2 -> gemappt auf neuen Index 2 in db1
        assert_eq!(map.get(1), Some(&2));
        // "apple": Index 2 in db2 -> gemappt auf Index 0 in db1
        assert_eq!(map.get(2), Some(&0));

        // Validierung der finalen Werte in db1
        assert_eq!(db1.get_by_name("apple").unwrap().count, 2);
        assert_eq!(db1.get_by_name("banana").unwrap().count, 2);
        assert_eq!(db1.get_by_name("cherry").unwrap().count, 1);
    }

    #[test]
    fn test_rebuild_map() {
        let mut db = TokenDB::default();
        db.push("apple");
        db.push("banana");

        // Map künstlich leeren, um Deserialisierungs-Zustand zu simulieren
        db.clear_internal_map_for_testing();
        assert!(db.get_by_name("apple").is_none());

        // Map neu aufbauen
        db.rebuild_map();

        // Lookups müssen wieder funktionieren
        assert!(db.get_by_name("apple").is_some());
        assert_eq!(db.get_by_name("banana").unwrap().index, 1);
    }

    #[test]
    fn test_serialization_and_deserialization() {
        // Erfordert das `tempfile` Crate in den dev-dependencies:
        // cargo add --dev tempfile
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("tokens.bin");

        let mut db_out = TokenDB::default();
        db_out.push("hello");
        db_out.push("world");
        db_out.push("hello");

        // In Datei schreiben
        db_out.save(&file_path).unwrap();

        // Aus Datei laden
        let db_in = TokenDB::load(&file_path).unwrap();

        // Prüfen, ob die Liste korrekt übertragen wurde
        assert_eq!(db_in.get_ref_list().len(), 2);
        assert_eq!(db_in.get_ref_list()[0].text.as_ref(), "hello");
        assert_eq!(db_in.get_ref_list()[0].count, 2);

        // CRUCIAL: Prüfen, ob der benutzerdefinierte Deserialize-Block
        // `rebuild_map` erfolgreich im Hintergrund aufgerufen hat!
        let entry = db_in.get_by_name("world");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().count, 1);
        assert_eq!(entry.unwrap().index, 1);
    }
}

