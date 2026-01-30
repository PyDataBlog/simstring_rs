use clap::{Parser, Subcommand, ValueEnum};
use simstring_rust::database::HashDb;
use simstring_rust::extractors::{CharacterNgrams, FeatureExtractor, WordNgrams};
use simstring_rust::measures::{Cosine, Dice, ExactMatch, Jaccard, Measure, Overlap};
use simstring_rust::Searcher;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a database from file sources
    Build {
        /// Database output path
        #[arg(short, long)]
        database: PathBuf,

        /// Input file path
        input: PathBuf,

        /// N-gram size
        #[arg(short, long, default_value_t = 3)]
        ngram: usize,

        /// Feature extractor type
        #[arg(long, value_enum, default_value_t = ExtractorType::Char)]
        extractor: ExtractorType,

        /// End marker for n-grams
        #[arg(long, default_value = "$")]
        marker: String,

        /// Input format
        #[arg(short, long, value_enum, default_value_t = InputFormat::Text)]
        format: InputFormat,

        /// CSV column index (only used with --format csv)
        #[arg(long, default_value_t = 0)]
        column: usize,

        /// Suppress progress output
        #[arg(short, long)]
        quiet: bool,
    },
    /// Search a database for similar strings
    Search {
        /// Database path (must be built previously)
        #[arg(short, long)]
        database: PathBuf,

        /// Input source used to build the DB
        #[arg(long)]
        source: PathBuf,

        /// Search queries (if empty, reads from stdin)
        queries: Vec<String>,

        /// Similarity measure
        #[arg(short, long, value_enum, default_value_t = MeasureType::Cosine)]
        similarity: MeasureType,

        /// Similarity threshold
        #[arg(short, long, default_value_t = 0.8)]
        threshold: f64,

        /// N-gram size
        #[arg(short, long, default_value_t = 3)]
        ngram: usize,

        /// Feature extractor type
        #[arg(long, value_enum, default_value_t = ExtractorType::Char)]
        extractor: ExtractorType,

        /// End marker
        #[arg(long, default_value = "$")]
        marker: String,

        /// Input format of source
        #[arg(short, long, value_enum, default_value_t = InputFormat::Text)]
        format: InputFormat,

        /// CSV column index of source
        #[arg(long, default_value_t = 0)]
        column: usize,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Text)]
        output: OutputFormat,

        /// Include similarity scores in output
        #[arg(long)]
        ranked: bool,

        /// Suppress headers and metadata
        #[arg(short, long)]
        quiet: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ExtractorType {
    Char,
    Word,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum MeasureType {
    Cosine,
    Dice,
    Jaccard,
    Overlap,
    Exact,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum InputFormat {
    Text,
    Json,
    Csv,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

fn build_db(
    input: &Path,
    ngram: usize,
    extractor_type: ExtractorType,
    marker: &str,
    format: InputFormat,
    column: usize,
    quiet: bool,
) -> Result<HashDb, Box<dyn std::error::Error>> {
    if !quiet {
        eprintln!("Building database from {:?}...", input);
    }
    let start = Instant::now();

    let extractor: Arc<dyn FeatureExtractor> = match extractor_type {
        ExtractorType::Char => Arc::new(CharacterNgrams::new(ngram, marker)),
        ExtractorType::Word => Arc::new(WordNgrams::new(ngram, " ", marker)),
    };

    let mut db = HashDb::new(extractor);

    let count = match format {
        InputFormat::Text => db.insert_from_text_file(input)?,
        InputFormat::Json => db.insert_from_json_file(input)?,
        InputFormat::Csv => db.insert_from_csv_file(input, column)?,
    };

    if !quiet {
        eprintln!("Indexed {} strings in {:.2?}", count, start.elapsed());
    }

    Ok(db)
}

fn perform_search_ref<M: Measure>(
    searcher: &Searcher<'_, M>,
    query: &str,
    threshold: f64,
    ranked: bool,
    output: OutputFormat,
    quiet: bool,
) {
    if ranked {
        match searcher.ranked_search(query, threshold) {
            Ok(results) => match output {
                OutputFormat::Text => {
                    if !quiet {
                        println!("Results for '{}':", query);
                    }
                    for (match_str, score) in results {
                        println!("{}\t{:.4}", match_str, score);
                    }
                }
                OutputFormat::Json => {
                    let json_results: Vec<_> = results
                        .into_iter()
                        .map(|(s, score)| {
                            serde_json::json!({
                                "match": s,
                                "score": score
                            })
                        })
                        .collect();
                    println!("{}", serde_json::to_string(&json_results).unwrap());
                }
            },
            Err(e) => eprintln!("Error searching for '{}': {}", query, e),
        }
    } else {
        match searcher.search(query, threshold) {
            Ok(results) => match output {
                OutputFormat::Text => {
                    if !quiet {
                        println!("Results for '{}':", query);
                    }
                    for match_str in results {
                        println!("{}", match_str);
                    }
                }
                OutputFormat::Json => {
                    println!("{}", serde_json::to_string(&results).unwrap());
                }
            },
            Err(e) => eprintln!("Error searching for '{}': {}", query, e),
        }
    }
}

fn run_search_logic<M: Measure>(
    db: &HashDb,
    measure: M,
    queries: &[String],
    threshold: f64,
    ranked: bool,
    output: OutputFormat,
    quiet: bool,
) {
    let searcher = Searcher::new(db, measure);
    if queries.is_empty() {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            if let Ok(query) = line {
                perform_search_ref(&searcher, &query, threshold, ranked, output, quiet);
            }
        }
    } else {
        for query in queries {
            perform_search_ref(&searcher, query, threshold, ranked, output, quiet);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            database: _,
            input,
            ngram,
            extractor,
            marker,
            format,
            column,
            quiet,
        } => {
            let _db = build_db(&input, ngram, extractor, &marker, format, column, quiet)?;
            if !quiet {
                eprintln!("Database built successfully (in-memory only for now).");
                eprintln!("To search, provide the source file again using --source");
            }
        }
        Commands::Search {
            database: _,
            source,
            queries,
            similarity,
            threshold,
            ngram,
            extractor,
            marker,
            format,
            column,
            output,
            ranked,
            quiet,
        } => {
            let db = build_db(&source, ngram, extractor, &marker, format, column, quiet)?;

            match similarity {
                MeasureType::Cosine => {
                    run_search_logic(&db, Cosine, &queries, threshold, ranked, output, quiet)
                }
                MeasureType::Dice => {
                    run_search_logic(&db, Dice, &queries, threshold, ranked, output, quiet)
                }
                MeasureType::Jaccard => {
                    run_search_logic(&db, Jaccard, &queries, threshold, ranked, output, quiet)
                }
                MeasureType::Overlap => {
                    run_search_logic(&db, Overlap, &queries, threshold, ranked, output, quiet)
                }
                MeasureType::Exact => {
                    run_search_logic(&db, ExactMatch, &queries, threshold, ranked, output, quiet)
                }
            }
        }
    }

    Ok(())
}
