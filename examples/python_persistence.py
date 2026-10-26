from pathlib import Path
from tempfile import TemporaryDirectory

from simstring_rust.database import HashDb
from simstring_rust.extractors import CharacterNgrams
from simstring_rust.measures import Cosine
from simstring_rust.searcher import Searcher


def main():
    with TemporaryDirectory() as directory:
        path = Path(directory)
        db = HashDb(CharacterNgrams(2, "$"))
        for text in ["hello", "help", "world"]:
            db.insert(text)
        db.save_to_file(path, "names.ssdb")
        restored = HashDb.load_from_file(path, "names.ssdb")
        restored.insert("halo")
        for text, score in Searcher(restored, Cosine()).ranked_search("hello", 0.5):
            print(f"{text}: {score:.4f}")


if __name__ == "__main__":
    main()
