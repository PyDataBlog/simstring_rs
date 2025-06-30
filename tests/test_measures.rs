use lasso::Rodeo;
use simstring_rust::{
    CharacterNgrams, Cosine, Database, Dice, ExactMatch, HashDb, Jaccard, Measure, Overlap,
};
use std::sync::Arc;

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[cfg(test)]
mod cosine_tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_score() {
        let mut interner = Rodeo::default();
        let cosine = Cosine;
        let x_str: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
        let y_str: Vec<String> = ["a", "b", "d", "e"]
            .iter()
            .map(|&s| s.to_string())
            .collect();

        let x_spurs: Vec<_> = x_str.iter().map(|s| interner.get_or_intern(s)).collect();
        let y_spurs: Vec<_> = y_str.iter().map(|s| interner.get_or_intern(s)).collect();

        let score = cosine.similarity(&x_spurs, &y_spurs);
        assert!(approx_eq(score, 0.5773502691896258));

        let z_str: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
        let z_spurs: Vec<_> = z_str.iter().map(|s| interner.get_or_intern(s)).collect();
        let score_exact = cosine.similarity(&x_spurs, &z_spurs);
        assert!(approx_eq(score_exact, 1.0));
    }

    #[test]
    fn test_cosine_min_feature_size() {
        let cosine = Cosine;
        let query_size = 5;
        assert_eq!(cosine.min_feature_size(query_size, 1.0), 5);
        assert_eq!(cosine.min_feature_size(query_size, 0.5), 2);
    }

    #[test]
    fn test_cosine_max_feature_size() {
        let cosine = Cosine;
        let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("123456789".to_string());
        assert_eq!(db.max_feature_len(), 10);

        let query_size = 5;
        assert_eq!(cosine.max_feature_size(query_size, 1.0, &db), 5);
        assert_eq!(cosine.max_feature_size(query_size, 0.5, &db), 10);
    }

    #[test]
    fn test_cosine_minimum_common_feature_count() {
        let cosine = Cosine;
        let query_size = 5;
        assert_eq!(cosine.minimum_common_feature_count(query_size, 5, 1.0), 5);
        assert_eq!(cosine.minimum_common_feature_count(query_size, 20, 1.0), 10);
        assert_eq!(cosine.minimum_common_feature_count(query_size, 5, 0.5), 3);
    }
}

#[cfg(test)]
mod dice_tests {
    use super::*;

    #[test]
    fn test_dice_similarity_score() {
        let mut interner = Rodeo::default();
        let dice = Dice;
        let x_str: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
        let y_str: Vec<String> = ["a", "b", "d", "e"]
            .iter()
            .map(|&s| s.to_string())
            .collect();

        let x_spurs: Vec<_> = x_str.iter().map(|s| interner.get_or_intern(s)).collect();
        let y_spurs: Vec<_> = y_str.iter().map(|s| interner.get_or_intern(s)).collect();

        let score = dice.similarity(&x_spurs, &y_spurs);
        assert!(approx_eq(score, 0.5714285714285714));
    }

    #[test]
    fn test_dice_min_feature_size() {
        let dice = Dice;
        let query_size = 5;
        assert_eq!(dice.min_feature_size(query_size, 1.0), 5);
        assert_eq!(dice.min_feature_size(query_size, 0.5), 2);
    }

    #[test]
    fn test_dice_max_feature_size() {
        let dice = Dice;
        let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("123456789012345".to_string());

        let query_size = 5;
        assert_eq!(dice.max_feature_size(query_size, 1.0, &db), 5);
        assert_eq!(dice.max_feature_size(query_size, 0.5, &db), 15);
    }

    #[test]
    fn test_dice_minimum_common_feature_count() {
        let dice = Dice;
        let query_size = 5;
        assert_eq!(dice.minimum_common_feature_count(query_size, 5, 1.0), 5);
        assert_eq!(dice.minimum_common_feature_count(query_size, 5, 0.5), 3);
    }
}

#[cfg(test)]
mod exact_match_tests {
    use super::*;

    #[test]
    fn test_exact_match_similarity_score() {
        let mut interner = Rodeo::default();
        let measure = ExactMatch;

        let x_spurs: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        let y_spurs: Vec<_> = ["c", "a", "b"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect(); // Same elements, different order
        let z_spurs: Vec<_> = ["a", "b", "d"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect(); // Different elements

        assert_eq!(measure.similarity(&x_spurs, &y_spurs), 1.0);

        assert_eq!(measure.similarity(&x_spurs, &z_spurs), 0.0);
    }

    #[test]
    fn test_exact_match_size_and_overlap_bounds() {
        let measure = ExactMatch;
        let query_size = 10;

        assert_eq!(measure.min_feature_size(query_size, 0.5), query_size);
        assert_eq!(measure.min_feature_size(query_size, 1.0), query_size);

        let feature_extractor = Arc::new(CharacterNgrams::default());
        let db = HashDb::new(feature_extractor);
        assert_eq!(measure.max_feature_size(query_size, 0.5, &db), query_size);
        assert_eq!(measure.max_feature_size(query_size, 1.0, &db), query_size);

        assert_eq!(
            measure.minimum_common_feature_count(query_size, query_size, 0.5),
            query_size
        );
        assert_eq!(
            measure.minimum_common_feature_count(query_size, query_size, 1.0),
            query_size
        );
    }
}

#[cfg(test)]
mod jaccard_tests {
    use super::*;

    #[test]
    fn test_jaccard_similarity_score() {
        let mut interner = Rodeo::default();
        let measure = Jaccard;
        let x_str: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        let y_str: Vec<String> = ["a", "b", "d", "e"].iter().map(|s| s.to_string()).collect();

        let x_spurs: Vec<_> = x_str.iter().map(|s| interner.get_or_intern(s)).collect();
        let y_spurs: Vec<_> = y_str.iter().map(|s| interner.get_or_intern(s)).collect();

        let score = measure.similarity(&x_spurs, &y_spurs);
        assert!(approx_eq(score, 0.4));
    }

    #[test]
    fn test_jaccard_min_feature_size() {
        let measure = Jaccard;
        let query_size = 5;
        assert_eq!(measure.min_feature_size(query_size, 1.0), 5);
        assert_eq!(measure.min_feature_size(query_size, 0.5), 3);
    }

    #[test]
    fn test_jaccard_max_feature_size() {
        let measure = Jaccard;
        let feature_extractor = Arc::new(CharacterNgrams::default());
        let mut db = HashDb::new(feature_extractor);

        db.insert("foo".to_string());
        db.insert("bar".to_string());
        db.insert("fooo".to_string());

        let query_size = 5;
        assert_eq!(measure.max_feature_size(query_size, 1.0, &db), 5);
        assert_eq!(measure.max_feature_size(query_size, 0.5, &db), 10);
    }

    #[test]
    fn test_jaccard_minimum_common_feature_count() {
        let measure = Jaccard;
        let query_size = 5;
        assert_eq!(measure.minimum_common_feature_count(query_size, 5, 1.0), 5);
        assert_eq!(measure.minimum_common_feature_count(query_size, 5, 0.5), 4);
    }
}

#[cfg(test)]
mod overlap_tests {
    use super::*;

    #[test]
    fn test_overlap_similarity_score() {
        let mut interner = Rodeo::default();
        let measure = Overlap;
        let x_spurs: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        let y_spurs: Vec<_> = ["a", "b", "d", "e"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();

        // intersection is 2, min_len is 3. score = 2/3
        let score = measure.similarity(&x_spurs, &y_spurs);
        assert!(approx_eq(score, 2.0 / 3.0));

        let z_spurs: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        let score_exact = measure.similarity(&x_spurs, &z_spurs);
        assert!(approx_eq(score_exact, 1.0));
    }

    #[test]
    fn test_overlap_min_feature_size() {
        let measure = Overlap;
        let query_size = 5;
        assert_eq!(measure.min_feature_size(query_size, 1.0), 1);
        assert_eq!(measure.min_feature_size(query_size, 0.5), 1);
    }

    #[test]
    fn test_overlap_max_feature_size() {
        let measure = Overlap;
        let feature_extractor = Arc::new(CharacterNgrams::new(3, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("fooo".to_string()); // feature length is 6
        let query_size = 5;

        assert_eq!(measure.max_feature_size(query_size, 1.0, &db), 6);
        assert_eq!(measure.max_feature_size(query_size, 0.5, &db), 6);
    }

    #[test]
    fn test_overlap_minimum_common_feature_count() {
        let measure = Overlap;
        let query_size = 5;
        assert_eq!(measure.minimum_common_feature_count(query_size, 5, 1.0), 5);
        assert_eq!(measure.minimum_common_feature_count(query_size, 20, 1.0), 5);
        assert_eq!(measure.minimum_common_feature_count(query_size, 5, 0.5), 3);
    }
}
