and axample how to use this library

```rust
use std::path::Path;

use token_db::token_db::TokenDB;

fn main() -> std::io::Result<()> {
    // Create the first database.
    let mut db1 = TokenDB::default();

    db1.push("hello");
    db1.push("world");
    db1.push("hello");

    // Save it to disk.
    let path = Path::new("db1.bin");
    db1.save(path)?;

    // Load the database back from disk.
    let mut db1 = TokenDB::load(path)?;

    println!("Loaded database:");
    for index in 0..2 {
        let entry = db1.get_by_index(index);
        println!("{}: {} ({})", entry.index, entry.text, entry.count);
    }

    // Create a second database.
    let mut db2 = TokenDB::default();

    db2.push("hello");
    db2.push("rust");
    db2.push("rust");

    // Merge db2 into db1.
    db1.join(&db2);

    // db1 now contains:
    //
    // hello -> 3
    // world -> 1
    // rust  -> 2

    println!("\nMerged database:");

    for index in 0..3 {
        let entry = db1.get_by_index(index);
        println!("{}: {} ({})", entry.index, entry.text, entry.count);
    }

    // Save the merged database.
    db1.save(Path::new("merged.bin"))?;

    Ok(())
}
```
