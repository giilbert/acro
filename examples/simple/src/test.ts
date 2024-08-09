import { Behavior, Entity, Input, Vec3 } from "jsr:@acro/lib";

class TestBehavior extends Behavior {
  constructor(entity: Entity) {
    super(entity);
  }

  update() {
    this.transform.position.addAssign({ x: 0.00001, y: 0.00001, z: 0 });
    if (Input.isKeyPressed("Space")) {
      this.transform.rotation.z -= 0.0001;
    }
    this.transform.scale = new Vec3(2, 2, 2);
  }
}

export const init = () => acro.registerBehavior("TestBehavior", TestBehavior);
