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
        }),
        {
          entity: this.entity,
          componentId: acro.COMPONENT_IDS.Transform,
          path: "",
        }
      );
    }
  }
}

class Transform {
  // TODO: rotation and scale
  constructor(position, attachedTo) {
    this._position = position;
    this.attachedTo = attachedTo;
  }

  get position() {
    if (this.attachedTo) {
      const value = Deno.core.ops.op_get_property_vec3(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path
      );

      Object.setPrototypeOf(value, Vec3.prototype);
      value.attachedTo = this._position.attachedTo;

      this._position = value;
    }
    return this._position;
  }

  set position(value) {
    if (this.attachedTo) {
      Deno.core.ops.op_set_property_vec3(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + "position",
        value.x,
        value.y,
        value.z
      );
    }
    this._position = value;
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
      this._x = Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".x"
      );
    }
    return this._x;
  }

  get y() {
    if (this.attachedTo) {
      this._y = Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".y"
      );
    }
    return this._y;
  }

  get z() {
    if (this.attachedTo) {
      this._z = Deno.core.ops.op_get_property_number(
        this.attachedTo.entity.generation,
        this.attachedTo.entity.index,
        this.attachedTo.componentId,
        this.attachedTo.path + ".z"
      );
    }
    return this._z;
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
    }
    this._x = value;
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
    }
    this._y = value;
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
    }
    this._z = value;
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
