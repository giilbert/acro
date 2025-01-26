import { AcroGlobalHook } from "./globals.ts";

declare global {
  // acro needs to be a var because it's a global variable
  // deno-lint-ignore no-var
  export var acro: AcroGlobalHook;
}

export const init = () => {
  globalThis.acro = new AcroGlobalHook();
};

export const registerComponents = (components: Record<string, number>) => {
  acro.registerComponents(components);
};

export const createBehavior = (
  generation: number,
  index: number,
  behaviorId: number,
  behaviorName: string
) => {
  acro.createBehavior(generation, index, behaviorId, behaviorName);
};

export const update = (deltaTime: number) => {
  acro.update(deltaTime);
};
