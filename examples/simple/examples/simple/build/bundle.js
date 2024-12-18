(() => {
  var r = class e {
    _x;
    _y;
    _z;
    attachment;
    constructor(t, n, s, m) {
      (this._x = t), (this._y = n), (this._z = s), (this.attachment = m);
    }
    add(t) {
      return new e(this.x + t.x, this.y + t.y, this.z + t.z);
    }
    addAssign(t) {
      (this.x += t.x), (this.y += t.y), (this.z += t.z);
    }
    sub(t) {
      return new e(this.x - t.x, this.y - t.y, this.z - t.z);
    }
    subAssign(t) {
      (this.x -= t.x), (this.y -= t.y), (this.z -= t.z);
    }
    scale(t) {
      return new e(this.x * t, this.y * t, this.z * t);
    }
    dot(t) {
      return this.x * t.x + this.y * t.y + this.z * t.z;
    }
    cross(t) {
      return new e(
        this.y * t.z - this.z * t.y,
        this.z * t.x - this.x * t.z,
        this.x * t.y - this.y * t.x
      );
    }
    get magnitude() {
      return Math.sqrt(this.x * this.x + this.y * this.y + this.z * this.z);
    }
    get normalized() {
      return this.scale(1 / this.magnitude);
    }
    set x(t) {
      this.attachment && o(this.attachment.add("x"), t), (this._x = t);
    }
    get x() {
      return (
        this.attachment && (this._x = i(this.attachment.add("x"))), this._x
      );
    }
    set y(t) {
      this.attachment && o(this.attachment.add("y"), t), (this._y = t);
    }
    get y() {
      return (
        this.attachment && (this._y = i(this.attachment.add("y"))), this._y
      );
    }
    set z(t) {
      this.attachment && o(this.attachment.add("z"), t), (this._z = t);
    }
    get z() {
      return (
        this.attachment && (this._z = i(this.attachment.add("z"))), this._z
      );
    }
  };
  var b = class e {
    _position;
    _rotation;
    _scale;
    attachment;
    static getComponentId() {
      return acro.COMPONENT_IDS.Transform;
    }
    static createDefault(t) {
      return new e(new r(0, 0, 0), new r(0, 0, 0), new r(1, 1, 1), t);
    }
    constructor(t, n, s, m) {
      (this._position = t),
        (this._rotation = n),
        (this._scale = s),
        (this.attachment = m);
    }
    get position() {
      return (
        this.attachment &&
          (this._position = p(this.attachment.add("position"))),
        this._position
      );
    }
    set position(t) {
      if (this.attachment) {
        let n = this.attachment.add("position");
        u(n, t), (t.attachment = n);
      }
      this._position = t;
    }
    get rotation() {
      return (
        this.attachment &&
          (this._rotation = p(this.attachment.add("rotation"))),
        this._rotation
      );
    }
    set rotation(t) {
      if (this.attachment) {
        let n = this.attachment.add("rotation");
        u(n, t), (t.attachment = n);
      }
      this._rotation = t;
    }
    get scale() {
      return (
        this.attachment && (this._scale = p(this.attachment.add("scale"))),
        this._scale
      );
    }
    set scale(t) {
      if (this.attachment) {
        let n = this.attachment.add("scale");
        u(n, t), (t.attachment = n);
      }
      this._scale = t;
    }
    get forward() {
      return new r(
        Math.cos(-this.rotation.x) * Math.sin(-this.rotation.y),
        -Math.sin(-this.rotation.x),
        Math.cos(-this.rotation.x) * Math.cos(-this.rotation.y)
      ).normalized;
    }
    get right() {
      return new r(Math.cos(-this.rotation.y), 0, -Math.sin(-this.rotation.y))
        .normalized;
    }
    get up() {
      return this.forward.cross(this.right);
    }
  };
  var c = class e {
    _x;
    _y;
    attachment;
    constructor(t, n, s) {
      (this._x = t), (this._y = n), (this.attachment = s);
    }
    add(t) {
      return new e(this.x + t.x, this.y + t.y);
    }
    addAssign(t) {
      (this.x += t.x), (this.y += t.y);
    }
    sub(t) {
      return new e(this.x - t.x, this.y - t.y);
    }
    subAssign(t) {
      (this.x -= t.x), (this.y -= t.y);
    }
    scale(t) {
      return new e(this.x * t, this.y * t);
    }
    dot(t) {
      return this.x * t.x + this.y * t.y;
    }
    get magnitude() {
      return Math.sqrt(this.x * this.x + this.y * this.y);
    }
    get normalized() {
      return this.scale(1 / this.magnitude);
    }
    set x(t) {
      this.attachment && o(this.attachment.add("x"), t), (this._x = t);
    }
    get x() {
      return (
        this.attachment && (this._x = i(this.attachment.add("x"))), this._x
      );
    }
    set y(t) {
      this.attachment && o(this.attachment.add("y"), t), (this._y = t);
    }
    get y() {
      return (
        this.attachment && (this._y = i(this.attachment.add("y"))), this._y
      );
    }
  };
  function A(e) {
    return Deno.core.ops.op_get_property_string(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path
    );
  }
  function P(e, t) {
    Deno.core.ops.op_set_property_string(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path,
      t
    );
  }
  function i(e) {
    return Deno.core.ops.op_get_property_number(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path
    );
  }
  function o(e, t) {
    Deno.core.ops.op_set_property_number(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path,
      t
    );
  }
  function v(e) {
    return Deno.core.ops.op_get_property_boolean(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path
    );
  }
  function M(e, t) {
    Deno.core.ops.op_set_property_boolean(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path,
      t
    );
  }
  function p(e) {
    let t = Deno.core.ops.op_get_property_vec3(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path
    );
    return new r(t.x, t.y, t.z, e);
  }
  function u(e, t) {
    Deno.core.ops.op_set_property_vec3(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path,
      t.x,
      t.y,
      t.z
    );
  }
  function V(e, t) {
    Deno.core.ops.op_call_function(
      e.entity.generation,
      e.entity.index,
      e.componentId,
      e.path,
      t
    );
  }
  var y = class {
      generation;
      index;
      constructor(t, n) {
        (this.generation = t), (this.index = n);
      }
      newAttachment(t, n) {
        return new f(this, t, n);
      }
      getComponent(t) {
        let n = this.newAttachment(t.getComponentId(), "");
        return t.createDefault(n);
      }
    },
    f = class e {
      entity;
      componentId;
      path;
      constructor(t, n, s) {
        (this.entity = t), (this.componentId = n), (this.path = s);
      }
      add(t) {
        return new e(this.entity, this.componentId, this.path + "." + t);
      }
    },
    h = class {
      entity;
      transform;
      constructor(t) {
        (this.entity = t), (this.transform = this.getComponent(b));
      }
      getComponent(t) {
        return this.entity.getComponent(t);
      }
      update(t) {}
    },
    l = (e) => {
      let t = Deno.core.ops.op_get_entity_by_absolute_path(e);
      return t ? new y(t.generation, t.index) : null;
    };
  var d = class {
    attachment;
    handlers;
    constructor(t) {
      (this.attachment = t), (this.handlers = []);
    }
    bind(t) {
      this.handlers.push(t),
        this.attachment && V(this.attachment.add("bind"), [t]);
    }
  };
  var B = (e) => Deno.core.ops.op_get_key_press(JSON.stringify(e)),
    N = (e) => Deno.core.ops.op_get_mouse_press(JSON.stringify(e)),
    S = () => {
      let [e, t] = Deno.core.ops.op_get_mouse_position();
      return new c(e, t);
    },
    a = { isKeyPressed: B, isMousePressed: N, getMousePosition: S };
  var g = 20;
  var w = class extends h {
      lastMousePosition = new c(0, 0);
      constructor(t) {
        super(t);
      }
      update(t) {
        let s = a.getMousePosition().sub(this.lastMousePosition);
        (this.lastMousePosition = a.getMousePosition()),
          a.isKeyPressed("KeyW")
            ? this.transform.position.addAssign(
                this.transform.forward.scale(g * t)
              )
            : a.isKeyPressed("KeyS") &&
              this.transform.position.addAssign(
                this.transform.forward.scale(-g * t)
              ),
          a.isKeyPressed("KeyA")
            ? this.transform.position.addAssign(
                this.transform.right.scale(g * t)
              )
            : a.isKeyPressed("KeyD") &&
              this.transform.position.addAssign(
                this.transform.right.scale(-g * t)
              );
      }
    },
    I = () => acro.registerBehavior("FlyCamera", w);
  var _ = class e {
    _content;
    _fontSize;
    _lineHeight;
    _weight;
    _italic;
    attachment;
    static getComponentId() {
      return acro.COMPONENT_IDS.Text;
    }
    static createDefault(t) {
      return new e("", 14, 16, 400, !1, t);
    }
    constructor(t, n, s, m, E, k) {
      (this._content = t),
        (this._fontSize = n),
        (this._lineHeight = s),
        (this._weight = m),
        (this._italic = E),
        (this.attachment = k);
    }
    get content() {
      return (
        this.attachment && (this._content = A(this.attachment.add("content"))),
        this._content
      );
    }
    set content(t) {
      this.attachment && P(this.attachment.add("content"), t),
        (this._content = t);
    }
    get fontSize() {
      return (
        this.attachment &&
          (this._fontSize = i(this.attachment.add("font_size"))),
        this._fontSize
      );
    }
    set fontSize(t) {
      this.attachment && o(this.attachment.add("font_size"), t),
        (this._fontSize = t);
    }
    get lineHeight() {
      return (
        this.attachment &&
          (this._lineHeight = i(this.attachment.add("line_height"))),
        this._lineHeight
      );
    }
    set lineHeight(t) {
      this.attachment && o(this.attachment.add("line_height"), t),
        (this._lineHeight = t);
    }
    get weight() {
      return (
        this.attachment && (this._weight = i(this.attachment.add("weight"))),
        this._weight
      );
    }
    set weight(t) {
      this.attachment && o(this.attachment.add("weight"), t),
        (this._weight = t);
    }
    get italic() {
      return (
        this.attachment && (this._italic = v(this.attachment.add("italic"))),
        this._italic
      );
    }
    set italic(t) {
      this.attachment && M(this.attachment.add("italic"), t),
        (this._italic = t);
    }
  };
  var x = class e {
    click;
    static getComponentId() {
      return acro.COMPONENT_IDS.Button;
    }
    static createDefault(t) {
      return new e(t);
    }
    constructor(t) {
      this.click = new d(t?.add("click"));
    }
  };
  var z = class extends h {
      text;
      button;
      constructor(t) {
        super(t),
          (this.text = l("/UI/Panel/Text")?.getComponent(_)),
          (this.button = l("/UI/Panel 2")?.getComponent(x)),
          this.button.click.bind(() => (this.transform.position.y += 1));
      }
      update(t) {
        a.isMousePressed("Left") && (this.transform.rotation.y -= 5 * t),
          a.isMousePressed("Right") && (this.transform.rotation.z += 5 * t),
          (this.text.content = `y rotation (radians): ${this.transform.rotation.y.toFixed(
            2
          )}`);
      }
    },
    D = () => acro.registerBehavior("TestBehavior", z);
  (void 0)?.();
  I?.();
  D?.();
})();
