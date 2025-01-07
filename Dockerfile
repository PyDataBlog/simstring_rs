FROM ubuntu:24.04

WORKDIR /app

RUN apt-get update && \
  apt-get install -y software-properties-common && \
  add-apt-repository universe && \
  apt-get update && \
  apt-get install -y build-essential simstring-bin time && \
  rm -rf /var/lib/apt/lists/*

COPY benches/data/company_names.txt /app/data/company_names.txt

COPY benches/bench_cpp.sh /app/benchmark.sh

RUN chmod +x /app/benchmark.sh

ENTRYPOINT ["/app/benchmark.sh"]
