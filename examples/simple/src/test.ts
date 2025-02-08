import { $, Behavior, Entity } from "jsr:@acro/core";
import { Input } from "jsr:@acro/input";
import { Button, Text } from "jsr:@acro/ui";

class TestBehavior extends Behavior {
  text: Text;
  button: Button;

  constructor(entity: Entity) {
    super(entity);

    this.text = $("/UI/Panel/Text")?.getComponent(Text)!;
    this.button = $("/UI/Panel 2")?.getComponent(Button)!;

    this.button.click.bind(() => {
      this.transform.position.y += 1;
    });
  }

  update(deltaTime: number) {
    if (Input.isMousePressed("Left"))
      this.transform.rotation.y -= 5 * deltaTime;
    if (Input.isMousePressed("Right"))
      this.transform.rotation.z += 5 * deltaTime;

    this.text.content = `y rotation (radians): ${this.transform.rotation.y.toFixed(
      2
    )}`;
  }
}

export const init = () => acro.registerBehavior("TestBehavior", TestBehavior);
