mod cosine;
mod dice;
mod exact_match;
mod jaccard;
mod overlap;

pub trait SimilarityMeasure {
    fn minimum_feature_size(&self, query_size: i64, alpha: f64) -> i64;
    fn maximum_feature_size(&self, query_size: i64, alpha: f64) -> i64;
    fn similarity_score(&self, x: &Vec<i64>, y: &Vec<i64>) -> f64;
    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64;
}

pub use cosine::Cosine;
pub use dice::Dice;
pub use exact_match::ExactMatch;
pub use jaccard::Jaccard;
pub use overlap::Overlap;
