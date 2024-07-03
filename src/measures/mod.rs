mod cosine;
mod dice;
mod exact_match;
mod jaccard;
mod overlap;

pub trait SimilarityMeasure {
    fn minimum_feature_size(arg: Type) -> f64 {
        todo!();
    }
    fn maximum_feature_size(arg: Type) -> f64 {
        todo!();
    }
    fn similarity_score(arg: Type) -> f64 {
        todo!();
    }
    fn minimum_overlap(arg: Type) -> f64 {
        todo!();
    }
}
