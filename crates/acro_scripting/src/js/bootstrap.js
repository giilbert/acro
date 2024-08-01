// TODO: refactor this!!!

class Entity {
  constructor(generation, index) {
    this.generation = generation;
    this.index = index;
  }
}

class Behavior {
  constructor(entity) {
    this.entity = entity;
    this.transform = this.getComponent("Transform");
  }

  getComponent(name) {
    if (name === "Transform") {
      return new Transform(
        new Vec3(0, 0, 0, {
          entity: this.entity,
          componentId: acro.COMPONENT_IDS.Transform,
          path: "position",
        })
      );
    }
  }
}

class Transform {
  // TODO: rotation and scale
  constructor(position) {
    this.position = position;
  }
}

class Vec3 {
  constructor(x, y, z, attachedTo) {
    this.x = x;
    this.y = y;
    this.z = z;
    this.attachedTo = attachedTo;
  }
}

class AcroGlobalHook {
  constructor() {
    this.behaviorConstructors = {};
    this.behaviors = {};
    // maps component names to ids
    this.COMPONENT_IDS = {};
  }

  update() {
    for (const behavior of Object.values(this.behaviors)) {
      behavior.update();
    }
  }

  registerBehavior(name, behavior) {
    this.behaviorConstructors[name] = behavior;
  }

  createBehavior(entityGeneration, entityIndex, id, name, ...args) {
    const behavior = new this.behaviorConstructors[name](
      new Entity(entityGeneration, entityIndex),
      ...args
    );
    this.behaviors[behavior.id] = behavior;
  }
}

globalThis.console = {
  log: function (...args) {
    Deno.core.print(args.map(JSON.stringify).join(" ") + "\n");
  },
};

globalThis.acro = new AcroGlobalHook();
