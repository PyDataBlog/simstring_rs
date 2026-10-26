import json
from pathlib import Path
import sys
from time import perf_counter

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams, WordNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher


def measure(run, repeats=1):
    samples = []
    for iteration in range(9):
        start = perf_counter()
        value = run()
        elapsed = (perf_counter() - start) * 1000 / repeats
        del value
        if iteration >= 2:
            samples.append(elapsed)
    return samples


def main():
    corpus = Path(sys.argv[1]).read_text(encoding="utf-8").splitlines()
    queries = corpus[:100] + ["unknown 東京🦀", ""]
    selected = sys.argv[2:]
    results = []
    for kind in ["character", "word"]:
        for n in [2, 3, 4]:
            extractor = CharacterNgrams(n, " ") if kind == "character" else WordNgrams(n, " ", " ")

            def build():
                db = HashDb(extractor)
                for text in corpus:
                    db.insert(text)
                return db

            db = build()
            assert db.strings() == corpus
            case = f"{kind}/{n}/insert"
            if not selected or case in selected:
                results.append({"case": case, "samples_ms": measure(build), "check": len(corpus)})
            searcher = Searcher(db, Cosine())
            for alpha in [0.6, 0.9]:
                case = f"{kind}/{n}/search/{alpha}"
                if selected and case not in selected:
                    continue
                check = [searcher.search(query, alpha) for query in queries]

                def search():
                    for _ in range(50):
                        for query in queries:
                            searcher.search(query, alpha)

                results.append({"case": case, "samples_ms": measure(search, 50), "check": check})
            del searcher, db
    print(json.dumps(results))


if __name__ == "__main__":
    main()
