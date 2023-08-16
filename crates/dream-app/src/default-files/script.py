from dream_py import *


class SampleScript:
    def __init__(self):
        self.x: float = -2.5

    def update(self, dt: float, entity):
        self.x += 2.0 * dt
        if self.x >= 2.5:
            self.x = -2.5
        entity.set_position(self.x, -4.8, -6.0)


SampleScript()
