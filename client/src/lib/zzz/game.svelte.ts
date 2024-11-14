import { GameEvent, type GameExpects, type PlayerEliminated, type PlayerResponds } from '$lib/models/game';
import type { SvelteMap } from 'svelte/reactivity';
import { encode } from '@msgpack/msgpack';
import { listenForMessagesOnWebSocket } from '$lib/utils/websocket';
import { parse } from 'valibot';

export class RunGameState {
    #ws: WebSocket;
    #onMessageController: AbortController;

    players: SvelteMap<bigint, string>;
    events = $state<(GameExpects | PlayerEliminated)[]>([]);

    constructor(ws: WebSocket, players: SvelteMap<bigint, string>) {
        this.#ws = ws;
        this.players = players;
        this.#onMessageController = listenForMessagesOnWebSocket(ws, data => {
            const event = parse(GameEvent, data);
            switch (event.type) {
                case 'GameExpects':
                    this.events.push(event);
                    break;
                case 'PlayerEliminated':
                    this.players.delete(event.pid);
                    this.events.push(event);
                    break;
                default:
                    throw new Error('unknown game event type');
            }
        });
    }

    respond(response: PlayerResponds) {
        this.#ws.send(encode(response, { useBigInt64: true }));
    }

    close() {
        this.#onMessageController.abort();
        this.#ws.close();
    }
}
