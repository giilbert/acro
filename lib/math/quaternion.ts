import { type Vec3Like, Vec3 } from "./vec3.ts";

// TODO: figure out how to use quaternions and make fly camera

export class Quaternion {
  constructor(
    public x: number,
    public y: number,
    public z: number,
    public w: number
  ) {}

  public static identity(): Quaternion {
    return new Quaternion(0, 0, 0, 1);
  }

  public static fromAxisAngle(axis: Vec3, angle: number): Quaternion {
    const s = Math.sin(angle / 2);
    return new Quaternion(
      axis.x * s,
      axis.y * s,
      axis.z * s,
      Math.cos(angle / 2)
    );
  }

  public static fromEulerAngles(angles: Vec3Like): Quaternion {
    const c1 = Math.cos(angles.y / 2);
    const s1 = Math.sin(angles.y / 2);
    const c2 = Math.cos(angles.x / 2);
    const s2 = Math.sin(angles.x / 2);
    const c3 = Math.cos(angles.z / 2);
    const s3 = Math.sin(angles.z / 2);

    return new Quaternion(
      c1 * s2 * c3 + s1 * c2 * s3,
      s1 * c2 * c3 - c1 * s2 * s3,
      c1 * c2 * s3 - s1 * s2 * c3,
      c1 * c2 * c3 + s1 * s2 * s3
    );
  }

  // FIXME: this might be causing the camera to rotate incorrectly
  public toEulerAngles(): Vec3 {
    const x = this.x;
    const y = this.y;
    const z = this.z;
    const w = this.w;

    const y2 = y ** 2;
    const z2 = z ** 2;
    const w2 = w ** 2;

    return new Vec3(
      Math.atan2(2 * x * w + 2 * y * z, 1 - 2 * (z2 + w2)),
      Math.asin(2 * (x * z - w * y)),
      Math.atan2(2 * x * y + 2 * z * w, 1 - 2 * (y2 + z2))
    );
  }

  public mul(rhs: Quaternion): Quaternion {
    const x = this.x;
    const y = this.y;
    const z = this.z;
    const w = this.w;

    const rx = rhs.x;
    const ry = rhs.y;
    const rz = rhs.z;
    const rw = rhs.w;

    return new Quaternion(
      w * rx + x * rw + y * rz - z * ry,
      w * ry + y * rw + z * rx - x * rz,
      w * rz + z * rw + x * ry - y * rx,
      w * rw - x * rx - y * ry - z * rz
    );
  }
}
