use crate::SimStringDB;
use crate::SimilarityMeasure;

pub fn search<M, DB>(
    measure: &M,
    db: &DB,
    query: &str,
    alpha: f64,
    ranked: bool,
) -> Vec<(String, f64)>
where
    M: SimilarityMeasure,
    DB: SimStringDB,
{
    todo!()
}
