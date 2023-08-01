from dream import rust_function


class PythonEntity:
    def __init__(self, handle):
        self.handle = handle

    def get_handle(self):
        rust_object = rust_function(42, "This is a python string", self)
        return self.handle


PythonEntity(6)
