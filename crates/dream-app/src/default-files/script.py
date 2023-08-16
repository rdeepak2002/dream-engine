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


class SampleScript:
    def bound_x(self, position: Vector3, min_x: float = -2.5, max_x: float = 2.5):
        return position if position.x < max_x else Vector3(min_x, position.y, position.z)

    def update(self, dt: float, entity_id: int):
        entity: Entity = Entity(entity_id)
        entity.position = self.bound_x(entity.position + Vector3(2.0 * dt, 0.0, 0.0))


SampleScript()
