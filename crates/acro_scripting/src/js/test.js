class TestBehavior extends Behavior {
  constructor(entity) {
    super(entity);
    console.log(this.transform.position);
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
