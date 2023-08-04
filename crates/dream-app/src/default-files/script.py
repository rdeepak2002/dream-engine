from dream import get_entity


class PythonScript:
    def update(self, handle: int):
        # TODO: pass handle through method
        entity = get_entity(handle)
        # new_position = Vector3(2, 2, 2)
        entity.set_position(1.0, -4.8, -6.0)
        return entity.get_transform().position.x


PythonScript()
