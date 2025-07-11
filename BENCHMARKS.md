# Benchmark Results

This file is automatically generated by the CI. Do not edit manually.

### Insert Benchmark
| language   | backend        |   ngram_size |     mean |    stddev |   iterations |
|:-----------|:---------------|-------------:|---------:|----------:|-------------:|
| julia      | SimString.jl   |            2 |  79.3845 | 19.9742   |          100 |
| julia      | SimString.jl   |            3 |  93.982  | 22.2376   |          100 |
| julia      | SimString.jl   |            4 | 112.772  | 28.26     |          100 |
| python     | simstring-fast |            2 |  87.7246 |  2.35556  |          100 |
| python     | simstring-fast |            3 | 103.227  |  2.75056  |          100 |
| python     | simstring-fast |            4 | 112.478  |  2.68081  |          100 |
| python     | simstring-rs   |            2 |  42.5656 |  0.735686 |          100 |
| python     | simstring-rs   |            3 |  51.8238 |  2.10491  |          100 |
| python     | simstring-rs   |            4 |  52.3574 |  1.52761  |          100 |
| ruby       | simstring-pure |            2 | 651.213  | 10.5263   |           31 |
| ruby       | simstring-pure |            3 | 729.424  | 12.9084   |           28 |
| ruby       | simstring-pure |            4 | 828.914  | 14.4889   |           25 |
| rust       | simstring-rs   |            2 |  40.5419 |  0.903586 |          100 |
| rust       | simstring-rs   |            3 |  46.5237 |  1.18169  |          100 |
| rust       | simstring-rs   |            4 |  48.0733 |  1.28903  |          100 |

### Search Benchmark
| language   | backend                 |   ngram_size |   threshold |      mean |    stddev |   iterations |
|:-----------|:------------------------|-------------:|------------:|----------:|----------:|-------------:|
| julia      | SimString.jl            |            2 |         0.6 | 375.601   | 5.55718   |           54 |
| julia      | SimString.jl            |            2 |         0.7 | 239.473   | 4.08148   |           84 |
| julia      | SimString.jl            |            2 |         0.8 | 131.495   | 2.41617   |          100 |
| julia      | SimString.jl            |            3 |         0.6 | 305.976   | 4.04843   |           66 |
| julia      | SimString.jl            |            3 |         0.7 | 203.977   | 3.9715    |           99 |
| julia      | SimString.jl            |            3 |         0.8 | 119.971   | 3.2088    |          100 |
| julia      | SimString.jl            |            4 |         0.6 | 281.679   | 4.89223   |           71 |
| julia      | SimString.jl            |            4 |         0.7 | 191.052   | 2.95292   |          100 |
| julia      | SimString.jl            |            4 |         0.8 | 114.617   | 3.22892   |          100 |
| python     | simstring-fast          |            2 |         0.6 | 104.53    | 2.48284   |          100 |
| python     | simstring-fast          |            2 |         0.7 |  48.5845  | 1.16871   |          100 |
| python     | simstring-fast          |            2 |         0.8 |  20.4824  | 0.42542   |          100 |
| python     | simstring-fast          |            2 |         0.9 |   8.77843 | 0.0747753 |          100 |
| python     | simstring-fast          |            3 |         0.6 |  81.8486  | 3.44618   |          100 |
| python     | simstring-fast          |            3 |         0.7 |  35.7895  | 1.72518   |          100 |
| python     | simstring-fast          |            3 |         0.8 |  16.9977  | 0.358583  |          100 |
| python     | simstring-fast          |            3 |         0.9 |   8.92975 | 0.143651  |          100 |
| python     | simstring-fast          |            4 |         0.6 |  69.2029  | 3.93952   |          100 |
| python     | simstring-fast          |            4 |         0.7 |  33.3105  | 1.66116   |          100 |
| python     | simstring-fast          |            4 |         0.8 |  16.6559  | 0.230515  |          100 |
| python     | simstring-fast          |            4 |         0.9 |   9.29387 | 0.073124  |          100 |
| python     | simstring-rust-bindings |            2 |         0.6 |  23.6049  | 1.37676   |          100 |
| python     | simstring-rust-bindings |            2 |         0.7 |  15.3168  | 0.931011  |          100 |
| python     | simstring-rust-bindings |            2 |         0.8 |   9.45999 | 0.523887  |          100 |
| python     | simstring-rust-bindings |            2 |         0.9 |   5.75463 | 0.288532  |          100 |
| python     | simstring-rust-bindings |            3 |         0.6 |  23.0744  | 1.25156   |          100 |
| python     | simstring-rust-bindings |            3 |         0.7 |  15.6374  | 0.752135  |          100 |
| python     | simstring-rust-bindings |            3 |         0.8 |  10.3874  | 1.28049   |          100 |
| python     | simstring-rust-bindings |            3 |         0.9 |   6.28978 | 0.416802  |          100 |
| python     | simstring-rust-bindings |            4 |         0.6 |  23.542   | 1.16585   |          100 |
| python     | simstring-rust-bindings |            4 |         0.7 |  17.5095  | 1.09403   |          100 |
| python     | simstring-rust-bindings |            4 |         0.8 |  11.3854  | 0.756792  |          100 |
| python     | simstring-rust-bindings |            4 |         0.9 |   6.88685 | 0.44314   |          100 |
| ruby       | simstring-pure          |            2 |         0.6 | 800.862   | 7.10941   |           25 |
| ruby       | simstring-pure          |            2 |         0.7 | 393.125   | 2.2306    |           51 |
| ruby       | simstring-pure          |            2 |         0.8 | 181.466   | 2.89068   |          100 |
| ruby       | simstring-pure          |            3 |         0.6 | 632.478   | 6.36024   |           32 |
| ruby       | simstring-pure          |            3 |         0.7 | 311.741   | 3.65735   |           65 |
| ruby       | simstring-pure          |            3 |         0.8 | 161.107   | 1.69266   |          100 |
| ruby       | simstring-pure          |            4 |         0.6 | 570.396   | 6.277     |           36 |
| ruby       | simstring-pure          |            4 |         0.7 | 308.399   | 2.86676   |           65 |
| ruby       | simstring-pure          |            4 |         0.8 | 164.56    | 1.43181   |          100 |
| rust       | simstring-rs            |            2 |         0.6 |  25.8398  | 0.131642  |          100 |
| rust       | simstring-rs            |            2 |         0.7 |  16.0857  | 0.262579  |          100 |
| rust       | simstring-rs            |            2 |         0.8 |   9.17178 | 0.203497  |          100 |
| rust       | simstring-rs            |            2 |         0.9 |   4.86511 | 0.0702372 |          100 |
| rust       | simstring-rs            |            3 |         0.6 |  25.7933  | 0.538819  |          100 |
| rust       | simstring-rs            |            3 |         0.7 |  17.3065  | 0.374714  |          100 |
| rust       | simstring-rs            |            3 |         0.8 |  10.3732  | 0.36631   |          100 |
| rust       | simstring-rs            |            3 |         0.9 |   5.46783 | 0.089724  |          100 |
| rust       | simstring-rs            |            4 |         0.6 |  26.7889  | 0.66009   |          100 |
| rust       | simstring-rs            |            4 |         0.7 |  19.2918  | 0.272784  |          100 |
| rust       | simstring-rs            |            4 |         0.8 |  11.7465  | 0.259253  |          100 |
| rust       | simstring-rs            |            4 |         0.9 |   6.24004 | 0.10508   |          100 |

