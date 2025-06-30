use simstring_rust::database::HashDb;
use simstring_rust::extractors::CharacterNgrams;
use simstring_rust::measures::Cosine;
use simstring_rust::Searcher;

use std::sync::Arc;
use std::time::Instant;

fn main() {
    // 1. Setup
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    let mut db = HashDb::new(feature_extractor);
    let measure = Cosine;

    // 2. Indexing
    println!("Indexing strings...");
    let start_index = Instant::now();
    let corpus = vec!["foo", "bar", "fooo"];
    for item in corpus {
        db.insert(item.to_string());
    }
    println!("Indexing done in {:?}", start_index.elapsed());
    println!("DB Summary: {db:?}");

    // 3. Searching
    let searcher = Searcher::new(&db, measure);

    let query = "foo";
    let alpha = 0.8;
    println!("\n--- Ranked Search for '{query}' with alpha = {alpha} ---");

    let start_search = Instant::now();
    match searcher.ranked_search(query, alpha) {
        Ok(results) => {
            println!("Search completed in {:?}\n", start_search.elapsed());
            if results.is_empty() {
                println!("No matches found.");
            } else {
                for (item, score) in results {
                    println!("- Match: '{item}', Score: {score:.4}");
                }
            }
        }
        Err(e) => {
            eprintln!("An error occurred during search: {e}");
        }
    }
}
