require 'benchmark'
require 'simstring_pure'
require 'descriptive_statistics'

def create_db(ngram_size)
  SimString::Database.new(SimString::NGramBuilder.new(ngram_size))
end

def bench_insert
  filepath = File.join(Dir.pwd, "benches", "data", "company_names.txt")
  companies = File.readlines(filepath, chomp: true)

  puts "Found #{companies.size} companies"
  puts "\nBenchmark Results:"
  puts "-" * 40

  # Test for n-gram sizes 2, 3, and 4
  [2, 3, 4].each do |ngram_size|
    measurements = []
    start_time = Time.now
    iterations = 0

    while Time.now - start_time < 40 && iterations < 100
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

bench_insert
