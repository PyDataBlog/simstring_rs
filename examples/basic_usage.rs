use simstring_rust::database::{HashDB, SimStringDB};
use simstring_rust::extractors::{CharacterNGrams, FeatureExtractor};
use simstring_rust::measures::Cosine;

fn main() {
    let _cs = Cosine::new();

    let feature_extractor = CharacterNGrams {
        n: 3,
        padder: " ".to_string(),
    };

    let mut db = HashDB::new(feature_extractor);

    db.insert("hello".to_string());
    db.insert("help".to_string());
    db.insert("halo".to_string());
    db.insert("world".to_string());

    let (total_collection, avg_size_ngrams, total_ngrams) = db.describe_collection();
    println!(
        "Database contains {} strings, average n-gram size {:.2}, total n-grams {}.",
        total_collection, avg_size_ngrams, total_ngrams
    );

    //println!("Complete DB State: {:?}", db); # FIX: db needs a fmt.debug implementation

    let query = "prepress";

    let query_features = db.feature_extractor.extract(query);
    let query_size = query_features.len();

    println!("Query size: {}", query_size);

    println!("Extracted features from query '{}':", query);
    for (feature, count) in &query_features {
        println!(" - Feature: '{}', Count: {}", feature, count);
    }
}
