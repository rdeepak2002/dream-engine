from dream import get_entity

class PythonScript:
    def update(self, dt: float, handle: int):
        # TODO: use dt to update positoin

        # TODO: store variable and do += dt
        # cuz dt is constant 0.16
        entity = get_entity(handle)
        entity.set_position(dt, -4.8, -6.0)
        return entity.get_transform().position.x


PythonScript()
