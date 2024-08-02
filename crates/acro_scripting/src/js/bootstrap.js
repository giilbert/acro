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
    this._x = x;
    this._y = y;
    this._z = z;
    this.attachedTo = attachedTo;
  }

  get x() {
    if (this.attachedTo) {
      return Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".x"
      );
    } else {
      return this._x;
    }
  }

  get y() {
    if (this.attachedTo) {
      return Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".y"
      );
    } else {
      return this._y;
    }
  }

  get z() {
    if (this.attachedTo) {
      return Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".z"
      );
    } else {
      return this._z;
    }
  }

  set x(value) {
    if (this.attachedTo) {
      Deno.core.ops.op_set_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".x",
        value
      );
    } else {
      this._x = value;
    }
  }

  set y(value) {
    if (this.attachedTo) {
      Deno.core.ops.op_set_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".y",
        value
      );
    } else {
      this._y = value;
    }
  }

  set z(value) {
    if (this.attachedTo) {
      Deno.core.ops.op_set_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".z",
        value
      );
    } else {
      this._z = value;
    }
  }
}

class AcroGlobalHook {
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

  registerBehavior(name, behavior) {
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

  createBehavior(entityGeneration, entityIndex, id, name, ...args) {
    const behavior = new this.behaviorConstructors[name](
      new Entity(entityGeneration, entityIndex),
      ...args
    );
    this.behaviors.set(id, behavior);
  }
}

globalThis.console = {
  log: function (...args) {
    Deno.core.print(args.map(JSON.stringify).join(" ") + "\n");
  },
};

globalThis.acro = new AcroGlobalHook();
