import * as v from 'valibot';
import { Id } from './id';

export interface StartGame {
    count: bigint;
}

export const GameStarted = v.object({
    type: v.literal('GameStarted'),
    count: v.bigint(),
});

const enum PlayerAction {
    Zip = 0,
    Zap,
    Zop,
}

export const GameExpected = v.object({
    type: v.literal('GameExpected'),
    pid: Id,
    action: v.picklist([PlayerAction.Zip, PlayerAction.Zap, PlayerAction.Zop]),
    deadline: v.pipe(
        v.string(),
        v.transform(date => new Date(date)),
    ),
});

export const GameConcluded = v.object({
    type: v.literal('GameConcluded'),
    pid: Id,
});

export type GameStarted = v.InferOutput<typeof GameStarted>;
export type GameExpected = v.InferOutput<typeof GameExpected>;
export type GameConcluded = v.InferOutput<typeof GameConcluded>;

export interface PlayerResponds {
    next: Id;
    action: PlayerAction;
}

export const PlayerEliminated = v.object({
    type: v.literal('PlayerEliminated'),
    pid: Id,
});

export type PlayerEliminated = v.InferOutput<typeof PlayerEliminated>;

export const GameEvent = v.variant('type', [GameExpected, PlayerEliminated, GameConcluded]);
export type GameEvent = v.InferOutput<typeof GameEvent>;
