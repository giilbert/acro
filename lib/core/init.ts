import { AcroGlobalHook } from "./globals.ts";

declare global {
  // acro needs to be a var because it's a global variable
  // deno-lint-ignore no-var
  export var acro: AcroGlobalHook;
}

export const init = () => {
  globalThis.acro = new AcroGlobalHook();
};
