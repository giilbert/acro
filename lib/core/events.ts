import { Attachment, callFunction } from "./mod.ts";

type AnyEventListener = (...args: any[]) => void;
export class EventEmitter {
  private attachment?: Attachment;

  private handlers: AnyEventListener[];

  constructor(attachment?: Attachment) {
    this.attachment = attachment;
    this.handlers = [];
  }

  public bind(handler: AnyEventListener) {
    this.handlers.push(handler);
    if (this.attachment) callFunction(this.attachment.add("bind"), [handler]);
  }
}
