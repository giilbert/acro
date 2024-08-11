import {
  type Attachment,
  getPropertyVec3,
  setPropertyVec3,
} from "jsr:@acro/core";
import { Vec3 } from "jsr:@acro/math";

export class Transform {
  private _position: Vec3;
  private _rotation: Vec3;
  private _scale: Vec3;

  attachment: Attachment | undefined;

  static getComponentId() {
    return acro.COMPONENT_IDS["Transform"];
  }

  constructor(
    position: Vec3,
    rotation: Vec3,
    scale: Vec3,
    attachment?: Attachment
  ) {
    this._position = position;
    this._rotation = rotation;
    this._scale = scale;

    this.attachment = attachment;
  }

  get position() {
    if (this.attachment)
      this._position = getPropertyVec3(this.attachment.add("position"));
    return this._position;
  }

  set position(value) {
    if (this.attachment) {
      const attachment = this.attachment.add("position");
      setPropertyVec3(attachment, value);
      value.attachment = attachment;
    }
    this._position = value;
  }

  get rotation() {
    if (this.attachment)
      this._rotation = getPropertyVec3(this.attachment.add("rotation"));
    return this._rotation;
  }

  set rotation(value) {
    if (this.attachment) {
      const attachment = this.attachment.add("rotation");
      setPropertyVec3(attachment, value);
      value.attachment = attachment;
    }
    this._rotation = value;
  }

  get scale() {
    if (this.attachment)
      this._scale = getPropertyVec3(this.attachment.add("scale"));
    return this._scale;
  }

  set scale(value) {
    if (this.attachment) {
      const attachment = this.attachment.add("scale");
      setPropertyVec3(attachment, value);
      value.attachment = attachment;
    }
    this._scale = value;
  }

  get forward() {
    return new Vec3(
      Math.cos(-this.rotation.x) * Math.sin(-this.rotation.y),
      -Math.sin(-this.rotation.x),
      Math.cos(-this.rotation.x) * Math.cos(-this.rotation.y)
    ).normalized;
  }

  get right() {
    return new Vec3(Math.cos(-this.rotation.y), 0, -Math.sin(-this.rotation.y))
      .normalized;
  }

  get up() {
    return this.forward.cross(this.right);
  }
}
