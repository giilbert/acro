// interface Deno {}

import { Attachment } from "./core.ts";
import { Vec3 } from "./vec3.ts";

declare namespace Deno.core.ops {
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

export function getPropertyVec3(attachment: Attachment): Vec3 {
  const value = Deno.core.ops.op_get_property_vec3(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  ) as Vec3;

  Object.setPrototypeOf(value, Vec3.prototype);
  value.attachment = attachment;

  return value;
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
