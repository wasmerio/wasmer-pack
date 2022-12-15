from host_imports import bindings
from host_imports.bindings import host_imports

class Fs(host_imports.Fs):
    def __init__(self, files: dict[str, str]):
        self.files = files

    def read_file(self, filename: str) -> str:
        return self.files[filename]

class Logging(host_imports.Logging):
    def __init__(self):
        self.logged_messages = []

    def log(self, message: str):
        self.logged_messages.append(message)


def test_hello_world():
    files = { "some-file.txt": "Hello, World!" }
    fs = Fs(files)
    logging = Logging()

    wasm = bindings.host_imports(fs, logging)
    wasm.start()

    assert logging.logged_messages == [
        "Read 13 bytes from some-file.txt: Hello, World!"
    ]

