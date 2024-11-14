import {
    type CreateLobby,
    CreateLobbyEvent,
    type JoinLobby,
    JoinLobbyEvent,
    type LobbyCreated,
    type LobbyJoined,
} from '$lib/models/lobby';
import type { GameStarted, StartGame } from '$lib/models/game';
import { connectToWebSocket, listenForMessagesOnWebSocket } from '$lib/utils/websocket';
import { RunGameState } from './game.svelte';
import { SvelteMap } from 'svelte/reactivity';
import { ZZZ_WEBSOCKET_BASE_URL } from '$lib/env';
import { encode } from '@msgpack/msgpack';
import { parse } from 'valibot';

export class GameNotYetStartedError extends Error {
    constructor() {
        super('game has not yet started');
        this.name = 'GameNotYetStartedError';
    }
}

export class CreateLobbyState {
    #ws: WebSocket;
    #onMessageController: AbortController;

    meta = $state<LobbyCreated | null>(null);
    players = new SvelteMap<bigint, string>();

    private constructor(ws: WebSocket, lobby: string, player: string) {
        this.#onMessageController = listenForMessagesOnWebSocket(ws, data => {
            const event = parse(CreateLobbyEvent, data);
            switch (event.type) {
                case 'LobbyCreated':
                    this.meta = event;
                    break;
                case 'LobbyPlayerJoined':
                    this.players.set(event.pid, event.player);
                    break;
                case 'LobbyPlayerLeft':
                    this.players.delete(event.pid);
                    break;
                default:
                    throw new Error('unknown lobby event type');
            }
        });
        ws.send(encode({ lobby, player } satisfies CreateLobby, { useBigInt64: true }));
        this.#ws = ws;
    }

    static async host(lobby: string, player: string, base = ZZZ_WEBSOCKET_BASE_URL) {
        const ws = new WebSocket(new URL('create', base));
        ws.binaryType = 'arraybuffer';
        await connectToWebSocket(ws);
        return new CreateLobbyState(ws, lobby, player);
    }

    async commit() {
        this.#onMessageController.abort();
        this.#ws.send(encode({ count: BigInt(this.players.size) } satisfies StartGame, { useBigInt64: true }));
        return await RunGameState.start(this.#ws, this.players);
    }
}

export class JoinLobbyState {
    #ws: WebSocket;
    #onMessageController: AbortController;

    meta = $state<LobbyJoined | null>(null);
    players = new SvelteMap<bigint, string>();
    ready = $state<GameStarted | null>(null);

    private constructor(ws: WebSocket, lid: bigint, player: string) {
        this.#onMessageController = listenForMessagesOnWebSocket(ws, data => {
            const event = parse(JoinLobbyEvent, data);
            switch (event.type) {
                case 'LobbyJoined':
                    this.meta = event;
                    break;
                case 'LobbyPlayerJoined':
                    this.players.set(event.pid, event.player);
                    break;
                case 'LobbyPlayerLeft':
                    this.players.delete(event.pid);
                    break;
                case 'GameStarted':
                    this.ready = event;
                    break;
                default:
                    throw new Error('unknown lobby event type');
            }
        });
        ws.send(encode({ lid, player } satisfies JoinLobby, { useBigInt64: true }));
        this.#ws = ws;
    }

    static async join(lid: bigint, player: string, base = ZZZ_WEBSOCKET_BASE_URL) {
        const ws = new WebSocket(new URL('join', base));
        ws.binaryType = 'arraybuffer';
        await connectToWebSocket(ws);
        return new JoinLobbyState(ws, lid, player);
    }

    commit() {
        if (this.ready === null) throw new GameNotYetStartedError();
        this.#onMessageController.abort();
        this.#ws.send(new ArrayBuffer(0));
        return new RunGameState(this.#ws, this.players, this.ready.count);
    }
}
