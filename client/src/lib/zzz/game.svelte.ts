import { GameEvent, type GameExpected, GameStarted, type PlayerEliminated, type PlayerResponds } from '$lib/models/game';
import {
    assertArrayBufferPayload,
    listenForMessagesOnWebSocket,
    waitForMessageOnWebSocket,
} from '$lib/utils/websocket';
import { decode, encode } from '@msgpack/msgpack';
import type { SvelteMap } from 'svelte/reactivity';
import { parse } from 'valibot';

export class UnexpectedPlayerCountError extends Error {
    expected: bigint;
    actual: bigint;

    constructor(expected: bigint, actual: bigint) {
        super(`expected player count to be ${expected} rather than ${actual}`);
        this.name = 'UnexpectedPlayerCountError';
        this.expected = expected;
        this.actual = actual;
    }
}

export class RunGameState {
    #ws: WebSocket;
    #onMessageController: AbortController;

    players: SvelteMap<bigint, string>;
    events = $state<(GameExpected | PlayerEliminated)[]>([]);
    winner = $state<bigint | null>(null);

    constructor(ws: WebSocket, players: SvelteMap<bigint, string>, expected: bigint) {
        const actual = BigInt(players.size);
        if (actual !== expected) throw new UnexpectedPlayerCountError(expected, actual);

        this.#ws = ws;
        this.players = players;
        this.#onMessageController = listenForMessagesOnWebSocket(ws, data => {
            const event = parse(GameEvent, data);
            switch (event.type) {
                case 'GameExpected':
                    this.events.push(event);
                    break;
                case 'PlayerEliminated':
                    this.players.delete(event.pid);
                    this.events.push(event);
                    break;
                case 'GameConcluded':
                    this.winner = event.pid;
                    break;
                default:
                    throw new Error('unknown game event type');
            }
        });
    }

    static async start(ws: WebSocket, players: SvelteMap<bigint, string>) {
        const data = decode(assertArrayBufferPayload(await waitForMessageOnWebSocket(ws)), { useBigInt64: true });
        const { count } = parse(GameStarted, data);
        return new RunGameState(ws, players, count);
    }

    respond(response: PlayerResponds) {
        this.#ws.send(encode(response, { useBigInt64: true }));
    }

    close() {
        this.#onMessageController.abort();
        this.#ws.close();
    }
}
