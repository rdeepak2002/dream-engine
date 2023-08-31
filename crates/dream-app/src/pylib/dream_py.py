from dream import dream_entity


class Entity:
    def __init__(self, handle: int):
        self.handle = handle
        self.internal = dream_entity(self.handle)

    @property
    def position(self):
        return Vector3(obj=self.internal.get_position())

    @position.setter
    def position(self, value):
        self.internal.set_position(value)

    def update(self, dt: float):
        pass


class Vector3:
    def __init__(self, x: float = 0.0, y: float = 0.0, z: float = 0.0, obj=None):
        self.x = x
        self.y = y
        self.z = z

        if obj is not None:
            self.x = obj.x
            self.y = obj.y
            self.z = obj.z

    def __add__(self, other):
        x = self.x + other.x
        y = self.y + other.y
        z = self.z + other.z
        return Vector3(x, y, z)
