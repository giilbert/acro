// TODO: refactor this!!!

class Entity {
  constructor(generation, index) {
    this.generation = generation;
    this.index = index;
  }

  newAttachment(componentId, path) {
    return new Attachment(this, componentId, path);
  }
}

class Attachment {
  constructor(entity, componentId, path) {
    this.entity = entity;
    this.componentId = componentId;
    this.path = path;
  }

  add(pathSegment) {
    return new Attachment(
      this.entity,
      this.componentId,
      this.path + "." + pathSegment
    );
  }
}

function getPropertyNumber(attachment) {
  return Deno.core.ops.op_get_property_number(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  );
}

function setPropertyNumber(attachment, value) {
  Deno.core.ops.op_set_property_number(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value
  );
}

function getPropertyVec3(attachment) {
  const value = Deno.core.ops.op_get_property_vec3(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path
  );

  Object.setPrototypeOf(value, Vec3.prototype);
  value.attachedTo = attachment;

  return value;
}

function setPropertyVec3(attachment, value) {
  Deno.core.ops.op_set_property_vec3(
    attachment.entity.generation,
    attachment.entity.index,
    attachment.componentId,
    attachment.path,
    value.x,
    value.y,
    value.z
  );
}

class Behavior {
  constructor(entity) {
    this.entity = entity;
    this.transform = this.getComponent(Transform);
  }

  getComponent(ComponentClass) {
    const attachment = this.entity.newAttachment(
      ComponentClass.getComponentId(),
      ""
    );

    if (ComponentClass === Transform) {
      return new Transform(
        new Vec3(0, 0, 0, attachment.add("position")),
        attachment
      );
    }

    throw new Error(`Unknown component class: ${ComponentClass}`);
  }
}

class Transform {
  static getComponentId() {
    return acro.COMPONENT_IDS.Transform;
  }

  constructor(position, attachment) {
    this._position = position;
    this.attachment = attachment;
  }

  get position() {
    if (this.attachment)
      this._position = getPropertyVec3(this.attachment.add("position"));
    return this._position;
  }

  set position(value) {
    if (this.attachment)
      setPropertyVec3(this.attachment.add("position"), value);
    this._position = value;
  }
}

class Vec3 {
  constructor(x, y, z, attachment) {
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
