// TODO: this shouldnt be a global
class Behavior {}

class AcroGlobalHook {
  constructor() {
    this.behaviorConstructors = {};
    this.behaviors = {};
  }

  update() {
    for (const behavior of Object.values(this.behaviors)) {
      behavior.update();
    }
  }

  registerBehavior(name, behavior) {
    this.behaviorConstructors[name] = behavior;
  }

  createBehavior(id, name, ...args) {
    Deno.core.print(
      `createBehavior(${id}, ${JSON.stringify(name)}, ${args})\n`
    );
    const behavior = new this.behaviorConstructors[name](id, ...args);
    this.behaviors[behavior.id] = behavior;
    return behavior;
  }
}

globalThis.acro = new AcroGlobalHook();
