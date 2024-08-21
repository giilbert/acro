type AnyEventListener = (...args: any[]) => void;

export class EventBridge {
  listeners: Map<number, AnyEventListener>;

  constructor() {
    this.listeners = new Map();
  }

  addListener(id: number, listener: AnyEventListener) {
    this.listeners.set(id, listener);
  }

  removeListener(id: number) {
    this.listeners.delete(id);
  }
}
