import { getPropertyNumber, setPropertyNumber } from "./deno.ts";
import type { Attachment } from "./core.ts";

interface Vec3Like {
  x: number;
  y: number;
  z: number;
}

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

  public add(rhs: Vec3Like): Vec3 {
    return new Vec3(this.x + rhs.x, this.y + rhs.y, this.z + rhs.z);
  }

  public addAssign(rhs: Vec3Like) {
    this.x += rhs.x;
    this.y += rhs.y;
    this.z += rhs.z;
  }

  public scale(scalar: number): Vec3 {
    return new Vec3(this.x * scalar, this.y * scalar, this.z * scalar);
  }

  public dot(rhs: Vec3Like): number {
    return this.x * rhs.x + this.y * rhs.y + this.z * rhs.z;
  }

  public cross(rhs: Vec3Like): Vec3 {
    return new Vec3(
      this.y * rhs.z - this.z * rhs.y,
      this.z * rhs.x - this.x * rhs.z,
      this.x * rhs.y - this.y * rhs.x
    );
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
