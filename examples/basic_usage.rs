use simstring_rust::database::HashDB;
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::Cosine;

fn main() {
    let feature_extractor = CharacterNGrams {
        n: 2,
        padder: " ".to_string(),
    };
    let measure = Cosine::new();
    let mut db = HashDB::new(feature_extractor, measure);

    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("halo".to_string());
    db.insert("world".to_string());

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
