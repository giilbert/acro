import { Behavior, Entity, Input, Vec3 } from "jsr:@acro/lib";

class TestBehavior extends Behavior {
  constructor(entity: Entity) {
    super(entity);
  }

  update(deltaTime: number) {
    // if (Input.isMousePressed("Left")) {
    //   this.transform.rotation.z -= 5 * deltaTime;
    // }
    this.transform.scale = new Vec3(2, 2, 2);
  }
}

export const init = () => acro.registerBehavior("TestBehavior", TestBehavior);
