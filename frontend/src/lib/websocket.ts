import { Keys, Select } from "lib/util";

export type WSDiconnectHandler = () => void;
export type WSErrorHandler = (error?: string | Error | object) => void;
export type WSHandlerMap<M extends object> = {
  [K in Keys<M>]?: (msg: Select<M, K>) => void;
};

/**
 * WebSocket Disconnect Codes
 * @see https://developer.mozilla.org/en-US/docs/Web/API/CloseEvent/code
 */
export const WEBSOCKET_CODES = {
  CLOSE_NORMAL: 1000,
  CLOSE_GOING_AWAY: 1001,
  CLOSE_ABNORMAL: 1006,
  SERVER_ERROR: 1011,
  SERVICE_RESTART: 1012,
};

export type WSEvent =
  | { Disconnect: CloseEvent }
  | { Connect: void }
  | { Error: Event };

export class WebSocketApi<
  S extends { [K: string]: unknown },
  C extends { [K: string]: unknown }
> {
  public lastMessage?: Date;

  private connection?: WebSocket;
  private endpoint!: string;

  private handlers: WSHandlerMap<S> = {};
  private metaHandlers: WSHandlerMap<WSEvent> = {};

  private messageQueue: C[] = [];

  constructor(endpoint: string) {
    this.connect(endpoint);
  }

  public reconnect(force: boolean = false) {
    // Don't try to reconnect if there is a connection already
    if (this.isOpen() && force) return;

    this.disconnect();
    this.connect(this.endpoint);
  }

  public isOpen(): boolean {
    return this.connection?.readyState === this.connection?.OPEN;
  }

  public connect(endpoint: string) {
    console.log("Connecting to", endpoint);
    this.endpoint = endpoint;
    this.connection = new WebSocket(endpoint);
    this.connection.onerror = (e) => this.metaHandlers["Error"]?.(e);
    this.connection.onclose = (e) => this.metaHandlers["Disconnect"]?.(e);

    this.connection.onopen = () => {
      this.metaHandlers["Connect"]?.();
      this.flushMessageQuque();
    };

    this.connection.onmessage = (e) => {
      const res = this.parseMsg(e.data);
      if (res) this.handleMessage(res);
    };
  }

  private parseMsg(msg: string): S | undefined {
    try {
      const json = JSON.parse(msg) as S;
      return json;
    } catch (e) {
      this.metaHandlers.Error?.(e as Event);
      return undefined;
    }
  }

  private handleMessage(msg: S) {
    this.lastMessage = new Date();
    for (const k in msg) {
      // There should be a better way than type casting two times. But it's 3am.
      const key = k as unknown as Keys<S>;
      if (!this.handlers[key])
        console.warn(`No message handler found for message type ${key}`);
      // Oh lord
      this.handlers[key]?.(msg[key as keyof S] as Select<S, Keys<S>>);
    }
  }

  public register<T extends Keys<S>>(
    type: T,
    handler: WSHandlerMap<S>[T]
  ): WebSocketApi<S, C> {
    this.handlers[type] = handler;
    return this;
  }

  public registerEvent<T extends Keys<WSEvent>>(
    type: T,
    handler: WSHandlerMap<WSEvent>[T]
  ) {
    this.metaHandlers[type] = handler;
    return this;
  }

  public queueMsg(msg: C) {
    this.messageQueue.push(msg);
  }

  public flushMessageQuque() {
    for (const item of this.messageQueue) {
      this.send(item);
    }
  }

  public send(msg: C) {
    if (this.isOpen()) this.connection?.send(JSON.stringify(msg));
    else this.queueMsg(msg);
  }

  public disconnect() {
    this.connection?.close();
  }
}
