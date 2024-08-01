class TestBehavior extends Behavior {
  constructor(entity) {
    super(entity);
  }

  update() {
    const a = this.transform.position.x;
    // console.log(a);
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
