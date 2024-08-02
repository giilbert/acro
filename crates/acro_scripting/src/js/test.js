class TestBehavior extends Behavior {
  constructor(entity) {
    super(entity);
  }

  update() {
    // console.log(this.transform.position.x);
    this.transform.position.x -= 0.00001;
    // console.log(this.transform.position.x);
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
