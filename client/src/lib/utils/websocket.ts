import { decode } from '@msgpack/msgpack';

export class UnexpectedMessageTypeError extends Error {
    type: string;
    constructor(type: string) {
        super(`unexpected message type ${type}`);
        this.name = 'UnexpectedMessageTypeError';
        this.type = type;
    }
}

export class UnexpectedMessageDataError extends Error {
    constructor() {
        super('unexpected message data');
        this.name = 'UnexpectedMessageDataError';
    }
}

export function connectToWebSocket(ws: WebSocket) {
    return new Promise<Event>((resolve, reject) => {
        const success = new AbortController();
        const close = new AbortController();
        const onSuccess = (event: Event) => {
            resolve(event);
            close.abort();
        };
        const onClose = (event: Event) => {
            reject(event);
            success.abort();
        };
        ws.addEventListener('open', onSuccess, { once: true, signal: success.signal });
        ws.addEventListener('close', onClose, { once: true, signal: close.signal });
    });
}

function assertArrayBufferPayload(event: MessageEvent) {
    if (event.type !== 'binary') throw new UnexpectedMessageTypeError(event.type);
    if (event.data instanceof ArrayBuffer) return event.data;
    throw new UnexpectedMessageDataError();
}

type OnMessageDataCallback = (data: unknown) => void;
export function listenForMessagesOnWebSocket(ws: WebSocket, onMessageData: OnMessageDataCallback) {
    const message = new AbortController();
    ws.addEventListener(
        'message',
        event => onMessageData(decode(assertArrayBufferPayload(event), { useBigInt64: true })),
        { signal: message.signal },
    );
    return message;
}
