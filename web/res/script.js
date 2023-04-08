class PlayerController {
    constructor() {

    }

    update(handle, dt) {
        let entity = new Entity(handle);
        let position = entity.getPosition();
        entity.setPosition(new Vector3(position.x + 0.1 * dt, position.y, position.z));
    }
}

new PlayerController()