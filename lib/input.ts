declare namespace Deno.core.ops {
  const op_get_key_press: (key: string) => boolean;
}

const isKeyPressed = (key: string) => {
  return Deno.core.ops.op_get_key_press(JSON.stringify(key));
};

export const Input = {
  isKeyPressed,
};
