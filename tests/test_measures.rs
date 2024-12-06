use simstring_rust::database::{HashDB, SimStringDB};
use simstring_rust::extractors::CharacterNGrams;
use simstring_rust::measures::{Cosine, Dice, ExactMatch, Jaccard, Overlap, SimilarityMeasure};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_scores() {
        let x = vec![1, 2, 3];
        let y = vec![1, 2, 4, 5];

        // Dice Similarity
        let dice = Dice::new();
        let dice_score = dice.similarity_score(&x, &y);
        assert!(
            (dice_score - 0.5714285714285714).abs() < 1e-9,
            "Expected Dice similarity to be approximately 0.5714285714285714, got {}",
            dice_score
        );

        // Jaccard Similarity
        let jaccard = Jaccard::new();
        let jaccard_score = jaccard.similarity_score(&x, &y);
        assert!(
            (jaccard_score - 0.4).abs() < 1e-9,
            "Expected Jaccard similarity to be approximately 0.4, got {}",
            jaccard_score
        );

        // Cosine Similarity
        let cosine = Cosine::new();
        let cosine_score = cosine.similarity_score(&x, &y);
        assert!(
            (cosine_score - 0.5773502691896258).abs() < 1e-9,
            "Expected Cosine similarity to be approximately 0.5773502691896258, got {}",
            cosine_score
        );

        // Overlap Similarity
        let overlap = Overlap::new();
        let overlap_score = overlap.similarity_score(&x, &y);
        assert!(
            (overlap_score - 0.6666666666666666).abs() < 1e-9,
            "Expected Overlap similarity to be approximately 0.6666666666666666, got {}",
            overlap_score
        );

        // Exact Match Similarity
        let exact_match = ExactMatch::new();
        let exact_match_score = exact_match.similarity_score(&x, &y);
        assert_eq!(
            exact_match_score, 0.0,
            "Expected Exact Match similarity to be 0.0, got {}",
            exact_match_score
        );
    }

    #[test]
    fn test_minimum_candidate_feature_size() {
        let query_size = 5;

        // Dice Measure
        let dice = Dice::new();
        assert_eq!(
            dice.minimum_feature_size(query_size, 1.0),
            5,
            "Expected minimum feature size for Dice (alpha=1.0) to be 5"
        );
        assert_eq!(
            dice.minimum_feature_size(query_size, 0.5),
            2,
            "Expected minimum feature size for Dice (alpha=0.5) to be 2"
        );

        // Jaccard Measure
        let jaccard = Jaccard::new();
        assert_eq!(
            jaccard.minimum_feature_size(query_size, 1.0),
            5,
            "Expected minimum feature size for Jaccard (alpha=1.0) to be 5"
        );
        assert_eq!(
            jaccard.minimum_feature_size(query_size, 0.5),
            3,
            "Expected minimum feature size for Jaccard (alpha=0.5) to be 3"
        );

        // Cosine Measure
        let cosine = Cosine::new();
        assert_eq!(
            cosine.minimum_feature_size(query_size, 1.0),
            5,
            "Expected minimum feature size for Cosine (alpha=1.0) to be 5"
        );
        assert_eq!(
            cosine.minimum_feature_size(query_size, 0.5),
            2,
            "Expected minimum feature size for Cosine (alpha=0.5) to be 2"
        );

        // Overlap Measure
        let overlap = Overlap::new();
        assert_eq!(
            overlap.minimum_feature_size(query_size, 1.0),
            1,
            "Expected minimum feature size for Overlap (alpha=1.0) to be 1"
        );
        assert_eq!(
            overlap.minimum_feature_size(query_size, 0.5),
            1,
            "Expected minimum feature size for Overlap (alpha=0.5) to be 1"
        );

        // Exact Match Measure
        let exact_match = ExactMatch::new();
        assert_eq!(
            exact_match.minimum_feature_size(query_size, 1.0),
            5,
            "Expected minimum feature size for Exact Match (alpha=1.0) to be 5"
        );
        assert_eq!(
            exact_match.minimum_feature_size(query_size, 0.5),
            5,
            "Expected minimum feature size for Exact Match (alpha=0.5) to be 5"
        );
    }

    #[test]
    fn test_maximum_candidate_feature_size() {
        let feature_extractor = CharacterNGrams {
            n: 3,
            padder: " ".to_string(),
        };
        let mut db = HashDB::new(feature_extractor);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let query_size = 5;

        // Dice Measure
        let dice = Dice::new();
        assert_eq!(
            dice.maximum_feature_size(&db, query_size, 1.0),
            5,
            "Expected maximum feature size for Dice (alpha=1.0) to be 5"
        );
        assert_eq!(
            dice.maximum_feature_size(&db, query_size, 0.5),
            15,
            "Expected maximum feature size for Dice (alpha=0.5) to be 15"
        );

        // Jaccard Measure
        let jaccard = Jaccard::new();
        assert_eq!(
            jaccard.maximum_feature_size(&db, query_size, 1.0),
            5,
            "Expected maximum feature size for Jaccard (alpha=1.0) to be 5"
        );
        assert_eq!(
            jaccard.maximum_feature_size(&db, query_size, 0.5),
            10,
            "Expected maximum feature size for Jaccard (alpha=0.5) to be 10"
        );

        // Cosine Measure
        let cosine = Cosine::new();
        assert_eq!(
            cosine.maximum_feature_size(&db, query_size, 1.0),
            5,
            "Expected maximum feature size for Cosine (alpha=1.0) to be 5"
        );
        assert_eq!(
            cosine.maximum_feature_size(&db, query_size, 0.5),
            20,
            "Expected maximum feature size for Cosine (alpha=0.5) to be 20"
        );

        // Overlap Measure
        let overlap = Overlap::new();
        assert_eq!(
            overlap.maximum_feature_size(&db, query_size, 1.0),
            6,
            "Expected maximum feature size for Overlap (alpha=1.0) to be 6"
        );
        assert_eq!(
            overlap.maximum_feature_size(&db, query_size, 0.5),
            6,
            "Expected maximum feature size for Overlap (alpha=0.5) to be 6"
        );

        // Exact Match Measure
        let exact_match = ExactMatch::new();
        assert_eq!(
            exact_match.maximum_feature_size(&db, query_size, 1.0),
            5,
            "Expected maximum feature size for Exact Match (alpha=1.0) to be 5"
        );
        assert_eq!(
            exact_match.maximum_feature_size(&db, query_size, 0.5),
            5,
            "Expected maximum feature size for Exact Match (alpha=0.5) to be 5"
        );
    }

    #[test]
    fn test_minimum_feature_overlap() {
        let query_size = 5;

        // Dice Measure
        let dice = Dice::new();
        assert_eq!(
            dice.minimum_overlap(query_size, 5, 1.0),
            5,
            "Expected minimum overlap for Dice (query_size=5, candidate_size=5, alpha=1.0) to be 5"
        );
        assert_eq!(
            dice.minimum_overlap(query_size, 20, 1.0),
            13,
            "Expected minimum overlap for Dice (query_size=5, candidate_size=20, alpha=1.0) to be 13"
        );
        assert_eq!(
            dice.minimum_overlap(query_size, 5, 0.5),
            3,
            "Expected minimum overlap for Dice (query_size=5, candidate_size=5, alpha=0.5) to be 3"
        );

        // Jaccard Measure
        let jaccard = Jaccard::new();
        assert_eq!(
            jaccard.minimum_overlap(query_size, 5, 1.0),
            5,
            "Expected minimum overlap for Jaccard (query_size=5, candidate_size=5, alpha=1.0) to be 5"
        );
        assert_eq!(
            jaccard.minimum_overlap(query_size, 20, 1.0),
            13,
            "Expected minimum overlap for Jaccard (query_size=5, candidate_size=20, alpha=1.0) to be 13"
        );
        assert_eq!(
            jaccard.minimum_overlap(query_size, 5, 0.5),
            4,
            "Expected minimum overlap for Jaccard (query_size=5, candidate_size=5, alpha=0.5) to be 4"
        );

        // Cosine Measure
        let cosine = Cosine::new();
        assert_eq!(
            cosine.minimum_overlap(query_size, 5, 1.0),
            5,
            "Expected minimum overlap for Cosine (query_size=5, candidate_size=5, alpha=1.0) to be 5"
        );
        assert_eq!(
            cosine.minimum_overlap(query_size, 20, 1.0),
            10,
            "Expected minimum overlap for Cosine (query_size=5, candidate_size=20, alpha=1.0) to be 10"
        );
        assert_eq!(
            cosine.minimum_overlap(query_size, 5, 0.5),
            3,
            "Expected minimum overlap for Cosine (query_size=5, candidate_size=5, alpha=0.5) to be 3"
        );

        // Overlap Measure
        let overlap = Overlap::new();
        assert_eq!(
            overlap.minimum_overlap(query_size, 5, 1.0),
            5,
            "Expected minimum overlap for Overlap (query_size=5, candidate_size=5, alpha=1.0) to be 5"
        );
        assert_eq!(
            overlap.minimum_overlap(query_size, 20, 1.0),
            5,
            "Expected minimum overlap for Overlap (query_size=5, candidate_size=20, alpha=1.0) to be 5"
        );
        assert_eq!(
            overlap.minimum_overlap(query_size, 5, 0.5),
            3,
            "Expected minimum overlap for Overlap (query_size=5, candidate_size=5, alpha=0.5) to be 3"
        );

        // Exact Match Measure
        let exact_match = ExactMatch::new();
        assert_eq!(
            exact_match.minimum_overlap(query_size, 5, 1.0),
            5,
            "Expected minimum overlap for Exact Match (query_size=5, candidate_size=5, alpha=1.0) to be 5"
        );
        assert_eq!(
            exact_match.minimum_overlap(query_size, 20, 1.0),
            5,
            "Expected minimum overlap for Exact Match (query_size=5, candidate_size=20, alpha=1.0) to be 5"
        );
    }
}
