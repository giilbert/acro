import { Vec2 } from "./mod.ts";

declare namespace Deno.core.ops {
  const op_get_key_press: (key: string) => boolean;
  const op_get_mouse_press: (button: string) => boolean;
  const op_get_mouse_position: () => [number, number];
}

const isKeyPressed = (key: string) => {
  return Deno.core.ops.op_get_key_press(JSON.stringify(key));
};

export type MouseButton = "Left" | "Right" | "Middle" | "Back" | "Forward";
const isMousePressed = (button: MouseButton) => {
  return Deno.core.ops.op_get_mouse_press(JSON.stringify(button));
};

const getMousePosition = () => {
  const [x, y] = Deno.core.ops.op_get_mouse_position();
  return new Vec2(x, y);
};

export const Input = {
  isKeyPressed,
  isMousePressed,
  getMousePosition,
};
