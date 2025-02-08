// interface Deno {}

import type { Attachment } from "./mod.ts";
import { Vec3 } from "jsr:@acro/math";

declare global {
  interface ImportMeta {
    platform: "deno" | "web";
  }

  // deno-lint-ignore no-var
  var wasm: Record<string, (...args: unknown[]) => unknown>;
}

declare namespace Deno.core {
  export const ops: Record<string, (...args: unknown[]) => unknown>;
}

export function createAttachedOp<
  TParameters extends unknown[],
  TOpReturn extends unknown,
  TReturn extends unknown = TOpReturn
>(
  name: string,
  transform: (opReturn: TOpReturn, attachment: Attachment) => TReturn = (
    opReturn,
    _
  ) => opReturn as unknown as TReturn
): (attachment: Attachment, ...args: TParameters) => TReturn {
  type OpsRecord = Record<string, (...args: unknown[]) => unknown>;
  const opFn =
    import.meta.platform === "web"
      ? (globalThis.wasm as OpsRecord)[name]
      : (Deno.core.ops as OpsRecord)[name];
  if (!opFn) throw new Error(`opFn ${name} not found!`);

  return (attachment, ...parameters) => {
    return transform(
      opFn(
        attachment.entity.generation,
        attachment.entity.index,
        attachment.componentId,
        attachment.path,
        ...parameters
      ) as TOpReturn,
      attachment
    );
  };
}

export function createGlobalOp<
  TParameters extends unknown[],
  TOpReturn extends unknown,
  TReturn extends unknown = TOpReturn
>(
  name: string,
  transform: (opReturn: TOpReturn) => TReturn = (opReturn) =>
    opReturn as unknown as TReturn
): (...args: TParameters) => TReturn {
  type OpsRecord = Record<string, (...args: unknown[]) => unknown>;
  const opFn =
    import.meta.platform === "web"
      ? (globalThis.wasm as OpsRecord)[name]
      : (Deno.core.ops as OpsRecord)[name];
  if (!opFn) throw new Error(`opFn ${name} not found!`);

  return (...parameters) => {
    return transform(opFn(...parameters) as TOpReturn);
  };
}

export const getPropertyString = createAttachedOp<[], string>(
  "op_get_property_string"
);

export const setPropertyString = createAttachedOp<[string], void>(
  "op_set_property_string"
);

export const getPropertyNumber = createAttachedOp<[], number>(
  "op_get_property_number"
);

export const setPropertyNumber = createAttachedOp<[number], void>(
  "op_set_property_number"
);

export const getPropertyBoolean = createAttachedOp<[], boolean>(
  "op_get_property_boolean"
);

export const setPropertyBoolean = createAttachedOp<[boolean], void>(
  "op_set_property_boolean"
);

export const getPropertyVec3 = createAttachedOp<
  [],
  { x: number; y: number; z: number },
  Vec3
>(
  "op_get_property_vec3",
  (ret, attachment) => new Vec3(ret.x, ret.y, ret.z, attachment)
);

export const setPropertyVec3 = createAttachedOp<[Vec3], void>(
  "op_set_property_vec3"
);

export const callFunction = createAttachedOp<[unknown[]], unknown>(
  "op_call_function"
);
