import * as v from 'valibot';
import { Id } from './id';

export interface CreateLobby {
    lobby: string;
    player: string;
}

export const LobbyCreated = v.object({
    type: v.literal('LobbyCreated'),
    lid: Id,
    pid: Id,
});

export const LobbyPlayerJoined = v.object({
    type: v.literal('LobbyPlayerJoined'),
    pid: Id,
    player: v.string(),
});

export const LobbyPlayerLeft = v.object({
    type: v.literal('LobbyPlayerLeft'),
    pid: Id,
});

export type LobbyCreated = v.InferOutput<typeof LobbyCreated>;
export type LobbyPlayerJoined = v.InferOutput<typeof LobbyPlayerJoined>;
export type LobbyPlayerLeft = v.InferOutput<typeof LobbyPlayerLeft>;

export interface JoinLobby {
    lid: Id;
    player: string;
}

export const LobbyJoined = v.object({
    type: v.literal('LobbyJoined'),
    lobby: v.string(),
    pid: Id,
});

export type LobbyJoined = v.InferOutput<typeof LobbyJoined>;
