import { Entity } from "./core.ts";

interface ConstructableBehavior {
  new (entity: Entity, ...args: any[]): any;
}

export class AcroGlobalHook {
  COMPONENT_IDS: Record<string, number>;
  behaviorConstructors: Record<string, ConstructableBehavior>;
  behaviors: Map<number, any>;

  constructor() {
    // maps component names to ids
    this.COMPONENT_IDS = {};

    this.behaviorConstructors = {};
    this.behaviors = new Map();
  }

  update() {
    for (const behavior of this.behaviors.values()) {
      behavior.update();
    }
  }

  registerBehavior(name: string, behavior: ConstructableBehavior) {
    const shouldReloadBehaviors = !!this.behaviorConstructors[name];

    this.behaviorConstructors[name] = behavior;

    if (shouldReloadBehaviors)
      // The behavior is already registered, recreate behaviors that use it
      for (const [id, behavior] of this.behaviors) {
        this.createBehavior(
          behavior.entity.generation,
          behavior.entity.index,
          id,
          name
        );
      }
  }

  createBehavior(
    entityGeneration: number,
    entityIndex: number,
    id: number,
    name: string,
    ...args: any[]
  ) {
    const behavior = new this.behaviorConstructors[name](
      new Entity(entityGeneration, entityIndex),
      ...args
    );
    this.behaviors.set(id, behavior);
  }
}
