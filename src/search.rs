use crate::SimStringDB;
use crate::SimilarityMeasure;

pub fn search<M, DB>(
    _measure: &M,
    _db: &DB,
    _query: &str,
    _alpha: f64,
    _ranked: bool,
) -> Vec<(String, f64)>
where
    M: SimilarityMeasure,
    DB: SimStringDB,
{
    todo!()
}
