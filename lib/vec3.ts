import { getPropertyNumber, setPropertyNumber } from "./deno.ts";
import type { Attachment } from "./core.ts";

export class Vec3 {
  private _x: number;
  private _y: number;
  private _z: number;
  attachment: Attachment | undefined;

  constructor(x: number, y: number, z: number, attachment?: Attachment) {
    this._x = x;
    this._y = y;
    this._z = z;
    this.attachment = attachment;
  }

  set x(value: number) {
    if (this.attachment) setPropertyNumber(this.attachment.add("x"), value);
    this._x = value;
  }

  get x() {
    if (this.attachment) this._x = getPropertyNumber(this.attachment.add("x"));
    return this._x;
  }

  set y(value: number) {
    if (this.attachment) setPropertyNumber(this.attachment.add("y"), value);
    this._y = value;
  }

  get y() {
    console.log("get");
    if (this.attachment) this._y = getPropertyNumber(this.attachment.add("y"));
    return this._y;
  }

  set z(value: number) {
    if (this.attachment) setPropertyNumber(this.attachment.add("z"), value);
    this._z = value;
  }

  get z() {
    if (this.attachment) this._z = getPropertyNumber(this.attachment.add("z"));
    return this._z;
  }

  get hello() {
    return "Hello";
  }
}
