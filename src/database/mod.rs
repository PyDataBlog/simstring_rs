mod hashdb;

use crate::measures::SimilarityMeasure;
use crate::search::SearchResult;

pub trait SimStringDB<TMeasure: SimilarityMeasure> {
    fn insert(&mut self, s: String);
    fn describe_collection(&self) -> (usize, f64, usize);
    fn get_max_feature_size(&self) -> i64;
    fn search(&mut self, query: &str, threshold: f64) -> Vec<SearchResult<String>>;
}

pub use hashdb::HashDB;
