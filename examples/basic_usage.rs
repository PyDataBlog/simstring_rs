use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::Cosine;
use std::env;

fn main() {
    let feature_extractor = CharacterNGrams {
        n: 2,
        padder: " ".to_string(),
    };
    let measure = Cosine::new();
    let mut db = HashDB::new(feature_extractor, measure);

    let current_dir = env::current_dir().unwrap();
    let file_path = current_dir.join("examples").join("data").join("example_data.txt");
    db.build_from_file(file_path.to_str().unwrap()).unwrap();

    let threshold = 0.5;
    let results = db.search("hell", threshold);

    if results.is_empty() {
        println!("No results found with threshold {}", threshold);
    } else {
        println!("Results with threshold {}:", threshold);
        for result in results {
            println!("Match: '{}' (score: {})", result.value, result.score);
        }
    }
}
