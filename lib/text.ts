import { getPropertyString, setPropertyString } from "./deno.ts";
import type { Attachment } from "./core.ts";

export class Text {
  private _content: string;

  attachment: Attachment | undefined;

  static getComponentId() {
    return acro.COMPONENT_IDS["Text"];
  }

  constructor(content: string, attachment?: Attachment) {
    this._content = content;

    this.attachment = attachment;
  }

  get content() {
    if (this.attachment)
      this._content = getPropertyString(this.attachment.add("content"));
    return this._content;
  }

  set content(value) {
    if (this.attachment) {
      const attachment = this.attachment.add("content");
      setPropertyString(attachment, value);
    }
    this._content = value;
  }
}
