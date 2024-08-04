import { getPropertyVec3, setPropertyVec3 } from "./deno.ts";
import type { Attachment, Vec3 } from "./core.ts";

export class Transform {
  private _position: Vec3;
  attachment: Attachment | undefined;

  static getComponentId() {
    return acro.COMPONENT_IDS["Transform"];
  }

  constructor(position: Vec3, attachment?: Attachment) {
    this._position = position;
    this.attachment = attachment;
  }

  get position() {
    if (this.attachment)
      this._position = getPropertyVec3(this.attachment.add("position"));
    return this._position;
  }

  set position(value) {
    if (this.attachment) {
      value.attachment = this.attachment.add("position");
      setPropertyVec3(value.attachment, value);
    }
    this._position = value;
  }
}
