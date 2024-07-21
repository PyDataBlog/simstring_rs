use simstring_rs::{measures::Cosine, Dice, ExactMatch, Jaccard, Overlap, SimilarityMeasure};

#[test]
fn test_cosine_measures() {
    let cosine = Cosine::new();

    let x = vec![1, 2, 3];
    let y = vec![1, 2, 4, 5];

    assert_eq!(
        cosine.similarity_score(&x.clone(), &y.clone()),
        0.5773502691896258
    );

    assert_eq!(cosine.minimum_feature_size(5, 1.), 5);
    assert_eq!(cosine.minimum_feature_size(5, 0.5), 2);

    assert_eq!(cosine.maximum_feature_size(5, 1.), 5);
    assert_eq!(cosine.maximum_feature_size(5, 0.5), 20);

    assert_eq!(cosine.minimum_overlap(5, 5, 1.), 5);
    assert_eq!(cosine.minimum_overlap(5, 20, 1.), 10);
    assert_eq!(cosine.minimum_overlap(5, 5, 0.5), 3);
}

#[test]
fn test_dice_measures() {
    let dice = Dice::new();

    let x = vec![1, 2, 3];
    let y = vec![1, 2, 4, 5];

    assert_eq!(
        dice.similarity_score(&x.clone(), &y.clone()),
        0.5714285714285714
    );

    assert_eq!(dice.minimum_feature_size(5, 1.), 5);
    assert_eq!(dice.minimum_feature_size(5, 0.5), 2);

    assert_eq!(dice.maximum_feature_size(5, 1.), 5);
    assert_eq!(dice.maximum_feature_size(5, 0.5), 15);

    assert_eq!(dice.minimum_overlap(5, 5, 1.), 13);
    assert_eq!(dice.minimum_overlap(5, 20, 1.), 50);
    assert_eq!(dice.minimum_overlap(5, 5, 0.5), 7);
}

#[test]
fn test_exact_match_measures() {
    let exact_match = ExactMatch::new();

    let x = vec![1, 2, 3];
    let y = vec![1, 2, 4, 5];

    assert_eq!(exact_match.similarity_score(&x.clone(), &y.clone()), 0.0);

    assert_eq!(exact_match.minimum_feature_size(5, 1.), 5);
    assert_eq!(exact_match.minimum_feature_size(5, 0.5), 5);

    assert_eq!(exact_match.maximum_feature_size(5, 1.), 5);
    assert_eq!(exact_match.maximum_feature_size(5, 0.5), 5);

    assert_eq!(exact_match.minimum_overlap(5, 5, 1.), 5);
    assert_eq!(exact_match.minimum_overlap(5, 20, 1.), 5);
}

#[test]
fn test_jacard_measures() {
    let jaccard = Jaccard::new();

    let x = vec![1, 2, 3];
    let y = vec![1, 2, 4, 5];

    assert_eq!(jaccard.similarity_score(&x.clone(), &y.clone()), 0.4);

    assert_eq!(jaccard.minimum_feature_size(5, 1.), 5);
    assert_eq!(jaccard.minimum_feature_size(5, 0.5), 2);

    assert_eq!(jaccard.maximum_feature_size(5, 1.), 5);
    assert_eq!(jaccard.maximum_feature_size(5, 0.5), 15);

    assert_eq!(jaccard.minimum_overlap(5, 5, 1.), 13);
    assert_eq!(jaccard.minimum_overlap(5, 20, 1.), 50);
    assert_eq!(jaccard.minimum_overlap(5, 5, 0.5), 7);
}

#[test]
fn test_overlap_measures() {
    let overlap = Overlap::new();

    let x = vec![3, 2, 3];
    let y = vec![3, 2, 4, 5];

    assert_eq!(
        overlap.similarity_score(&x.clone(), &y.clone()),
        0.6666666666666666
    );

    assert_eq!(overlap.minimum_feature_size(5, 1.), 5);
    assert_eq!(overlap.minimum_feature_size(5, 0.5), 2);

    assert_eq!(overlap.maximum_feature_size(5, 1.), 5);
    assert_eq!(overlap.maximum_feature_size(5, 0.5), 15);

    assert_eq!(overlap.minimum_overlap(5, 5, 1.), 13);
    assert_eq!(overlap.minimum_overlap(5, 20, 1.), 50);
    assert_eq!(overlap.minimum_overlap(5, 5, 0.5), 7);
}
