import { getPropertyNumber, setPropertyNumber } from "deno";
import { Attachment } from "./core";

export class Vec3 {
  _x: number;
  _y: number;
  _z: number;
  attachment: Attachment | undefined;

  constructor(x: number, y: number, z: number, attachment?: Attachment) {
    this._x = x;
    this._y = y;
    this._z = z;
    this.attachment = attachment;
  }

  get x() {
    if (this.attachment) this._z = getPropertyNumber(this.attachment.add("x"));
    return this._x;
  }

  get y() {
    if (this.attachment) this._y = getPropertyNumber(this.attachment.add("y"));
    return this._y;
  }

  get z() {
    if (this.attachment) this._z = getPropertyNumber(this.attachment.add("z"));
    return this._z;
  }

  set x(value) {
    if (this.attachment) setPropertyNumber(this.attachment.add("x"), value);
    this._x = value;
  }

  set y(value) {
    if (this.attachment) setPropertyNumber(this.attachment.add("y"), value);
    this._y = value;
  }

  set z(value) {
    if (this.attachment) setPropertyNumber(this.attachment.add("z"), value);
    this._z = value;
  }
}
