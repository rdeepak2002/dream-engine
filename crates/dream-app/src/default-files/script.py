from dream_py import Entity, Vector3, fix_vec_3


class SampleScript:
    def __init__(self):
        self.x: float = -2.5

    def update(self, dt: float, entity: Entity):
        # self.x += 2.0 * dt
        # if self.x >= 2.5:
        #     self.x = -2.5
        # entity.set_position(self.x, -4.8, -6.0)

        # new_position = Vector3(1.0, 2.0, 3.0)
        # new_position = fix_vec_3(new_position)
        # entity.set_position(new_position.x, new_position.y, new_position.z)

        # new_position = entity.get_transform().position
        # new_position = new_position + Vector3(2.0 * dt, 0.0, 0.0)
        # entity.set_position(new_position.x, new_position.y, new_position.z)

        current_position: Vector3 = entity.get_transform().get_position()
        tmp = Vector3(2.0 * dt, 0.0, 0.0)
        tmp.x = 2.0 * dt
        tmp.y = 0.0
        tmp.z = 0.0
        velocity: Vector3 = fix_vec_3(tmp)

        x_new: float = current_position.get_x() + velocity.get_x()
        y_new: float = current_position.get_y() + velocity.get_y()
        z_new: float = current_position.get_z() + velocity.get_z()

        if x_new >= 2.5:
            x_new = -2.5

        tmp = Vector3(x_new, y_new, z_new)
        tmp.x = x_new
        tmp.y = y_new
        tmp.z = z_new
        new_position: Vector3 = fix_vec_3(tmp)
        entity.set_position(new_position.get_x(), new_position.get_y(), new_position.get_z())


SampleScript()
