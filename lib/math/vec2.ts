import {
  type Attachment,
  getPropertyNumber,
  setPropertyNumber,
} from "jsr:@acro/core";

export interface Vec2Like {
  x: number;
  y: number;
}

export class Vec2 {
  private _x: number;
  private _y: number;
  attachment: Attachment | undefined;

  constructor(x: number, y: number, attachment?: Attachment) {
    this._x = x;
    this._y = y;
    this.attachment = attachment;
  }

  public add(rhs: Vec2Like): Vec2 {
    return new Vec2(this.x + rhs.x, this.y + rhs.y);
  }

  public addAssign(rhs: Vec2Like) {
    this.x += rhs.x;
    this.y += rhs.y;
  }

  public sub(rhs: Vec2Like): Vec2 {
    return new Vec2(this.x - rhs.x, this.y - rhs.y);
  }

  public subAssign(rhs: Vec2Like) {
    this.x -= rhs.x;
    this.y -= rhs.y;
  }

  public scale(scalar: number): Vec2 {
    return new Vec2(this.x * scalar, this.y * scalar);
  }

  public dot(rhs: Vec2Like): number {
    return this.x * rhs.x + this.y * rhs.y;
  }

  get magnitude(): number {
    return Math.sqrt(this.x * this.x + this.y * this.y);
  }

  get normalized(): Vec2 {
    return this.scale(1 / this.magnitude);
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
}
