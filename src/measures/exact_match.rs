use super::SimilarityMeasure;

pub struct ExactMatch;

impl ExactMatch {
    pub fn new() -> Self {
        ExactMatch
    }
}

impl SimilarityMeasure for ExactMatch {
    fn minimum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        todo!()
    }

    fn maximum_feature_size(&self, query_size: i64, alpha: f64) -> i64 {
        todo!()
    }

    fn similarity_score(&self, x: &Vec<i64>, y: &Vec<i64>) -> f64 {
        todo!()
    }

    fn minimum_overlap(&self, query_size: i64, candidate_size: i64, alpha: f64) -> i64 {
        todo!()
    }
}
