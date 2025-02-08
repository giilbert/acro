import { Vec2 } from "jsr:@acro/math";
import { createGlobalOp } from "jsr:@acro/core";

const isKeyPressedOp = createGlobalOp<[string], boolean>("op_get_key_press");
const isKeyPressed = (key: string) => {
  return isKeyPressedOp(JSON.stringify(key));
};

export type MouseButton = "Left" | "Right" | "Middle" | "Back" | "Forward";
const isMousePressedOp = createGlobalOp<[string], boolean>(
  "op_get_mouse_press"
);
const isMousePressed = (button: MouseButton) => {
  return isMousePressedOp(JSON.stringify(button));
};

const getMousePositionOp = createGlobalOp<[], [number, number]>(
  "op_get_mouse_position"
);
const getMousePosition = () => {
  const [x, y] = getMousePositionOp();
  return new Vec2(x, y);
};

export const Input = {
  isKeyPressed,
  isMousePressed,
  getMousePosition,
};
