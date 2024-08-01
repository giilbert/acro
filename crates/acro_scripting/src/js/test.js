class TestBehavior extends Behavior {
  constructor(entity) {
    super(entity);
  }

  update() {
    const a = this.transform.position.x;
    console.log(this.transform.position.x);
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
