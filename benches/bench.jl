using BenchmarkTools
using SimString
using JSON

function load_companies()
    current_dir = pwd()
    file_path = joinpath(current_dir, "benches", "data", "company_names.txt")
    return readlines(file_path)
end

function create_db(ngram_size::Int)
    return SimString.DictDB(SimString.CharacterNGrams(ngram_size, " "))
end

function bench_insert()
    companies = load_companies()
    println("\nBenchmarking database insertions:")
    println(repeat("-", 40))

    results = []

    for ngram_size in [2, 3, 4]
        b = @benchmarkable begin
            db = create_db($ngram_size)
            for company in $companies
                push!(db, company)
            end
        end samples=100 seconds=20

        result = run(b)

        mean_time = mean(result.times) / 1e6  # Convert ns to ms
        stddev = std(result.times) / 1e6      # Convert ns to ms

        println("ngram_$ngram_size:")
        println("  Mean: $(round(mean_time, digits=2))ms")
        println("  Std Dev: $(round(stddev, digits=2))ms")
        println("  Iterations: $(length(result.times))")

        push!(results, Dict("ngram_size" => ngram_size, "mean" => mean_time, "stddev" => stddev, "iterations" => length(result.times)))
    end

    return results
end

function bench_search()
    companies = load_companies()
    search_terms = companies[1:100]  # Use first 100 companies as search terms

    println("\nBenchmarking database searches:")
    println(repeat("-", 40))

    results = []

    for ngram_size in [2, 3, 4]
        db = create_db(ngram_size)
        for company in companies
            push!(db, company)
        end

        for threshold in [0.6, 0.7, 0.8]
            b = @benchmarkable begin
                for term in $search_terms
                    SimString.search(SimString.Cosine(), $db, term; α=$threshold, ranked=true)
                end
            end samples=100 seconds=20

            result = run(b)

            mean_time = mean(result.times) / 1e6  # Convert ns to ms
            stddev = std(result.times) / 1e6      # Convert ns to ms

            println("ngram_$(ngram_size) (threshold=$(threshold)):")
            println("  Mean: $(round(mean_time, digits=2))ms")
            println("  Std Dev: $(round(stddev, digits=2))ms")
            println("  Iterations: $(length(result.times))")

            push!(results, Dict("ngram_size" => ngram_size, "threshold" => threshold, "mean" => mean_time, "stddev" => stddev, "iterations" => length(result.times)))
        end
    end

    return results
end

function main()
    insert_results = bench_insert()
    search_results = bench_search()

    json_output = Dict("insert_results" => insert_results, "search_results" => search_results)
    open("benches/benchmark_results.json", "w") do f
        JSON.print(f, json_output)
    end
end

main()
