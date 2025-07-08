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

function bench_insert(results::Vector)
    companies = load_companies()

    for ngram_size in [2, 3, 4]
        b = @benchmarkable begin
            db = create_db($ngram_size)
            for company in $companies
                push!(db, company)
            end
        end samples=100 seconds=20

        result = run(b)

        mean_time = mean(result.times) / 1e6  # Convert ns to ms
        stddev_time = std(result.times) / 1e6      # Convert ns to ms

        push!(results, Dict(
            "language" => "julia",
            "backend" => "SimString.jl",
            "benchmark" => "insert",
            "parameters" => Dict("ngram_size" => ngram_size),
            "stats" => Dict(
                "mean" => mean_time,
                "stddev" => stddev_time,
                "iterations" => length(result.times)
            )
        ))
    end
end

function bench_search(results::Vector)
    companies = load_companies()
    search_terms = companies[1:100]

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
            stddev_time = std(result.times) / 1e6      # Convert ns to ms

            push!(results, Dict(
                "language" => "julia",
                "backend" => "SimString.jl",
                "benchmark" => "search",
                "parameters" => Dict("ngram_size" => ngram_size, "threshold" => threshold),
                "stats" => Dict(
                    "mean" => mean_time,
                    "stddev" => stddev_time,
                    "iterations" => length(result.times)
                )
            ))
        end
    end
end

function main()
    results = []
    bench_insert(results)
    bench_search(results)
    println(JSON.json(results, 2))
end

main()
