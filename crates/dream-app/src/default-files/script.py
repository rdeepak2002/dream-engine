from dream import get_entity


class PythonScript:
    def update(self):
        entity = get_entity(1)
        return entity.get_transform()


PythonScript()
