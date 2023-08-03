from dream import get_entity


class PythonScript:
    def update(self, handle):
        # TODO: pass handle through method
        entity = get_entity(handle)
        return entity.get_transform()


PythonScript()
