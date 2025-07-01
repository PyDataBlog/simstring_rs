use crate::{
    database::{Database, HashDb},
    extractors::{CharacterNgrams, FeatureExtractor, WordNgrams},
    measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap},
    search::{SearchError as RustSearchError, Searcher as RustSearcher},
};
use pyo3::create_exception;
use pyo3::prelude::*;
use std::sync::Arc;

create_exception!(simstring_rust, SearchError, pyo3::exceptions::PyValueError);

// Wrapper for FeatureExtractor trait as I can't find any direct translation.
#[derive(Clone)]
enum PyFeatureExtractor {
    Character(CharacterNgrams),
    Word(WordNgrams),
}

impl FeatureExtractor for PyFeatureExtractor {
    fn features(&self, text: &str, interner: &mut lasso::Rodeo) -> Vec<lasso::Spur> {
        match self {
            PyFeatureExtractor::Character(e) => e.features(text, interner),
            PyFeatureExtractor::Word(e) => e.features(text, interner),
        }
    }
}

#[pyclass(name = "CharacterNgrams")]
#[derive(Clone)]
struct PyCharacterNgrams(CharacterNgrams);

#[pymethods]
impl PyCharacterNgrams {
    #[new]
    fn new(n: usize, endmarker: &str) -> Self {
        Self(CharacterNgrams::new(n, endmarker))
    }
}

#[pyclass(name = "WordNgrams")]
#[derive(Clone)]
struct PyWordNgrams(WordNgrams);

#[pymethods]
impl PyWordNgrams {
    #[new]
    fn new(n: usize, splitter: &str, padder: &str) -> Self {
        Self(WordNgrams::new(n, splitter, padder))
    }
}

// Wrapper for Measure trait
#[derive(Clone, Copy)]
enum PyMeasure {
    Cosine,
    Dice,
    ExactMatch,
    Jaccard,
    Overlap,
}

impl Measure for PyMeasure {
    fn min_feature_size(&self, query_size: usize, alpha: f64) -> usize {
        match self {
            PyMeasure::Cosine => Cosine.min_feature_size(query_size, alpha),
            PyMeasure::Dice => Dice.min_feature_size(query_size, alpha),
            PyMeasure::ExactMatch => ExactMatch.min_feature_size(query_size, alpha),
            PyMeasure::Jaccard => Jaccard.min_feature_size(query_size, alpha),
            PyMeasure::Overlap => Overlap.min_feature_size(query_size, alpha),
        }
    }

    fn max_feature_size(&self, query_size: usize, alpha: f64, db: &dyn Database) -> usize {
        match self {
            PyMeasure::Cosine => Cosine.max_feature_size(query_size, alpha, db),
            PyMeasure::Dice => Dice.max_feature_size(query_size, alpha, db),
            PyMeasure::ExactMatch => ExactMatch.max_feature_size(query_size, alpha, db),
            PyMeasure::Jaccard => Jaccard.max_feature_size(query_size, alpha, db),
            PyMeasure::Overlap => Overlap.max_feature_size(query_size, alpha, db),
        }
    }

    fn minimum_common_feature_count(&self, query_size: usize, y_size: usize, alpha: f64) -> usize {
        match self {
            PyMeasure::Cosine => Cosine.minimum_common_feature_count(query_size, y_size, alpha),
            PyMeasure::Dice => Dice.minimum_common_feature_count(query_size, y_size, alpha),
            PyMeasure::ExactMatch => {
                ExactMatch.minimum_common_feature_count(query_size, y_size, alpha)
            }
            PyMeasure::Jaccard => Jaccard.minimum_common_feature_count(query_size, y_size, alpha),
            PyMeasure::Overlap => Overlap.minimum_common_feature_count(query_size, y_size, alpha),
        }
    }

    fn similarity(&self, x: &[lasso::Spur], y: &[lasso::Spur]) -> f64 {
        match self {
            PyMeasure::Cosine => Cosine.similarity(x, y),
            PyMeasure::Dice => Dice.similarity(x, y),
            PyMeasure::ExactMatch => ExactMatch.similarity(x, y),
            PyMeasure::Jaccard => Jaccard.similarity(x, y),
            PyMeasure::Overlap => Overlap.similarity(x, y),
        }
    }
}

#[pyclass(name = "Cosine")]
#[derive(Clone, Copy)]
struct PyCosine;
#[pymethods]
impl PyCosine {
    #[new]
    fn new() -> Self {
        PyCosine
    }
}

#[pyclass(name = "Dice")]
#[derive(Clone, Copy)]
struct PyDice;
#[pymethods]
impl PyDice {
    #[new]
    fn new() -> Self {
        PyDice
    }
}

#[pyclass(name = "ExactMatch")]
#[derive(Clone, Copy)]
struct PyExactMatch;
#[pymethods]
impl PyExactMatch {
    #[new]
    fn new() -> Self {
        PyExactMatch
    }
}

#[pyclass(name = "Jaccard")]
#[derive(Clone, Copy)]
struct PyJaccard;
#[pymethods]
impl PyJaccard {
    #[new]
    fn new() -> Self {
        PyJaccard
    }
}

#[pyclass(name = "Overlap")]
#[derive(Clone, Copy)]
struct PyOverlap;
#[pymethods]
impl PyOverlap {
    #[new]
    fn new() -> Self {
        PyOverlap
    }
}

#[pyclass(name = "HashDb")]
struct PyHashDb {
    db: HashDb,
}

#[pymethods]
impl PyHashDb {
    #[new]
    fn new(extractor: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_feature_extractor =
            if let Ok(char_ngram) = extractor.extract::<PyRef<PyCharacterNgrams>>() {
                PyFeatureExtractor::Character(char_ngram.0.clone())
            } else if let Ok(word_ngram) = extractor.extract::<PyRef<PyWordNgrams>>() {
                PyFeatureExtractor::Word(word_ngram.0.clone())
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Extractor must be CharacterNgrams or WordNgrams",
                ));
            };

        let db = HashDb::new(Arc::new(py_feature_extractor));
        Ok(Self { db })
    }

    fn insert(&mut self, text: String) {
        self.db.insert(text);
    }

    fn clear(&mut self) {
        self.db.clear();
    }

    fn __len__(&self) -> usize {
        self.db.strings.len()
    }
}

#[pyclass(name = "Searcher")]
struct PySearcher {
    db: Py<PyHashDb>,
    measure: PyMeasure,
}

#[pymethods]
impl PySearcher {
    #[new]
    fn new(db: Py<PyHashDb>, measure: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_measure = if measure.is_instance_of::<PyCosine>() {
            PyMeasure::Cosine
        } else if measure.is_instance_of::<PyDice>() {
            PyMeasure::Dice
        } else if measure.is_instance_of::<PyExactMatch>() {
            PyMeasure::ExactMatch
        } else if measure.is_instance_of::<PyJaccard>() {
            PyMeasure::Jaccard
        } else if measure.is_instance_of::<PyOverlap>() {
            PyMeasure::Overlap
        } else {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "Measure must be one of Cosine, Dice, Jaccard, Overlap, ExactMatch",
            ));
        };
        Ok(Self {
            db,
            measure: py_measure,
        })
    }

    fn search<'py>(
        &self,
        py: Python<'py>,
        query_string: &str,
        alpha: f64,
    ) -> PyResult<Vec<String>> {
        let db_borrow = self.db.borrow(py);
        let searcher = RustSearcher::new(&db_borrow.db, self.measure);
        searcher.search(query_string, alpha).map_err(|e| match e {
            RustSearchError::InvalidThreshold(val) => {
                SearchError::new_err(format!("Invalid threshold: {val}"))
            }
        })
    }

    fn ranked_search<'py>(
        &self,
        py: Python<'py>,
        query_string: &str,
        alpha: f64,
    ) -> PyResult<Vec<(String, f64)>> {
        let db_borrow = self.db.borrow(py);
        let searcher = RustSearcher::new(&db_borrow.db, self.measure);
        searcher
            .ranked_search(query_string, alpha)
            .map_err(|e| match e {
                RustSearchError::InvalidThreshold(val) => {
                    SearchError::new_err(format!("Invalid threshold: {val}"))
                }
            })
    }
}

#[pymodule]
fn simstring_rust(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCharacterNgrams>()?;
    m.add_class::<PyWordNgrams>()?;
    m.add_class::<PyHashDb>()?;
    m.add_class::<PySearcher>()?;
    m.add_class::<PyCosine>()?;
    m.add_class::<PyDice>()?;
    m.add_class::<PyJaccard>()?;
    m.add_class::<PyOverlap>()?;
    m.add_class::<PyExactMatch>()?;
    m.add("SearchError", py.get_type::<SearchError>())?;
    Ok(())
}
