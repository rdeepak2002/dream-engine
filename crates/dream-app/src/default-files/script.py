from dream_py import *


class SampleScript:
    def bound_x(self, position: Vector3, min_x: float = -2.5, max_x: float = 2.5):
        return position if position.x < max_x else Vector3(min_x, position.y, position.z)

    def update(self, dt: float, entity_id: int):
        entity: Entity = Entity(entity_id)
        entity.position = self.bound_x(entity.position + Vector3(2.0 * dt, 0.0, 0.0))


SampleScript()
