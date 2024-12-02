use super::FeatureExtractor;

pub struct WordNGrams {
    n: i16,
    padder: String,
    splitter: String,
}

impl FeatureExtractor for WordNGrams {
    fn extract(&self, s: String) -> Vec<(&str, i32)> {
        todo!()
    }
}
