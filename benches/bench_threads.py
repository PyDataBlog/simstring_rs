"""End-to-end Python throughput; includes ThreadPoolExecutor scheduling."""
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from statistics import median
from time import perf_counter

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher

companies = Path("benches/data/company_names.txt").read_text().splitlines()
db = HashDb(CharacterNgrams(2, " "))
for company in companies:
    db.insert(company)
searcher = Searcher(db, Cosine())
queries = companies[:100] * 10


def search(query):
    return len(searcher.ranked_search(query, 0.6))


for workers in [1, 4]:
    samples = []
    with ThreadPoolExecutor(max_workers=workers) as executor:
        for _ in range(6):
            start = perf_counter()
            count = sum(executor.map(search, queries))
            samples.append(perf_counter() - start)
    print(json.dumps({"workers": workers, "queries_per_second": len(queries) / median(samples[1:]), "matches": count}))
