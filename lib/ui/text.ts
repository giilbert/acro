import {
  type Attachment,
  getPropertyBoolean,
  getPropertyNumber,
  getPropertyString,
  setPropertyBoolean,
  setPropertyNumber,
  setPropertyString,
} from "jsr:@acro/core";

export class Text {
  private _content: string;
  private _fontSize: number;
  private _lineHeight: number;
  private _weight: number;
  private _italic: boolean;

  attachment?: Attachment;

  static getComponentId() {
    return acro.COMPONENT_IDS["Text"];
  }

  static createDefault(attachment: Attachment) {
    return new Text("", 14, 16, 400, false, attachment);
  }

  constructor(
    content: string,
    fontSize: number,
    lineHeight: number,
    weight: number,
    italic: boolean,
    attachment?: Attachment
  ) {
    this._content = content;
    this._fontSize = fontSize;
    this._lineHeight = lineHeight;
    this._weight = weight;
    this._italic = italic;

    this.attachment = attachment;
  }

  get content() {
    if (this.attachment)
      this._content = getPropertyString(this.attachment.add("content"));
    return this._content;
  }

  set content(value: string) {
    if (this.attachment)
      setPropertyString(this.attachment.add("content"), value);
    this._content = value;
  }

  get fontSize() {
    if (this.attachment)
      this._fontSize = getPropertyNumber(this.attachment.add("font_size"));
    return this._fontSize;
  }

  set fontSize(value: number) {
    if (this.attachment)
      setPropertyNumber(this.attachment.add("font_size"), value);
    this._fontSize = value;
  }

  get lineHeight() {
    if (this.attachment)
      this._lineHeight = getPropertyNumber(this.attachment.add("line_height"));
    return this._lineHeight;
  }

  set lineHeight(value: number) {
    if (this.attachment)
      setPropertyNumber(this.attachment.add("line_height"), value);
    this._lineHeight = value;
  }

  get weight() {
    if (this.attachment)
      this._weight = getPropertyNumber(this.attachment.add("weight"));
    return this._weight;
  }

  set weight(value: number) {
    if (this.attachment) {
      setPropertyNumber(this.attachment.add("weight"), value);
    }
    this._weight = value;
  }

  get italic() {
    if (this.attachment)
      this._italic = getPropertyBoolean(this.attachment.add("italic"));
    return this._italic;
  }

  set italic(value: boolean) {
    if (this.attachment)
      setPropertyBoolean(this.attachment.add("italic"), value);

    this._italic = value;
  }
}
