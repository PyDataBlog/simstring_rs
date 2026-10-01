"""Process peak RSS after building a 100,000-row index (macOS/Linux)."""
import json
import resource
import sys
from itertools import cycle, islice
from pathlib import Path

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams

companies = Path("benches/data/company_names.txt").read_text().splitlines()
db = HashDb(CharacterNgrams(3, " "))
for company in islice(cycle(companies), 100_000):
    db.insert(company)
peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
print(json.dumps({"rows": len(db), "process_peak_rss_mib": peak / (1024**2 if sys.platform == "darwin" else 1024)}))
