import { AcroGlobalHook } from "./globals.ts";

declare global {
  export var acro: AcroGlobalHook;
}

export const init = () => {
  globalThis.acro = new AcroGlobalHook();
};
