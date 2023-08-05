from dream import get_entity

class PythonScript:
    def __init__(self):
        self.x = -2.5

    def update(self, dt: float, handle: int):
        self.x += 2.0 * dt
        if self.x >= 2.5:
            self.x = -2.5
        entity = get_entity(handle)
        entity.set_position(self.x, -4.8, -6.0)
        return entity.get_transform().position.x


PythonScript()
