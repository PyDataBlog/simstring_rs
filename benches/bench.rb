require 'benchmark'
require 'simstring_pure'
require 'descriptive_statistics'
require 'json'

def create_db(ngram_size)
  SimString::Database.new(SimString::NGramBuilder.new(ngram_size))
end

def load_companies
  filepath = File.join(Dir.pwd, "benches", "data", "company_names.txt")
  File.readlines(filepath, chomp: true)
end

def bench_insert
  companies = load_companies()

  puts "\nBenchmarking database insertions:"
  puts "-" * 40

  results = []

  [2, 3, 4].each do |ngram_size|
    measurements = []
    start_time = Time.now
    iterations = 0

    while Time.now - start_time < 20 && iterations < 100
      time = Benchmark.realtime do
        db = create_db(ngram_size)
        companies.each do |company|
          db.add(company)
        end
      end
      measurements << time
      iterations += 1
    end

    mean_time = measurements.mean * 1000
    stddev = measurements.standard_deviation * 1000

    puts "ngram_#{ngram_size}:"
    puts "  Mean: #{mean_time.round(2)}ms"
    puts "  Std Dev: #{stddev.round(2)}ms"
    puts "  Iterations: #{measurements.length}"

    results << {
      ngram_size: ngram_size,
      mean: mean_time,
      stddev: stddev,
      iterations: measurements.length
    }
  end

  results
end

def bench_search
  companies = load_companies()
  search_terms = companies[0...100]  # Use first 100 companies as search terms

  puts "\nBenchmarking database searches:"
  puts "-" * 40

  results = []

  [2, 3, 4].each do |ngram_size|
    db = create_db(ngram_size)
    companies.each { |company| db.add(company) }
    matcher = SimString::StringMatcher.new(db, SimString::CosineMeasure.new)

    [0.6, 0.7, 0.8].each do |threshold|
      measurements = []
      start_time = Time.now
      iterations = 0

      while Time.now - start_time < 20 && iterations < 100
        time = Benchmark.realtime do
          search_terms.each do |term|
            matcher.ranked_search(term, threshold)
          end
        end
        measurements << time
        iterations += 1
      end

      mean_time = measurements.mean * 1000
      stddev = measurements.standard_deviation * 1000

      puts "ngram_#{ngram_size} (threshold=#{threshold}):"
      puts "  Mean: #{mean_time.round(2)}ms"
      puts "  Std Dev: #{stddev.round(2)}ms"
      puts "  Iterations: #{measurements.length}"

      results << {
        ngram_size: ngram_size,
        threshold: threshold,
        mean: mean_time,
        stddev: stddev,
        iterations: measurements.length
      }
    end
  end

  results
end

def main
  insert_results = bench_insert
  search_results = bench_search

  json_output = {
    insert_results: insert_results,
    search_results: search_results
  }

  File.open("benches/benchmark_results.json", "w") do |f|
    f.write(JSON.pretty_generate(json_output))
  end
end

main
