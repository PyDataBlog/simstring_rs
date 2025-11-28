use lasso::{Rodeo, Spur};
use rustc_hash::FxHashSet;
use simstring_rust::database::{Database, HashDb, StringId};
use simstring_rust::extractors::{CharacterNgrams, FeatureExtractor};
use simstring_rust::measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap};
use std::sync::{Arc, Mutex};

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
        assert!(
            approx_eq(score, 0.5773502691896258),
            "Cosine similarity of ['a','b','c'] and ['a','b','d','e'] should be ~0.577"
        );

        let z_str: Vec<String> = ["a", "b", "c"].iter().map(|&s| s.to_string()).collect();
        let z_spurs: Vec<_> = z_str.iter().map(|s| interner.get_or_intern(s)).collect();
        let score_exact = cosine.similarity(&x_spurs, &z_spurs);
        assert!(
            approx_eq(score_exact, 1.0),
            "Cosine similarity of identical vectors should be 1.0"
        );
    }

    #[test]
    fn test_cosine_min_feature_size() {
        let cosine = Cosine;
        let query_size = 5;
        assert_eq!(
            cosine.min_feature_size(query_size, 1.0),
            5,
            "Cosine min_feature_size with alpha=1.0 should equal query_size (alpha²*5 = 5)"
        );
        assert_eq!(
            cosine.min_feature_size(query_size, 0.5),
            2,
            "Cosine min_feature_size with query_size=5, alpha=0.5 should be 2 (ceil(0.5²*5) = 2)"
        );
    }

    #[test]
    fn test_cosine_max_feature_size() {
        let cosine = Cosine;
        let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("123456789".to_string());
        assert_eq!(
            db.max_feature_len(),
            10,
            "Database max feature length should be 10 for '123456789' with 2-grams"
        );

        let query_size = 5;
        assert_eq!(
            cosine.max_feature_size(query_size, 1.0, &db),
            5,
            "Cosine max_feature_size with alpha=1.0 should equal query_size (5/(1²) = 5)"
        );
        assert_eq!(
            cosine.max_feature_size(query_size, 0.5, &db),
            10,
            "Cosine max_feature_size with alpha=0.5 should be min(5/(0.5²), 10) = 10"
        );
    }

    #[test]
    fn test_cosine_minimum_common_feature_count() {
        let cosine = Cosine;
        let query_size = 5;
        assert_eq!(
            cosine.minimum_common_feature_count(query_size, 5, 1.0),
            5,
            "Cosine tau with query=5, y=5, alpha=1.0 should be 5 (ceil(1.0*sqrt(5*5)) = 5)"
        );
        assert_eq!(
            cosine.minimum_common_feature_count(query_size, 20, 1.0),
            10,
            "Cosine tau with query=5, y=20, alpha=1.0 should be 10 (ceil(1.0*sqrt(5*20)) = 10)"
        );
        assert_eq!(
            cosine.minimum_common_feature_count(query_size, 5, 0.5),
            3,
            "Cosine tau with query=5, y=5, alpha=0.5 should be 3 (ceil(0.5*sqrt(5*5)) = 3)"
        );
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
        assert!(
            approx_eq(score, 0.5714285714285714),
            "Dice similarity of ['a','b','c'] and ['a','b','d','e'] should be ~0.571 (2*2/(3+4))"
        );
    }

    #[test]
    fn test_dice_min_feature_size() {
        let dice = Dice;
        let query_size = 5;
        assert_eq!(
            dice.min_feature_size(query_size, 1.0),
            5,
            "Dice min_feature_size with alpha=1.0 should equal query_size (ceil(1.0/(2.0-1.0)*5) = 5)"
        );
        assert_eq!(
            dice.min_feature_size(query_size, 0.5),
            2,
            "Dice min_feature_size with query_size=5, alpha=0.5 should be 2 (ceil(0.5/(2.0-0.5)*5) = 2)"
        );
    }

    #[test]
    fn test_dice_max_feature_size() {
        let dice = Dice;
        let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("123456789012345".to_string());

        let query_size = 5;
        assert_eq!(
            dice.max_feature_size(query_size, 1.0, &db),
            5,
            "Dice max_feature_size with alpha=1.0 should equal query_size (floor((2.0-1.0)/1.0*5) = 5)"
        );
        assert_eq!(
            dice.max_feature_size(query_size, 0.5, &db),
            15,
            "Dice max_feature_size with alpha=0.5 should be min(floor((2.0-0.5)/0.5*5), 16) = 15"
        );
    }

    #[test]
    fn test_dice_minimum_common_feature_count() {
        let dice = Dice;
        let query_size = 5;
        assert_eq!(
            dice.minimum_common_feature_count(query_size, 5, 1.0),
            5,
            "Dice tau with query=5, y=5, alpha=1.0 should be 5 (ceil(0.5*1.0*(5+5)) = 5)"
        );
        assert_eq!(
            dice.minimum_common_feature_count(query_size, 5, 0.5),
            3,
            "Dice tau with query=5, y=5, alpha=0.5 should be 3 (ceil(0.5*0.5*(5+5)) = 3)"
        );
    }
}

#[cfg(test)]
mod exact_match_tests {
    use super::*;

    #[test]
    fn test_exact_match_similarity_score() {
        let mut interner = Rodeo::default();
        let measure = ExactMatch;

        let mut x_spurs: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        x_spurs.sort_unstable();
        let mut y_spurs: Vec<_> = ["c", "a", "b"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        y_spurs.sort_unstable();
        let z_spurs: Vec<_> = ["a", "b", "d"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();

        assert_eq!(
            measure.similarity(&x_spurs, &y_spurs),
            1.0,
            "ExactMatch should return 1.0 for same elements in same order (after sorting)"
        );

        assert_eq!(
            measure.similarity(&x_spurs, &z_spurs),
            0.0,
            "ExactMatch should return 0.0 for different elements"
        );
    }

    #[test]
    fn test_exact_match_size_and_overlap_bounds() {
        let measure = ExactMatch;
        let query_size = 10;

        assert_eq!(
            measure.min_feature_size(query_size, 0.5),
            query_size,
            "ExactMatch min_feature_size should always equal query_size regardless of alpha"
        );
        assert_eq!(
            measure.min_feature_size(query_size, 1.0),
            query_size,
            "ExactMatch min_feature_size should always equal query_size regardless of alpha"
        );

        let feature_extractor = Arc::new(CharacterNgrams::default());
        let db = HashDb::new(feature_extractor);
        assert_eq!(
            measure.max_feature_size(query_size, 0.5, &db),
            query_size,
            "ExactMatch max_feature_size should always equal query_size regardless of alpha"
        );
        assert_eq!(
            measure.max_feature_size(query_size, 1.0, &db),
            query_size,
            "ExactMatch max_feature_size should always equal query_size regardless of alpha"
        );

        assert_eq!(
            measure.minimum_common_feature_count(query_size, query_size, 0.5),
            query_size,
            "ExactMatch tau should always equal query_size regardless of alpha"
        );
        assert_eq!(
            measure.minimum_common_feature_count(query_size, query_size, 1.0),
            query_size,
            "ExactMatch tau should always equal query_size regardless of alpha"
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
        assert!(
            approx_eq(score, 0.4),
            "Jaccard similarity of ['a','b','c'] and ['a','b','d','e'] should be 0.4 (2/(3+4-2))"
        );
    }

    #[test]
    fn test_jaccard_min_feature_size() {
        let measure = Jaccard;
        let query_size = 5;
        assert_eq!(
            measure.min_feature_size(query_size, 1.0),
            5,
            "Jaccard min_feature_size with alpha=1.0 should equal query_size (ceil(1.0*5) = 5)"
        );
        assert_eq!(
            measure.min_feature_size(query_size, 0.5),
            3,
            "Jaccard min_feature_size with query_size=5, alpha=0.5 should be 3 (ceil(0.5*5) = 3)"
        );
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
        assert_eq!(
            measure.max_feature_size(query_size, 1.0, &db),
            5,
            "Jaccard max_feature_size with alpha=1.0 should equal query_size (floor(5/1.0) = 5)"
        );
        assert_eq!(
            measure.max_feature_size(query_size, 0.5, &db),
            10,
            "Jaccard max_feature_size with alpha=0.5 should be 10 (floor(5/0.5) = 10)"
        );
    }

    #[test]
    fn test_jaccard_minimum_common_feature_count() {
        let measure = Jaccard;
        let query_size = 5;
        assert_eq!(
            measure.minimum_common_feature_count(query_size, 5, 1.0),
            5,
            "Jaccard tau with query=5, y=5, alpha=1.0 should be 5 (ceil(1.0*(5+5)/(1.0+1.0)) = 5)"
        );
        assert_eq!(
            measure.minimum_common_feature_count(query_size, 5, 0.5),
            4,
            "Jaccard tau with query=5, y=5, alpha=0.5 should be 4 (ceil(0.5*(5+5)/(1.0+0.5)) = 4)"
        );
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

        let score = measure.similarity(&x_spurs, &y_spurs);
        assert!(
            approx_eq(score, 2.0 / 3.0),
            "Overlap similarity should be 2/3 (intersection=2, min_len=3)"
        );

        let z_spurs: Vec<_> = ["a", "b", "c"]
            .iter()
            .map(|s| interner.get_or_intern(s))
            .collect();
        let score_exact = measure.similarity(&x_spurs, &z_spurs);
        assert!(
            approx_eq(score_exact, 1.0),
            "Overlap similarity of identical vectors should be 1.0"
        );
    }

    #[test]
    fn test_overlap_min_feature_size() {
        let measure = Overlap;
        let query_size = 5;
        assert_eq!(
            measure.min_feature_size(query_size, 1.0),
            1,
            "Overlap min_feature_size should always be 1 regardless of query_size or alpha"
        );
        assert_eq!(
            measure.min_feature_size(query_size, 0.5),
            1,
            "Overlap min_feature_size should always be 1 regardless of query_size or alpha"
        );
    }

    #[test]
    fn test_overlap_max_feature_size() {
        let measure = Overlap;
        let feature_extractor = Arc::new(CharacterNgrams::new(3, "$"));
        let mut db = HashDb::new(feature_extractor);
        db.insert("fooo".to_string());
        let query_size = 5;

        assert_eq!(
            measure.max_feature_size(query_size, 1.0, &db),
            6,
            "Overlap max_feature_size should return db.max_feature_len() = 6"
        );
        assert_eq!(
            measure.max_feature_size(query_size, 0.5, &db),
            6,
            "Overlap max_feature_size should return db.max_feature_len() = 6"
        );
    }

    #[test]
    fn test_overlap_minimum_common_feature_count() {
        let measure = Overlap;
        let query_size = 5;
        assert_eq!(
            measure.minimum_common_feature_count(query_size, 5, 1.0),
            5,
            "Overlap tau with query=5, y=5, alpha=1.0 should be 5 (ceil(1.0*min(5,5)) = 5)"
        );
        assert_eq!(
            measure.minimum_common_feature_count(query_size, 20, 1.0),
            5,
            "Overlap tau with query=5, y=20, alpha=1.0 should be 5 (ceil(1.0*min(5,20)) = 5)"
        );
        assert_eq!(
            measure.minimum_common_feature_count(query_size, 5, 0.5),
            3,
            "Overlap tau with query=5, y=5, alpha=0.5 should be 3 (ceil(0.5*min(5,5)) = 3)"
        );
    }
}

fn create_dummy_db() -> HashDb {
    let feature_extractor = Arc::new(CharacterNgrams::new(2, "$"));
    HashDb::new(feature_extractor)
}

#[test]
fn test_cosine_edge_cases() {
    let measure = Cosine;
    let mut interner = Rodeo::default();
    let x = vec![interner.get_or_intern("a")];
    let empty = vec![];

    assert_eq!(
        measure.similarity(&empty, &empty),
        0.0,
        "Cosine similarity of two empty vectors should be 0.0"
    );
    assert_eq!(
        measure.similarity(&x, &empty),
        0.0,
        "Cosine similarity of non-empty and empty vectors should be 0.0"
    );
    assert_eq!(
        measure.similarity(&empty, &x),
        0.0,
        "Cosine similarity of empty and non-empty vectors should be 0.0"
    );

    let db = create_dummy_db();
    assert_eq!(
        measure.max_feature_size(5, 0.0, &db),
        0,
        "Cosine max_feature_size with alpha=0.0 should return db.max_feature_len() (0 for empty db)"
    );
}

#[test]
fn test_dice_edge_cases() {
    let measure = Dice;
    let mut interner = Rodeo::default();
    let x = vec![interner.get_or_intern("a")];
    let empty = vec![];

    assert_eq!(
        measure.similarity(&empty, &empty),
        1.0,
        "Dice similarity of two empty vectors should be 1.0"
    );
    assert_eq!(
        measure.similarity(&x, &empty),
        0.0,
        "Dice similarity of non-empty and empty vectors should be 0.0"
    );
    assert_eq!(
        measure.similarity(&empty, &x),
        0.0,
        "Dice similarity of empty and non-empty vectors should be 0.0"
    );

    assert_eq!(
        measure.min_feature_size(5, 2.1),
        0,
        "Dice min_feature_size with alpha > 2.0 should return 0"
    );

    let db = create_dummy_db();
    assert_eq!(
        measure.max_feature_size(5, 0.0, &db),
        0,
        "Dice max_feature_size with alpha=0.0 should return db.max_feature_len() (0 for empty db)"
    );
}

#[test]
fn test_jaccard_edge_cases() {
    let measure = Jaccard;
    let mut interner = Rodeo::default();
    let x = vec![interner.get_or_intern("a")];
    let empty = vec![];

    assert_eq!(
        measure.similarity(&empty, &empty),
        1.0,
        "Jaccard similarity of two empty vectors should be 1.0"
    );
    assert_eq!(
        measure.similarity(&x, &empty),
        0.0,
        "Jaccard similarity of non-empty and empty vectors should be 0.0"
    );
    assert_eq!(
        measure.similarity(&empty, &x),
        0.0,
        "Jaccard similarity of empty and non-empty vectors should be 0.0"
    );

    assert_eq!(
        measure.minimum_common_feature_count(5, 5, -1.0),
        0,
        "Jaccard tau with negative alpha should return 0"
    );
}

#[test]
fn test_overlap_edge_cases() {
    let measure = Overlap;
    let mut interner = Rodeo::default();
    let x = vec![interner.get_or_intern("a")];
    let empty = vec![];

    assert_eq!(
        measure.similarity(&empty, &empty),
        1.0,
        "Overlap similarity of two empty vectors should be 1.0"
    );
    assert_eq!(
        measure.similarity(&x, &empty),
        0.0,
        "Overlap similarity of non-empty and empty vectors should be 0.0"
    );
    assert_eq!(
        measure.similarity(&empty, &x),
        0.0,
        "Overlap similarity of empty and non-empty vectors should be 0.0"
    );
}

#[test]
fn test_cosine_zero_alpha_max_feature_size() {
    let measure = Cosine;
    let db = MockDatabase;
    assert_eq!(
        measure.max_feature_size(5, 0.0, &db),
        100,
        "Cosine max_feature_size with alpha=0.0 should return db.max_feature_len()"
    );
}

#[test]
fn test_cosine_similarity_empty_inputs() {
    let measure = Cosine;
    let empty: &[Spur] = &[];
    let non_empty = &[Spur::default()];

    assert_eq!(
        measure.similarity(empty, empty),
        0.0,
        "Cosine similarity of two empty slices should be 0.0"
    );
    assert_eq!(
        measure.similarity(empty, non_empty),
        0.0,
        "Cosine similarity of empty and non-empty slices should be 0.0"
    );
    assert_eq!(
        measure.similarity(non_empty, empty),
        0.0,
        "Cosine similarity of non-empty and empty slices should be 0.0"
    );
}

#[test]
fn test_dice_zero_alpha_max_feature_size() {
    let measure = Dice;
    let db = MockDatabase;
    assert_eq!(
        measure.max_feature_size(5, 0.0, &db),
        100,
        "Dice max_feature_size with alpha=0.0 should return db.max_feature_len()"
    );
}

#[test]
fn test_dice_similarity_empty_inputs() {
    let measure = Dice;
    let empty: &[Spur] = &[];
    let non_empty = &[Spur::default()];

    assert_eq!(
        measure.similarity(empty, empty),
        1.0,
        "Dice similarity of two empty slices should be 1.0"
    );
    assert_eq!(
        measure.similarity(empty, non_empty),
        0.0,
        "Dice similarity of empty and non-empty slices should be 0.0"
    );
    assert_eq!(
        measure.similarity(non_empty, empty),
        0.0,
        "Dice similarity of non-empty and empty slices should be 0.0"
    );
}

#[test]
fn test_jaccard_negative_alpha_min_common_features() {
    let measure = Jaccard;
    assert_eq!(
        measure.minimum_common_feature_count(5, 5, -1.0),
        0,
        "Jaccard tau with alpha=-1.0 should return 0"
    );
}

#[test]
fn test_jaccard_similarity_empty_inputs() {
    let measure = Jaccard;
    let empty: &[Spur] = &[];
    let non_empty = &[Spur::default()];

    assert_eq!(
        measure.similarity(empty, empty),
        1.0,
        "Jaccard similarity of two empty slices should be 1.0"
    );
    assert_eq!(
        measure.similarity(empty, non_empty),
        0.0,
        "Jaccard similarity of empty and non-empty slices should be 0.0"
    );
    assert_eq!(
        measure.similarity(non_empty, empty),
        0.0,
        "Jaccard similarity of non-empty and empty slices should be 0.0"
    );
}

#[test]
fn test_overlap_similarity_empty_inputs() {
    let measure = Overlap;
    let empty: &[Spur] = &[];
    let non_empty = &[Spur::default()];

    assert_eq!(
        measure.similarity(empty, empty),
        1.0,
        "Overlap similarity of two empty slices should be 1.0"
    );
    assert_eq!(
        measure.similarity(empty, non_empty),
        0.0,
        "Overlap similarity of empty and non-empty slices should be 0.0"
    );
    assert_eq!(
        measure.similarity(non_empty, empty),
        0.0,
        "Overlap similarity of non-empty and empty slices should be 0.0"
    );
}

#[cfg(test)]
mod comprehensive_edge_cases {
    use super::*;

    #[test]
    fn test_exact_match_different_lengths() {
        let measure = ExactMatch;
        let mut interner = Rodeo::default();

        let short: Vec<Spur> = vec![interner.get_or_intern("a"), interner.get_or_intern("b")];
        let long: Vec<Spur> = vec![
            interner.get_or_intern("a"),
            interner.get_or_intern("b"),
            interner.get_or_intern("c"),
        ];

        assert_eq!(
            measure.similarity(&short, &long),
            0.0,
            "ExactMatch should return 0.0 for vectors of different lengths (short vs long)"
        );
        assert_eq!(
            measure.similarity(&long, &short),
            0.0,
            "ExactMatch should return 0.0 for vectors of different lengths (long vs short)"
        );

        let empty: Vec<Spur> = vec![];
        assert_eq!(
            measure.similarity(&empty, &short),
            0.0,
            "ExactMatch should return 0.0 when comparing empty vector to non-empty"
        );
        assert_eq!(
            measure.similarity(&short, &empty),
            0.0,
            "ExactMatch should return 0.0 when comparing non-empty vector to empty"
        );
    }

    #[test]
    fn test_exact_match_same_length_different_content() {
        let measure = ExactMatch;
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![
            interner.get_or_intern("a"),
            interner.get_or_intern("b"),
            interner.get_or_intern("c"),
        ];
        let y: Vec<Spur> = vec![
            interner.get_or_intern("a"),
            interner.get_or_intern("b"),
            interner.get_or_intern("d"),
        ];

        assert_eq!(
            measure.similarity(&x, &y),
            0.0,
            "ExactMatch should return 0.0 for same-length vectors with different content"
        );
    }

    #[test]
    fn test_cosine_zero_intersection() {
        let measure = Cosine;
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![interner.get_or_intern("a"), interner.get_or_intern("b")];
        let y: Vec<Spur> = vec![interner.get_or_intern("c"), interner.get_or_intern("d")];

        assert_eq!(
            measure.similarity(&x, &y),
            0.0,
            "Cosine similarity should be 0.0 when vectors have no common elements (numerator = 0)"
        );
    }

    #[test]
    fn test_cosine_denominator_safety() {
        let measure = Cosine;
        let empty: &[Spur] = &[];

        assert_eq!(
            measure.similarity(empty, empty),
            0.0,
            "Cosine similarity should return 0.0 for two empty vectors (early return before denominator calculation)"
        );

        let mut interner = Rodeo::default();
        let single: Vec<Spur> = vec![interner.get_or_intern("x")];
        assert_eq!(
            measure.similarity(&single, &single),
            1.0,
            "Cosine similarity should be 1.0 for identical single-element vectors (denominator = sqrt(1*1) = 1.0)"
        );
    }

    #[test]
    fn test_dice_denominator_safety() {
        let measure = Dice;

        let empty: &[Spur] = &[];
        assert_eq!(
            measure.similarity(empty, empty),
            1.0,
            "Dice similarity should return 1.0 for two empty vectors (early return before denominator calculation)"
        );

        let mut interner = Rodeo::default();
        let single: Vec<Spur> = vec![interner.get_or_intern("x")];
        assert_eq!(
            measure.similarity(&single, &single),
            1.0,
            "Dice similarity should be 1.0 for identical single-element vectors (2*1/(1+1) = 1.0)"
        );
    }

    #[test]
    fn test_dice_zero_intersection() {
        let measure = Dice;
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![interner.get_or_intern("a")];
        let y: Vec<Spur> = vec![interner.get_or_intern("b")];

        assert_eq!(
            measure.similarity(&x, &y),
            0.0,
            "Dice similarity should be 0.0 for non-intersecting sets (intersection=0, 2*0/(1+1) = 0.0)"
        );
    }

    #[test]
    fn test_jaccard_union_size_safety() {
        let measure = Jaccard;
        let mut interner = Rodeo::default();

        let empty: Vec<Spur> = vec![];
        assert_eq!(
            measure.similarity(&empty, &empty),
            1.0,
            "Jaccard similarity should return 1.0 for two empty vectors (early return before union calculation)"
        );

        let x: Vec<Spur> = vec![interner.get_or_intern("a")];
        let y: Vec<Spur> = x.clone();
        assert_eq!(
            measure.similarity(&x, &y),
            1.0,
            "Jaccard similarity should be 1.0 for identical sets (intersection=1, union=1+1-1=1, 1/1=1.0)"
        );

        let z: Vec<Spur> = vec![interner.get_or_intern("b")];
        assert_eq!(
            measure.similarity(&x, &z),
            0.0,
            "Jaccard similarity should be 0.0 for non-intersecting sets (intersection=0, union=1+1-0=2, 0/2=0.0)"
        );
    }

    #[test]
    fn test_jaccard_zero_intersection() {
        let measure = Jaccard;
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![interner.get_or_intern("a"), interner.get_or_intern("b")];
        let y: Vec<Spur> = vec![interner.get_or_intern("c"), interner.get_or_intern("d")];

        assert_eq!(
            measure.similarity(&x, &y),
            0.0,
            "Jaccard similarity should be 0.0 for non-intersecting sets (intersection=0, union=2+2-0=4, 0/4=0.0)"
        );
    }

    #[test]
    fn test_overlap_denominator_safety() {
        let measure = Overlap;

        let empty: &[Spur] = &[];
        assert_eq!(
            measure.similarity(empty, empty),
            1.0,
            "Overlap similarity should return 1.0 for two empty vectors (early return before denominator calculation)"
        );

        let mut interner = Rodeo::default();
        let single: Vec<Spur> = vec![interner.get_or_intern("x")];
        assert_eq!(
            measure.similarity(&single, &single),
            1.0,
            "Overlap similarity should be 1.0 for identical single-element vectors (intersection=1, min(1,1)=1, 1/1=1.0)"
        );
    }

    #[test]
    fn test_overlap_zero_intersection() {
        let measure = Overlap;
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![
            interner.get_or_intern("a"),
            interner.get_or_intern("b"),
            interner.get_or_intern("c"),
        ];
        let y: Vec<Spur> = vec![interner.get_or_intern("d"), interner.get_or_intern("e")];

        assert_eq!(
            measure.similarity(&x, &y),
            0.0,
            "Overlap similarity should be 0.0 for non-intersecting sets (intersection=0, min(3,2)=2, 0/2=0.0)"
        );
    }

    #[test]
    fn test_all_measures_numerical_stability() {
        let mut interner = Rodeo::default();
        let single: Vec<Spur> = vec![interner.get_or_intern("x")];
        let empty: Vec<Spur> = vec![];

        assert_eq!(
            Cosine.similarity(&single, &single),
            1.0,
            "Cosine: single identical element should give 1.0"
        );
        assert_eq!(
            Dice.similarity(&single, &single),
            1.0,
            "Dice: single identical element should give 1.0"
        );
        assert_eq!(
            Jaccard.similarity(&single, &single),
            1.0,
            "Jaccard: single identical element should give 1.0"
        );
        assert_eq!(
            Overlap.similarity(&single, &single),
            1.0,
            "Overlap: single identical element should give 1.0"
        );
        assert_eq!(
            ExactMatch.similarity(&single, &single),
            1.0,
            "ExactMatch: single identical element should give 1.0"
        );

        assert_eq!(
            Cosine.similarity(&empty, &single),
            0.0,
            "Cosine: empty vs non-empty should give 0.0"
        );
        assert_eq!(
            Dice.similarity(&empty, &single),
            0.0,
            "Dice: empty vs non-empty should give 0.0"
        );
        assert_eq!(
            Jaccard.similarity(&empty, &single),
            0.0,
            "Jaccard: empty vs non-empty should give 0.0"
        );
        assert_eq!(
            Overlap.similarity(&empty, &single),
            0.0,
            "Overlap: empty vs non-empty should give 0.0"
        );
        assert_eq!(
            ExactMatch.similarity(&empty, &single),
            0.0,
            "ExactMatch: empty vs non-empty should give 0.0"
        );

        assert_eq!(
            Cosine.similarity(&single, &empty),
            0.0,
            "Cosine: symmetry check - non-empty vs empty should give 0.0"
        );
        assert_eq!(
            Dice.similarity(&single, &empty),
            0.0,
            "Dice: symmetry check - non-empty vs empty should give 0.0"
        );
        assert_eq!(
            Jaccard.similarity(&single, &empty),
            0.0,
            "Jaccard: symmetry check - non-empty vs empty should give 0.0"
        );
        assert_eq!(
            Overlap.similarity(&single, &empty),
            0.0,
            "Overlap: symmetry check - non-empty vs empty should give 0.0"
        );
        assert_eq!(
            ExactMatch.similarity(&single, &empty),
            0.0,
            "ExactMatch: symmetry check - non-empty vs empty should give 0.0"
        );
    }

    #[test]
    fn test_all_measures_with_duplicates() {
        let mut interner = Rodeo::default();

        let x: Vec<Spur> = vec![interner.get_or_intern("a"), interner.get_or_intern("b")];
        let y: Vec<Spur> = vec![interner.get_or_intern("a"), interner.get_or_intern("b")];

        assert_eq!(
            Cosine.similarity(&x, &y),
            1.0,
            "Cosine: identical two-element sets should give 1.0"
        );
        assert_eq!(
            Dice.similarity(&x, &y),
            1.0,
            "Dice: identical two-element sets should give 1.0"
        );
        assert_eq!(
            Jaccard.similarity(&x, &y),
            1.0,
            "Jaccard: identical two-element sets should give 1.0"
        );
        assert_eq!(
            Overlap.similarity(&x, &y),
            1.0,
            "Overlap: identical two-element sets should give 1.0"
        );
        assert_eq!(
            ExactMatch.similarity(&x, &y),
            1.0,
            "ExactMatch: identical two-element sets should give 1.0"
        );
    }
}

struct MockDatabase;
impl Database for MockDatabase {
    fn insert(&mut self, _text: String) {}
    fn clear(&mut self) {}
    fn lookup_strings(&self, _size: usize, _feature: Spur) -> Option<&FxHashSet<StringId>> {
        None
    }
    fn get_string(&self, _id: StringId) -> Option<&str> {
        None
    }
    fn get_features(&self, _id: StringId) -> Option<&Vec<Spur>> {
        None
    }
    fn feature_extractor(&self) -> &dyn FeatureExtractor {
        unimplemented!()
    }
    fn max_feature_len(&self) -> usize {
        100
    }
    fn interner(&self) -> Arc<Mutex<Rodeo>> {
        unimplemented!()
    }
    fn total_strings(&self) -> usize {
        0
    }
}
