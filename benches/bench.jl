using SimString
using BenchmarkTools
using Statistics

function create_db(ngram_size::Int)
    return DictDB(CharacterNGrams(ngram_size, " "))
end

function bench_insert()
    filepath = joinpath(pwd(), "benches", "data", "company_names.txt")
    companies = readlines(filepath)

    suite = BenchmarkGroup()

    # Test for n-gram sizes 2, 3, and 4
    for ngram_size in [2, 3, 4]
        suite["ngram_$ngram_size"] = @benchmarkable begin
            db = create_db($ngram_size)
            for company in $companies
                push!(db, company)
            end
        end samples=100 seconds=20
    end

    results = run(suite)

    println("\nBenchmark Results:")
    println(repeat("-", 40))
    for (name, result) in results
        println("$name:")
        println("  Mean: $(mean(result.times) / 1e6) ms")
        println("  Std Dev: $(std(result.times) / 1e6) ms")
        println("  Memory: $(result.memory) bytes")
        println("  Allocations: $(result.allocs)")
    end
end

if abspath(PROGRAM_FILE) == @__FILE__
    bench_insert()
end
