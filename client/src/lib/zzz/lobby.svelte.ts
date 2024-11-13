import { connectToWebSocket, listenForMessagesOnWebSocket } from '$lib/utils/websocket';
import { CreateLobbyEvent, type CreateLobby, type LobbyCreated } from '$lib/models/lobby';
import { RunGameState } from './game.svelte';
import type { StartGame } from '$lib/models/game';
import { SvelteMap } from 'svelte/reactivity';
import { ZZZ_WEBSOCKET_BASE_URL } from '$lib/env';
import { encode } from '@msgpack/msgpack';
import { parse } from 'valibot';

export class CreateLobbyState {
    #ws: WebSocket;
    #onMessageController: AbortController;

    meta = $state<Omit<LobbyCreated, 'type'> | null>(null);
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

    commit() {
        this.#onMessageController.abort();
        this.#ws.send(encode({ count: BigInt(this.players.size) } satisfies StartGame, { useBigInt64: true }));
        return new RunGameState(this.#ws, this.players);
    }
}
