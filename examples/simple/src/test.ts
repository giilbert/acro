import { Behavior, Entity } from "jsr:@acro/lib";

class TestBehavior extends Behavior {
  constructor(entity: Entity) {
    super(entity);
  }

  update() {
    this.transform.position.x += 0.0001;
  }
}

export const init = () => acro.registerBehavior("TestBehavior", TestBehavior);
