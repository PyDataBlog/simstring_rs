use simstring_rs::{measures::Cosine, SimilarityMeasure};

#[test]
fn test_similarity_scores() {
    let cosine = Cosine::new();

    let x = vec![1, 2, 3];
    let y = vec![1, 2, 4, 5];

    assert_eq!(
        cosine.similarity_score(&x.clone(), &y.clone()),
        0.5773502691896258
    );
}
