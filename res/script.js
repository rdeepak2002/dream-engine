class PlayerController {
    constructor() {

    }

    update(handle, dt) {
        let entity = new Entity(handle);
        let position = entity.getPosition();
        if (position.x < 3.0) {
            entity.setPosition(new Vector3(position.x + 1.0 * dt, position.y, position.z));
        }
    }
}

new PlayerController()