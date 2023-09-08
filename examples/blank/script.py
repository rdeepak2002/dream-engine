from dream_py import *


class SampleScript(Entity):
    def bound_x(self, position: Vector3, min_x: float = -800.0, max_x: float = 800.0):
        return position if position.x < max_x else Vector3(min_x, position.y, position.z)

    def update(self, dt: float):
        self.position = self.bound_x(self.position + Vector3(200.0 * dt, 0.0, 0.0))
