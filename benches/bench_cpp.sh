#!/bin/bash

iterations=100

ngram_sizes=(2 3 4)

declare -A average_times

for n in "${ngram_sizes[@]}"; do
    echo "Benchmarking for n-gram size: $n"

    total_time=0

    for i in $(seq 1 $iterations); do
        echo "  Iteration $i:"

        start_time=$(date +%s%N)

        # Build the Simstring database with the current n-gram size and markers
        simstring -b -n "$n" -m -d "company_db" < benches/data/company_names.txt

        end_time=$(date +%s%N)

        elapsed_time=$(( (end_time - start_time) / 1000000 ))

        total_time=$((total_time + elapsed_time))

        echo "    Elapsed time: ${elapsed_time} ms"
    done

    average_time=$((total_time / iterations))

    average_times["$n"]=$average_time

    echo ""
done

echo "Summary of Average Execution Times:"
for n in "${!average_times[@]}"; do
    echo "  N-gram size $n: ${average_times[$n]} ms"
done
