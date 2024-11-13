import type { SvelteMap } from 'svelte/reactivity';

export class RunGameState {
    #ws: WebSocket;
    #players: SvelteMap<bigint, string>;

    constructor(ws: WebSocket, players: SvelteMap<bigint, string>) {
        this.#ws = ws;
        this.#players = players;
        // TODO: Intercept game events.
    }
}
