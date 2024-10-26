import { type Attachment, EventEmitter } from "jsr:@acro/core";

export class Button {
  public click: EventEmitter;

  static getComponentId() {
    return acro.COMPONENT_IDS["Button"];
  }

  static createDefault(attachment: Attachment) {
    return new Button(attachment);
  }

  constructor(attachment?: Attachment) {
    this.click = new EventEmitter(attachment?.add("click"));
  }
}
