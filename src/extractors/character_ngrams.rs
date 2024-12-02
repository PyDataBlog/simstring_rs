use super::FeatureExtractor;

pub struct CharacterNGrams {
    n: i16,
    padder: String,
}

impl FeatureExtractor for CharacterNGrams {
    fn extract(&self, s: String) -> Vec<(&str, i32)> {
        todo!()
    }
}
