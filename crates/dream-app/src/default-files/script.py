from dream_py import *


class Vector3:
    def __init__(self, x: float = 0.0, y: float = 0.0, z: float = 0.0):
        self.x = x
        self.y = y
        self.z = z

    def __add__(self, other):
        x = self.x + other.x
        y = self.y + other.y
        z = self.z + other.z
        return Vector3(x, y, z)


class SampleScript:
    def bound_x(self, x):
        return x if x < 2.5 else -2.5

    def update(self, dt: float, entity):
        cur_pos = entity.get_position()
        vel = Vector3(2.0 * dt, 0.0, 0.0)
        new_pos = Vector3(self.bound_x(cur_pos.x + vel.x), cur_pos.y + vel.y, cur_pos.z + vel.z)
        entity.set_position(new_pos)


SampleScript()
