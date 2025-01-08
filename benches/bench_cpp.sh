#!/bin/bash

iterations=100
measurement_time=20
ngram_sizes=(2 3 4)
thresholds=(0.6 0.7 0.8)

head -n 100 benches/data/company_names.txt > benches/data/search_queries.txt

echo -e "\nBenchmarking database insertions:"
echo "----------------------------------------"

results=()

for n in "${ngram_sizes[@]}"
do
    echo "ngram_$n:"

    start_time="$(date +%s)"
    count=0
    mapfile -t times

    while [ "$(date +%s)" -lt "$((start_time + measurement_time))" ] && [ $count -lt $iterations ]
    do
        start_ns="$(date +%s%N)"
        simstring -b -n "$n" -m -q -d "company_db" < benches/data/company_names.txt >/dev/null 2>&1
        end_ns="$(date +%s%N)"
        elapsed_ms="$(( (end_ns - start_ns) / 1000000 ))"
        times+=("$elapsed_ms")
        ((count++))
    done

    sum=0
    for t in "${times[@]}"
    do
        sum=$((sum + t))
    done
    mean=$(echo "scale=2; $sum / ${#times[@]}" | bc)

    sum_squared_diff=0
    for t in "${times[@]}"
    do
        diff=$(echo "scale=2; $t - $mean" | bc)
        squared_diff=$(echo "scale=2; $diff * $diff" | bc)
        sum_squared_diff=$(echo "scale=2; $sum_squared_diff + $squared_diff" | bc)
    done
    stddev=$(echo "scale=2; sqrt($sum_squared_diff / ${#times[@]})" | bc)

    echo "  Mean: ${mean}ms"
    echo "  Std Dev: ${stddev}ms"
    echo "  Iterations: ${#times[@]}"

    results+=("{\"ngram_size\": $n, \"mean\": $mean, \"stddev\": $stddev, \"iterations\": ${#times[@]}}")
done

echo -e "\nBenchmarking database searches:"
echo "----------------------------------------"

for n in "${ngram_sizes[@]}"
do
    simstring -b -n "$n" -m -q -d "company_db" < benches/data/company_names.txt >/dev/null 2>&1

    for threshold in "${thresholds[@]}"
    do
        echo "ngram_${n} (threshold=${threshold}):"

        start_time="$(date +%s)"
        count=0
        mapfile -t times

        while [ "$(date +%s)" -lt "$((start_time + measurement_time))" ] && [ $count -lt $iterations ]
        do
            start_ns="$(date +%s%N)"

            while IFS= read -r query
            do
                simstring -d "company_db" -t "$threshold" -s cosine -q <<< "$query" >/dev/null 2>&1
            done < benches/data/search_queries.txt

            end_ns="$(date +%s%N)"
            elapsed_ms="$(( (end_ns - start_ns) / 1000000 ))"
            times+=("$elapsed_ms")
            ((count++))
        done

        sum=0
        for t in "${times[@]}"
        do
            sum=$((sum + t))
        done
        mean=$(echo "scale=2; $sum / ${#times[@]}" | bc)

        sum_squared_diff=0
        for t in "${times[@]}"
        do
            diff=$(echo "scale=2; $t - $mean" | bc)
            squared_diff=$(echo "scale=2; $diff * $diff" | bc)
            sum_squared_diff=$(echo "scale=2; $sum_squared_diff + $squared_diff" | bc)
        done
        stddev=$(echo "scale=2; sqrt($sum_squared_diff / ${#times[@]})" | bc)

        echo "  Mean: ${mean}ms"
        echo "  Std Dev: ${stddev}ms"
        echo "  Iterations: ${#times[@]}"

        results+=("{\"ngram_size\": $n, \"threshold\": $threshold, \"mean\": $mean, \"stddev\": $stddev, \"iterations\": ${#times[@]}}")
    done
done

json_output="{\"results\": [$(IFS=,; echo "${results[*]}")]}"

echo "$json_output" > benches/benchmark_results.json

rm -f company_db benches/data/search_queries.txt
