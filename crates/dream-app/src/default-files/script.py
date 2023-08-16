class SampleScript:
    def __init__(self):
        self.x: float = -2.5

    def update(self, dt: float, entity):
        self.x += 2.0 * dt
        if self.x >= 2.5:
            self.x = -2.5
        entity.set_position(self.x, -4.8, -6.0)
        # foo = Vector3(1.0, 2.0, 3.0)
        # foo = entity.get_transform().position
        # foo.x += 2.0 * dt
        # bar = Vector3(1.0, 3.0, 4.0)
        # foo = foo + bar
        # entity.set_position(foo.x, foo.y, foo.z)


SampleScript()
