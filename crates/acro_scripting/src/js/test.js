class TestBehavior extends Behavior {
  constructor() {
    super();
    Deno.core.print("TestBehavior constructor\n");
  }
}

acro.registerBehavior("TestBehavior", TestBehavior);
