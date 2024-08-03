import { getPropertyVec3, setPropertyVec3 } from "deno";
import { Attachment } from "./core";
import { Vec3 } from "./vec3";

export class Transform {
  _position: Vec3;
  attachment: Attachment | undefined;

  static getComponentId() {
    return acro.COMPONENT_IDS.Transform;
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
    if (this.attachment)
      setPropertyVec3(this.attachment.add("position"), value);
    this._position = value;
  }
}
