import { GameConcluded, GameExpected, GameStarted, PlayerEliminated } from './game';
import { LobbyCreated, LobbyJoined, LobbyPlayerJoined, LobbyPlayerLeft } from './lobby';

import { type InferOutput, variant } from 'valibot';

export const HostEvent = variant('type', [LobbyCreated, LobbyPlayerJoined, LobbyPlayerLeft, GameStarted]);
export type HostEvent = InferOutput<typeof HostEvent>;

export const GuestEvent = variant('type', [LobbyJoined, LobbyPlayerJoined, LobbyPlayerLeft, GameStarted]);
export type GuestEvent = InferOutput<typeof GuestEvent>;

export const GameEvent = variant('type', [GameExpected, PlayerEliminated, GameConcluded]);
export type GameEvent = InferOutput<typeof GameEvent>;
