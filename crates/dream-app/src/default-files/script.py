from dream_py import vec3


# class Vector3:


class SampleScript:
    def __init__(self):
        self.x: float = -2.5

    def bound_x(self, x):
        return x if x < 2.5 else -2.5

    def update(self, dt: float, entity):
        cur_pos = entity.get_position()
        vel = vec3(2.0 * dt, 0.0, 0.0)
        new_pos = vec3(self.bound_x(cur_pos.x + vel.x), cur_pos.y + vel.y, cur_pos.z + vel.z)
        entity.set_position(new_pos)


SampleScript()
