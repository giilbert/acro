// interface Deno {}

import type { Attachment } from "./mod.ts";
import { Vec3 } from "jsr:@acro/math";

declare global {
  interface ImportMeta {
    platform: "deno" | "web";
  }
}

declare namespace Deno.core.ops {
  const op_get_property_string: (
    generation: number,
    index: number,
    componentId: number,
    path: string
  ) => string;
  const op_set_property_string: (
    generation: number,
    index: number,
    componentId: number,
    path: string,
    value: string
  ) => void;

  const op_get_property_number: (
    generation: number,
    index: number,
    componentId: number,
    path: string
  ) => number;
  const op_set_property_number: (
    generation: number,
    index: number,
    componentId: number,
    path: string,
    value: number
  ) => void;

  const op_get_property_boolean: (
    generation: number,
    index: number,
    componentId: number,
    path: string
  ) => boolean;
  const op_set_property_boolean: (
    generation: number,
    index: number,
    componentId: number,
    path: string,
    value: boolean
  ) => void;

  const op_get_property_vec3: (
    generation: number,
    index: number,
    componentId: number,
    path: string
  ) => { x: number; y: number; z: number };
  const op_set_property_vec3: (
    generation: number,
    index: number,
    componentId: number,
    path: string,
    x: number,
    y: number,
    z: number
  ) => void;

  const op_call_function: (
    generation: number,
    index: number,
    componentId: number,
    path: string,
    args: any[]
  ) => any;
}

export function getPropertyString(attachment: Attachment): string {
  return Deno.core.ops.op_get_property_string(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  );
}

export function setPropertyString(attachment: Attachment, value: string) {
  Deno.core.ops.op_set_property_string(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value
  );
}

export function getPropertyNumber(attachment: Attachment): number {
  return Deno.core.ops.op_get_property_number(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  );
}

export function setPropertyNumber(attachment: Attachment, value: number) {
  Deno.core.ops.op_set_property_number(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value
  );
}

export function getPropertyBoolean(attachment: Attachment): boolean {
  return Deno.core.ops.op_get_property_boolean(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  );
}

export function setPropertyBoolean(attachment: Attachment, value: boolean) {
  Deno.core.ops.op_set_property_boolean(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value
  );
}

export function getPropertyVec3(attachment: Attachment): Vec3 {
  if (import.meta.platform === "web") {
    throw new Error("getPropertyVec3 not implemented in browser!");
  } else {
    const value = Deno.core.ops.op_get_property_vec3(
      attachment.entity.generation,
      attachment.entity.index,
      attachment.componentId,
      attachment.path
    ) as { x: number; y: number; z: number };

    return new Vec3(value.x, value.y, value.z, attachment);
  }
}

export function setPropertyVec3(attachment: Attachment, value: Vec3) {
  Deno.core.ops.op_set_property_vec3(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value.x,
    value.y,
    value.z
  );
}

export function callFunction(attachment: Attachment, args: any[]): any {
  Deno.core.ops.op_call_function(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    args
  );
}
