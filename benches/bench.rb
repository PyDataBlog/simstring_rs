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

def bench_insert(results)
  companies = load_companies()

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

    results << {
      language: "ruby",
      backend: "simstring-pure",
      benchmark: "insert",
      parameters: { ngram_size: ngram_size },
      stats: {
        mean: (measurements.mean * 1000),
        stddev: (measurements.standard_deviation * 1000),
        iterations: measurements.length
      }
    }
  end
end

def bench_search(results)
  companies = load_companies()
  search_terms = companies[0...100]

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
            matcher.search(term, threshold)
          end
        end
        measurements << time
        iterations += 1
      end

      results << {
        language: "ruby",
        backend: "simstring-pure",
        benchmark: "search",
        parameters: { ngram_size: ngram_size, threshold: threshold },
        stats: {
          mean: (measurements.mean * 1000),
          stddev: (measurements.standard_deviation * 1000),
          iterations: measurements.length
        }
      }
    end
  end
end

def main
  results = []
  bench_insert(results)
  bench_search(results)
  puts JSON.pretty_generate(results)
end

main
