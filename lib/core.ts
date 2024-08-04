import { Transform } from "./transform.ts";
import { Vec3 } from "./vec3.ts";

export { Vec3 };

export class Entity {
  generation: number;
  index: number;

  constructor(generation: number, index: number) {
    this.generation = generation;
    this.index = index;
  }

  newAttachment(componentId: number, path: string) {
    return new Attachment(this, componentId, path);
  }
}

export class Attachment {
  entity: Entity;
  componentId: number;
  path: string;

  constructor(entity: Entity, componentId: number, path: string) {
    this.entity = entity;
    this.componentId = componentId;
    this.path = path;
  }

  add(pathSegment: string) {
    return new Attachment(
      this.entity,
      this.componentId,
      this.path + "." + pathSegment
    );
  }
}

interface ComponentConstructor<T> {
  // deno-lint-ignore no-explicit-any
  new (...args: any[]): T;
  getComponentId(): number;
}

export class Behavior {
  entity: Entity;
  transform: Transform;

  constructor(entity: Entity) {
    this.entity = entity;
    this.transform = this.getComponent(Transform);
  }

  getComponent<T>(ComponentClass: ComponentConstructor<T>): T {
    const attachment = this.entity.newAttachment(
      ComponentClass.getComponentId(),
      ""
    );

    if (ComponentClass === Transform) {
      return new Transform(
        new Vec3(0, 0, 0, attachment.add("position")),
        attachment
      ) as T;
    }

    throw new Error(`Unknown component class: ${ComponentClass}`);
  }

  update() {}
}
