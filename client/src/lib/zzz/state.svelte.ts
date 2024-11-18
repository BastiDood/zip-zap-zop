import type { CreateLobby, JoinLobby } from '$lib/models/lobby';
import { GameEvent, GuestEvent, HostEvent } from '$lib/models';
import type { GameExpected, StartGame } from '$lib/models/game';
import type { Id } from '$lib/models/id';

import { decode, encode } from '@msgpack/msgpack';
import { parse } from 'valibot';

import { SvelteMap } from 'svelte/reactivity';
import { ZZZ_WEBSOCKET_BASE_URL } from '$lib/env';

function send(ws: WebSocket, data: unknown) {
    ws.send(encode(data, { useBigInt64: true }));
}

export class State {
    #ws: WebSocket;
    #schema: typeof HostEvent | typeof GuestEvent | typeof GameEvent;

    /** Known ID-to-name mappings for all players. */
    players = new SvelteMap<Id, string>();
    /** The last expected response by the server. */
    expected = $state<GameExpected | null>(null);
    /** The latest player eliminated from the game. */
    eliminated = $state<Id | null>(null);
    /** Player ID of the game winner. */
    winner = $state<Id | null>(null);

    /** Lobby ID */
    lid = $state<Id | null>(null);
    /** Lobby name. */
    lobby = $state<string | null>(null);

    /** Player ID */
    pid = $state<Id | null>(null);
    /** Player name. */
    player = $state<string | null>(null);

    private constructor(ws: WebSocket, schema: typeof GuestEvent | typeof HostEvent, player: string) {
        ws.binaryType = 'arraybuffer';
        this.#ws = ws;
        this.#schema = schema;
        this.player = player;
    }

    /** Start the state machine as a lobby "host". */
    static host(lobby: string, player: string) {
        const state = new State(new WebSocket(new URL('create', ZZZ_WEBSOCKET_BASE_URL)), HostEvent, player);
        state.lobby = lobby;
        state.#ws.addEventListener(
            'open',
            () => {
                const controller = new AbortController();
                state.#ws.addEventListener('close', () => controller.abort(), { once: true });
                state.#ws.addEventListener(
                    'message',
                    event => {
                        if (event.data instanceof ArrayBuffer) state.#tick(decode(event.data, { useBigInt64: true }));
                        else throw new Error('unexpected message format');
                    },
                    { signal: controller.signal },
                );
                send(state.#ws, { lobby, player } satisfies CreateLobby);
            },
            { once: true },
        );
        return state;
    }

    /** Start the state machine as a lobby "guest". */
    static guest(lid: Id, player: string) {
        const state = new State(new WebSocket(new URL('join', ZZZ_WEBSOCKET_BASE_URL)), GuestEvent, player);
        state.lid = lid;
        state.#ws.addEventListener(
            'open',
            () => {
                const controller = new AbortController();
                state.#ws.addEventListener('close', () => controller.abort(), { once: true });
                state.#ws.addEventListener(
                    'message',
                    event => {
                        if (event.data instanceof ArrayBuffer) state.#tick(decode(event.data, { useBigInt64: true }));
                        else throw new Error('unexpected message format');
                    },
                    { signal: controller.signal },
                );
                send(state.#ws, { lid, player } satisfies JoinLobby);
            },
            { once: true },
        );
        return state;
    }

    #tick(data: unknown) {
        const event = parse(this.#schema, data);
        console.log(event);
        switch (event.type) {
            case 'LobbyCreated':
                this.lid = event.lid;
                this.pid = event.pid;
                break;
            case 'LobbyJoined':
                this.lobby = event.lobby;
                this.pid = event.pid;
                break;
            case 'LobbyPlayerJoined':
                this.players.set(event.pid, event.player);
                break;
            case 'GameEliminated':
                if (this.pid === event.pid) this.pid = null;
                this.eliminated = event.pid;
                this.expected = null;
            // falls through
            case 'LobbyPlayerLeft':
                this.players.delete(event.pid);
                break;
            case 'GameStarted':
                if (BigInt(this.players.size + 1) !== event.count) throw new Error('player count mismatch');
                this.#schema = GameEvent;
                break;
            case 'GameExpected':
                this.expected = event;
                break;
            case 'GameConcluded':
                this.winner = event.pid;
                this.pid = null;
                this.lid = null;
                this.lobby = null;
                this.#ws.close();
                break;
        }
    }

    /** Host: start the game. */
    start() {
        if (this.#schema !== HostEvent) throw new Error('player is not the host');
        send(this.#ws, { count: BigInt(this.players.size + 1) } satisfies StartGame);
    }
}
