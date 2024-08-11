import { $, Behavior, Entity } from "jsr:@acro/core";
import { Vec3 } from "jsr:@acro/math";
import { Input } from "jsr:@acro/input";
import { Text } from "jsr:@acro/ui";

class TestBehavior extends Behavior {
  text: Text;

  constructor(entity: Entity) {
    super(entity);
    this.text = $("/UI/Panel/Text")?.getComponent(Text)!;
  }

  update(deltaTime: number) {
    if (Input.isMousePressed("Left")) {
      this.transform.rotation.z -= 5 * deltaTime;
      this.text.content = this.transform.rotation.z.toFixed(2);
      this.text.fontSize += 0.01;
    }
    this.transform.scale = new Vec3(2, 2, 2);
  }
}

export const init = () => acro.registerBehavior("TestBehavior", TestBehavior);
