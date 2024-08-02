class TestBehavior extends Behavior {
  constructor(entity) {
    super(entity);
  }

  update() {
    // console.log(this.transform.position.x);
    this.transform.position = new Vec3(-11, 4, 3);
    // console.log(this.transform.position.x);
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
