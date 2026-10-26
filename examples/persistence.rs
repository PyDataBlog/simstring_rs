use simstring_rust::{CharacterNgrams, Cosine, HashDb, Searcher};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let mut db = HashDb::new(Arc::new(CharacterNgrams::new(2, "$")));
    for text in ["hello", "help", "world"] {
        db.insert(text.into());
    }
    db.save_to_file(directory.path(), "names.ssdb")?;
    let mut restored = HashDb::load_from_file(directory.path(), "names.ssdb")?;
    restored.insert("halo".into());
    let searcher = Searcher::new(&restored, Cosine);
    for (text, score) in searcher.ranked_search("hello", 0.5)? {
        println!("{text}: {score:.4}");
    }
    Ok(())
}
