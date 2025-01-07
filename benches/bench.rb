require 'benchmark'
require 'simstring_pure'
require 'descriptive_statistics'

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

    puts "ngram_#{ngram_size}:"
    puts "  Mean: #{(measurements.mean * 1000).round(2)}ms"
    puts "  Std Dev: #{(measurements.standard_deviation * 1000).round(2)}ms"
    puts "  Iterations: #{measurements.length}"
  end
end

def bench_search
  companies = load_companies()
  search_terms = companies[0...100]  # Use first 100 companies as search terms

  puts "\nBenchmarking database searches:"
  puts "-" * 40

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

      puts "ngram_#{ngram_size} (threshold=#{threshold}):"
      puts "  Mean: #{(measurements.mean * 1000).round(2)}ms"
      puts "  Std Dev: #{(measurements.standard_deviation * 1000).round(2)}ms"
      puts "  Iterations: #{measurements.length}"
    end
  end
end

def main
  bench_insert
  bench_search
end

main
