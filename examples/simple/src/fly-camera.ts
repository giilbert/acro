import { Behavior, Entity, Input, Vec2, Vec3 } from "jsr:@acro/lib";

const MOVE_SPEED = 10;

class FlyCamera extends Behavior {
  private lastMousePosition = new Vec2(0, 0);

  constructor(entity: Entity) {
    super(entity);
  }

  update(deltaTime: number) {
    const currentMousePosition = Input.getMousePosition();
    const mouseDelta = currentMousePosition.sub(this.lastMousePosition);
    this.lastMousePosition = Input.getMousePosition();

    if (Input.isKeyPressed("KeyW")) {
      this.transform.position.addAssign(
        this.transform.forward.scale(MOVE_SPEED * deltaTime)
      );
    } else if (Input.isKeyPressed("KeyS")) {
      this.transform.position.addAssign(
        this.transform.forward.scale(-MOVE_SPEED * deltaTime)
      );
    }

    if (Input.isKeyPressed("KeyA")) {
      this.transform.position.addAssign(
        this.transform.right.scale(MOVE_SPEED * deltaTime)
      );
    } else if (Input.isKeyPressed("KeyD")) {
      this.transform.position.addAssign(
        this.transform.right.scale(-MOVE_SPEED * deltaTime)
      );
    }

    // if (mouseDelta.magnitude > 0) console.log(mouseDelta);

    if (Input.isMousePressed("Right")) {
      this.transform.rotation.x += mouseDelta.y * 0.002;
      this.transform.rotation.y += mouseDelta.x * 0.002;
    }

    // this.transform.scale = new Vec3(2, 2, 2);
  }
}

export const init = () => acro.registerBehavior("FlyCamera", FlyCamera);
