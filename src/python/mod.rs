//! Python bindings for the crate, exposed by the [`simstring-rust`][pypi]
//! PyPI package (imported as `simstring_rust`).
//!
//! The PyO3 extension module mirrors `Searcher`, `Database`, and their
//! helper types for Python consumers: `CharacterNgrams`, `WordNgrams`, and
//! `CustomExtractor` under `extractors`; `Cosine`, `Dice`, `ExactMatch`,
//! `Jaccard`, and `Overlap` under `measures`; `HashDb` under `database`;
//! `Searcher` under `searcher`; and the `SearchError` exception under
//! `errors`. User-facing docstrings accompany every exported class and
//! method, so `help(simstring_rust.searcher.Searcher)` documents the same
//! behavior as the Rust docs.
//!
//! [pypi]: https://pypi.org/project/simstring-rust/

use crate::{
    database::{Database, HashDb, PersistenceError},
    extractors::{CharacterNgrams, ExtractorConfig, FeatureExtractor, WordNgrams},
    measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap},
    search::{SearchError as RustSearchError, Searcher as RustSearcher},
};
use pyo3::create_exception;
use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;

/// Docstring of the root `simstring_rust` module.
const MODULE_DOC: &str = "Native approximate string matching (the SimString algorithm) in Rust.\n\
\n\
Strings are indexed by feature sets (character or word n-grams, or your own\n\
via ``CustomExtractor``) and threshold queries ask: which strings are at\n\
least ``alpha`` similar to my query? Retrieval is exact per the CPMerge\n\
algorithm: no valid matches are missed and none are fabricated.\n\
\n\
Submodules:\n\
    database: ``HashDb``, the in-memory string + inverted-index store.\n\
    extractors: ``CharacterNgrams``, ``WordNgrams``, ``CustomExtractor``.\n\
    measures: ``Cosine``, ``Dice``, ``ExactMatch``, ``Jaccard``, ``Overlap``.\n\
    searcher: ``Searcher`` with ``search()`` and ``ranked_search()``.\n\
    errors: the ``SearchError`` exception.\n\
\n\
Typical usage::\n\
\n\
    from simstring_rust.database import HashDb\n\
    from simstring_rust.extractors import CharacterNgrams\n\
    from simstring_rust.measures import Cosine\n\
    from simstring_rust.searcher import Searcher\n\
\n\
    db = HashDb(CharacterNgrams(2, '$'))\n\
    db.insert('apple')\n\
    Searcher(db, Cosine()).search('appel', 0.8)";

/// Docstring of the `simstring_rust.database` submodule.
const DATABASE_DOC: &str = "In-memory string database with an inverted feature index.\n\
\n\
Classes:\n\
    HashDb: stores strings, tokenizing each with a feature extractor.";

/// Docstring of the `simstring_rust.extractors` submodule.
const EXTRACTORS_DOC: &str = "Feature extractors: tokenizing text into unique feature strings.\n\
\n\
Classes:\n\
    CharacterNgrams: overlapping character windows over an end-marked text.\n\
    WordNgrams: token windows over a space-split, padded word sequence.\n\
    CustomExtractor: wrap any Python ``apply(text) -> Iterable[str]`` class.";

/// Docstring of the `simstring_rust.measures` submodule.
const MEASURES_DOC: &str = "Similarity measures scoring two feature sets in ``0.0..=1.0``.\n\
\n\
Classes:\n\
    Cosine: ``common / sqrt(x * y)`` (balanced default).\n\
    Dice: ``2 * common / (x + y)`` (original SimString default).\n\
    ExactMatch: ``1.0`` only for identical feature sets.\n\
    Jaccard: ``common / (x + y - common)`` (strictest set measure).\n\
    Overlap: ``common / min(x, y)`` (subset-friendly, most permissive).";

/// Docstring of the `simstring_rust.searcher` submodule.
const SEARCHER_DOC: &str = "Threshold search engines over a ``HashDb``.\n\
\n\
Classes:\n\
    Searcher: exact threshold search with ``search()`` and ``ranked_search()``.";

/// Docstring of the `simstring_rust.errors` submodule.
const ERRORS_DOC: &str = "Exceptions raised by the package.\n\
\n\
Exceptions:\n\
    SearchError: raised for thresholds outside ``(0.0, 1.0]``.";

create_exception!(
    simstring_rust,
    SearchError,
    pyo3::exceptions::PyValueError,
    "Raised for invalid search thresholds. Catch this when calling\n\
     ``Searcher.search`` / ``Searcher.ranked_search`` with an `alpha`\n\
     outside the valid range ``0.0 < alpha <= 1.0`` (for example ``0.0``).\n\
     Derives from ``ValueError``."
);

#[derive(Clone)]
struct CustomExtractorInner {
    extractor: Arc<Py<PyAny>>,
}

impl CustomExtractorInner {
    fn new(extractor: Py<PyAny>) -> Self {
        Self {
            extractor: Arc::new(extractor),
        }
    }

    fn collect_raw_features(&self, text: &str) -> PyResult<Vec<String>> {
        let extractor = Arc::clone(&self.extractor);
        Python::attach(|py| {
            let extractor = extractor.bind(py);
            let result = extractor.call_method1("apply", (text,))?;
            let iter = result.try_iter()?;

            let mut features = Vec::new();
            for item in iter {
                let item = item?;
                features.push(item.extract::<String>()?);
            }

            Ok(features)
        })
    }

    fn apply(&self, text: &str) -> PyResult<Vec<String>> {
        let raw = self.collect_raw_features(text)?;
        let mut features = Vec::with_capacity(raw.len());
        crate::extractors::visit_counted_features(&raw, &mut |feature| {
            features.push(feature.to_owned())
        });
        Ok(features)
    }
}

// Wrapper for FeatureExtractor trait as I can't find any direct translation.
#[derive(Clone)]
enum PyFeatureExtractor {
    Character(CharacterNgrams),
    Word(WordNgrams),
    Custom(CustomExtractorInner),
}

impl FeatureExtractor for PyFeatureExtractor {
    fn persistence_config(&self) -> Option<ExtractorConfig> {
        match self {
            Self::Character(extractor) => extractor.persistence_config(),
            Self::Word(extractor) => extractor.persistence_config(),
            Self::Custom(_) => None,
        }
    }

    fn visit_features(&self, text: &str, visitor: &mut dyn FnMut(&str)) {
        match self {
            PyFeatureExtractor::Character(e) => e.visit_features(text, visitor),
            PyFeatureExtractor::Word(e) => e.visit_features(text, visitor),
            PyFeatureExtractor::Custom(e) => match e.collect_raw_features(text) {
                Ok(features) => crate::extractors::visit_counted_features(&features, visitor),
                Err(err) => {
                    Python::attach(|py| err.print(py));
                    panic!("Custom extractor apply() raised an exception");
                }
            },
        }
    }
}

/// Character *n*-gram feature extractor.
///
/// Pads the text with `n - 1` copies of `endmarker` on both sides and slides a
/// window of `n` characters over it. Repeated grams are qualified with a
/// 1-based occurrence counter (`aa`, `aa` → `aa1`, `aa2`), so every feature is
/// unique within one text. Operates on Unicode characters, not bytes.
///
/// Args:
///     n: Window length in characters. ``0`` produces no features.
///     endmarker: Marker repeated ``n - 1`` times around the text.
///
/// Examples:
///     >>> CharacterNgrams(2, '$').apply('hello')
///     ['$h1', 'he1', 'el1', 'll1', 'lo1', 'o$1']
#[pyclass(name = "CharacterNgrams")]
#[derive(Clone)]
struct PyCharacterNgrams(CharacterNgrams);

#[pymethods]
impl PyCharacterNgrams {
    /// See the class docstring for parameter details.
    #[new]
    fn new(n: usize, endmarker: &str) -> Self {
        Self(CharacterNgrams::new(n, endmarker))
    }

    /// Extracts the feature strings of ``text``.
    ///
    /// Args:
    ///     text: The text to tokenize.
    ///
    /// Returns:
    ///     The occurrence-qualified feature strings.
    fn apply(&self, text: &str) -> Vec<String> {
        let mut interner = lasso::Rodeo::default();
        let features = self.0.features(text, &mut interner);

        features
            .into_iter()
            .map(|spur| interner.resolve(&spur).to_string())
            .collect()
    }
}

/// Word *n*-gram feature extractor.
///
/// Splits ``text`` on ``splitter`` (dropping empty tokens), pads the token
/// sequence with ``padder`` at both ends, and joins every window of ``n``
/// consecutive tokens with a single space. Windows are occurrence-qualified
/// with a 1-based counter suffix. The join always uses a single space,
/// independent of ``splitter``.
///
/// Args:
///     n: Window length in tokens. ``0`` produces no features.
///     splitter: Separator string, e.g. ``" "``.
///     padder: Token added to both sequence ends, e.g. ``" "``.
///
/// Examples:
///     >>> WordNgrams(2, ' ', '_').apply('green apples')
///     ['_ green1', 'green apples1', 'apples _1']
#[pyclass(name = "WordNgrams")]
#[derive(Clone)]
struct PyWordNgrams(WordNgrams);

#[pymethods]
impl PyWordNgrams {
    /// See the class docstring for parameter details.
    #[new]
    fn new(n: usize, splitter: &str, padder: &str) -> Self {
        Self(WordNgrams::new(n, splitter, padder))
    }

    /// Extracts the feature strings of ``text``.
    ///
    /// Args:
    ///     text: The text to tokenize.
    ///
    /// Returns:
    ///     The occurrence-qualified feature strings.
    fn apply(&self, text: &str) -> Vec<String> {
        let mut interner = lasso::Rodeo::default();
        let features = self.0.features(text, &mut interner);

        features
            .into_iter()
            .map(|spur| interner.resolve(&spur).to_string())
            .collect()
    }
}

/// A feature extractor wrapping a Python-side ``apply`` callable.
///
/// Accepts any object providing ``apply(text: str) -> Iterable[str]``. The
/// wrapper counts occurrences, so raw features may repeat: the second
/// feature ``"aa"`` counts as ``"aa2"`` automatically.
///
/// Args:
///     extractor: Object with an ``apply`` method returning an iterable of
///         strings.
///
/// Raises:
///     TypeError: If ``extractor`` has no ``apply`` attribute.
///
/// Examples:
/// ```text
///     >>> from simstring_rust.extractors import CustomExtractor
///     >>> class LowerBigrams:
///     ...     def apply(self, text):
///     ...         tokens = text.lower().split()
///     ...         return [f"{a}|{b}" for a, b in zip(tokens, tokens[1:])]
///     >>> extractor = CustomExtractor(LowerBigrams())
///     >>> extractor.apply('Some Custom Text')
///     ['some|custom1', 'custom|text1']
///
/// ```
#[pyclass(name = "CustomExtractor")]
#[derive(Clone)]
struct PyCustomExtractor(CustomExtractorInner);

#[pymethods]
impl PyCustomExtractor {
    /// See the class docstring for parameter details.
    #[new]
    fn new(extractor: Py<PyAny>) -> PyResult<Self> {
        Python::attach(|py| {
            let bound = extractor.bind(py);
            if !bound.hasattr("apply")? {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "Custom extractor must provide an apply(text: str) -> Iterable[str] method",
                ))
            } else {
                Ok(Self(CustomExtractorInner::new(extractor)))
            }
        })
    }

    /// Extracts the occurrence-qualified feature strings of ``text``.
    ///
    /// Args:
    ///     text: The text to tokenize.
    ///
    /// Returns:
    ///     The qualified features, with any exceptions from the underlying
    ///     ``apply`` propagated.
    fn apply(&self, text: &str) -> PyResult<Vec<String>> {
        self.0.apply(text)
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

    fn similarity_from_counts(&self, x: usize, y: usize, common: usize) -> f64 {
        match self {
            PyMeasure::Cosine => Cosine.similarity_from_counts(x, y, common),
            PyMeasure::Dice => Dice.similarity_from_counts(x, y, common),
            PyMeasure::ExactMatch => ExactMatch.similarity_from_counts(x, y, common),
            PyMeasure::Jaccard => Jaccard.similarity_from_counts(x, y, common),
            PyMeasure::Overlap => Overlap.similarity_from_counts(x, y, common),
        }
    }
}

/// Cosine similarity measure: ``common / sqrt(x_len * y_len)``.
///
/// A balanced default that penalizes size differences moderately. An empty
/// side always scores ``0.0``.
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

/// Dice coefficient measure: ``2 * common / (x_len + y_len)``.
///
/// The default measure of the original SimString paper. Two empty feature
/// sets are defined as perfectly similar (``1.0``).
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

/// Equality measure: ``1.0`` for identical feature sets, ``0.0`` otherwise.
///
/// Use it to fall back from threshold-based fuzzy matching to a hard
/// membership probe on feature sets.
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

/// Jaccard similarity measure: ``common / (x_len + y_len - common)``.
///
/// The strictest set measure — intersection over union — good when only
/// strongly related strings should surface. Two empty sets are ``1.0``.
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

/// Overlap coefficient measure: ``common / min(x_len, y_len)``.
///
/// The most permissive measure: any string whose feature set fully contains
/// the query's scores ``1.0``, regardless of extra features — useful for
/// substring and containment style retrieval.
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

/// The in-memory database of strings plus their feature inverted index.
///
/// Strings are inserted one by one and tokenized by the extractor given at
/// construction. Duplicates are kept distinctly — inserting ``"apple"``
/// twice yields two database entries.
///
/// Args:
///     extractor: ``CharacterNgrams``, ``WordNgrams``, or ``CustomExtractor``
///         that defines how every inserted string (and later query) is
///         tokenized into features.
///
/// Raises:
///     TypeError: If ``extractor`` is not one of the supported extractors.
///
/// Examples:
///     >>> from simstring_rust.database import HashDb
///     >>> from simstring_rust.extractors import CharacterNgrams
///     >>> db = HashDb(CharacterNgrams(2, '$'))
///     >>> db.insert('apple')
///     >>> len(db)
///     1
#[pyclass(name = "HashDb")]
struct PyHashDb {
    db: HashDb,
}

impl From<PersistenceError> for PyErr {
    fn from(error: PersistenceError) -> Self {
        match error {
            PersistenceError::Io(error) => error.into(),
            PersistenceError::UnsupportedExtractor => {
                pyo3::exceptions::PyTypeError::new_err(error.to_string())
            }
            PersistenceError::Allocation(_) => {
                pyo3::exceptions::PyMemoryError::new_err(error.to_string())
            }
            _ => pyo3::exceptions::PyValueError::new_err(error.to_string()),
        }
    }
}

#[pymethods]
impl PyHashDb {
    /// See the class docstring for parameter details.
    #[new]
    fn new(extractor: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_feature_extractor =
            if let Ok(char_ngram) = extractor.extract::<PyRef<PyCharacterNgrams>>() {
                PyFeatureExtractor::Character(char_ngram.0.clone())
            } else if let Ok(word_ngram) = extractor.extract::<PyRef<PyWordNgrams>>() {
                PyFeatureExtractor::Word(word_ngram.0.clone())
            } else if let Ok(custom) = extractor.extract::<PyRef<PyCustomExtractor>>() {
                PyFeatureExtractor::Custom(custom.0.clone())
            } else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "Extractor must be CharacterNgrams, WordNgrams, or CustomExtractor",
                ));
            };

        let db = HashDb::new(Arc::new(py_feature_extractor));
        Ok(Self { db })
    }

    /// Indexes ``text``, assigning its own ID even if the string repeats.
    ///
    /// Args:
    ///     text: The string to insert.
    fn insert(&mut self, text: String) {
        self.db.insert(text);
    }

    /// Removes every string, releasing the built features and interner.
    /// Subsequent IDs restart from zero.
    fn clear(&mut self) {
        self.db.clear();
    }

    /// Saves the full index to an existing directory, atomically replacing
    /// ``file_name`` after a successful write.
    ///
    /// Args:
    ///     path: Existing directory that will contain the snapshot.
    ///     file_name: Single file name, not a path.
    ///
    /// Raises:
    ///     OSError: If the directory or file cannot be written.
    ///     ValueError: If the snapshot is invalid or ``file_name`` is not a
    ///         single file name.
    ///     TypeError: If the database uses an unsupported extractor.
    fn save_to_file(&self, py: Python<'_>, path: PathBuf, file_name: PathBuf) -> PyResult<()> {
        py.detach(|| self.db.save_to_file(path, file_name))
            .map_err(Into::into)
    }

    /// Restores a mutable database and its built-in extractor from a snapshot.
    ///
    /// Args:
    ///     path: Directory containing the snapshot.
    ///     file_name: Snapshot file name, not a path.
    ///
    /// Raises:
    ///     OSError: If the snapshot cannot be read.
    ///     ValueError: If the snapshot is invalid or ``file_name`` is not a
    ///         single file name.
    #[staticmethod]
    fn load_from_file(py: Python<'_>, path: PathBuf, file_name: PathBuf) -> PyResult<Self> {
        let db = py.detach(|| HashDb::load_from_file(path, file_name))?;
        Ok(Self { db })
    }

    /// Returns a copy of all indexed strings in insertion order (index ``i``
    /// equals string ID ``i``).
    fn strings(&mut self) -> Vec<String> {
        self.db.strings.clone()
    }

    /// Number of indexed strings.
    fn __len__(&self) -> usize {
        self.db.total_strings()
    }
}

/// Threshold search engine over a :class:`HashDb`.
///
/// Ranks every indexed string by the configured measure, pruning candidate
/// feature sizes so retrieval remains exact (no false results, none missing)
/// up to the floating point conventions of the measure.
///
/// Args:
///     db: The database to search in. Insert and clear operations can no
///         longer run while a search borrows the database (the GIL helps,
///         but PyO3 tracks the borrow statically).
///     measure: One of ``Cosine()``, ``Dice()``, ``Jaccard()``,
///         ``Overlap()``, or ``ExactMatch()``.
///
/// Raises:
///     TypeError: If ``measure`` is not one of the supported measures.
///
/// Examples:
///     >>> from simstring_rust.database import HashDb
///     >>> from simstring_rust.extractors import CharacterNgrams
///     >>> from simstring_rust.measures import Cosine
///     >>> from simstring_rust.searcher import Searcher
///     >>> db = HashDb(CharacterNgrams(2, '$'))
///     >>> db.insert('apple')
///     >>> searcher = Searcher(db, Cosine())
///     >>> searcher.search('appl', 0.7)
///     ['apple']
#[pyclass(name = "Searcher")]
struct PySearcher {
    db: Py<PyHashDb>,
    measure: PyMeasure,
}

#[pymethods]
impl PySearcher {
    /// See the class docstring for parameter details.
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

    /// Returns every string at least ``alpha``-similar to the query,
    /// ascending. For scores and best-first ordering use
    /// :meth:`ranked_search`.
    ///
    /// Args:
    ///     query_string: The query text.
    ///     alpha: Minimal similarity in ``(0.0, 1.0]``.
    ///
    /// Returns:
    ///     List of matching strings, sorted ascending.
    ///
    /// Raises:
    ///     SearchError: If ``alpha`` is outside ``(0.0, 1.0]``.
    fn search<'py>(
        &self,
        py: Python<'py>,
        query_string: &str,
        alpha: f64,
    ) -> PyResult<Vec<String>> {
        let db_borrow = self.db.borrow(py);
        let searcher = RustSearcher::new(&db_borrow.db, self.measure);
        let results = py
            .detach(|| searcher.search(query_string, alpha))
            .map_err(|e| match e {
                RustSearchError::InvalidThreshold(val) => {
                    SearchError::new_err(format!("Invalid threshold: {val}"))
                }
            })?;
        Ok(results.into_iter().map(|s| s.to_string()).collect())
    }

    /// Returns ``(string, score)`` pairs sorted from the most to the least
    /// similar, ties ascending by string.
    ///
    /// Args:
    ///     query_string: The query text.
    ///     alpha: Minimal similarity in ``(0.0, 1.0]``.
    ///
    /// Returns:
    ///     List of ``(text, score)`` pairs with ``score >= alpha``.
    ///
    /// Raises:
    ///     SearchError: If ``alpha`` is outside ``(0.0, 1.0]``.
    fn ranked_search<'py>(
        &self,
        py: Python<'py>,
        query_string: &str,
        alpha: f64,
    ) -> PyResult<Vec<(String, f64)>> {
        let db_borrow = self.db.borrow(py);
        let searcher = RustSearcher::new(&db_borrow.db, self.measure);
        let results = py
            .detach(|| searcher.ranked_search(query_string, alpha))
            .map_err(|e| match e {
                RustSearchError::InvalidThreshold(val) => {
                    SearchError::new_err(format!("Invalid threshold: {val}"))
                }
            })?;
        Ok(results
            .into_iter()
            .map(|(s, score)| (s.to_string(), score))
            .collect())
    }
}

#[pymodule]
fn simstring_rust(py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__doc__", MODULE_DOC)?;

    // Database submodule
    let database_module = PyModule::new(py, "database")?;
    database_module.add("__doc__", DATABASE_DOC)?;
    database_module.add_class::<PyHashDb>()?;
    m.add_submodule(&database_module)?;

    // Extractors submodule
    let extractors_module = PyModule::new(py, "extractors")?;
    extractors_module.add("__doc__", EXTRACTORS_DOC)?;
    extractors_module.add_class::<PyCharacterNgrams>()?;
    extractors_module.add_class::<PyWordNgrams>()?;
    extractors_module.add_class::<PyCustomExtractor>()?;
    m.add_submodule(&extractors_module)?;

    // Measures submodule
    let measures_module = PyModule::new(py, "measures")?;
    measures_module.add("__doc__", MEASURES_DOC)?;
    measures_module.add_class::<PyCosine>()?;
    measures_module.add_class::<PyDice>()?;
    measures_module.add_class::<PyJaccard>()?;
    measures_module.add_class::<PyOverlap>()?;
    measures_module.add_class::<PyExactMatch>()?;
    m.add_submodule(&measures_module)?;

    // Searcher submodule
    let searcher_module = PyModule::new(py, "searcher")?;
    searcher_module.add("__doc__", SEARCHER_DOC)?;
    searcher_module.add_class::<PySearcher>()?;
    m.add_submodule(&searcher_module)?;

    // errors submodule
    let errors_module = PyModule::new(py, "errors")?;
    errors_module.add("__doc__", ERRORS_DOC)?;
    errors_module.add("SearchError", py.get_type::<SearchError>())?;
    m.add_submodule(&errors_module)?;

    // Add modules to sys.modules to allow direct import
    let sys = PyModule::import(py, "sys")?;
    let modules = sys.getattr("modules")?.cast_into::<pyo3::types::PyDict>()?;
    modules.set_item("simstring_rust.database", database_module)?;
    modules.set_item("simstring_rust.extractors", extractors_module)?;
    modules.set_item("simstring_rust.measures", measures_module)?;
    modules.set_item("simstring_rust.searcher", searcher_module)?;
    modules.set_item("simstring_rust.errors", errors_module)?;

    Ok(())
}
