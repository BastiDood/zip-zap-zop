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

export const GameExpects = v.object({
    type: v.literal('GameExpects'),
    pid: Id,
    action: v.picklist([PlayerAction.Zip, PlayerAction.Zap, PlayerAction.Zop]),
    deadline: v.pipe(
        v.string(),
        v.transform(date => new Date(date)),
    ),
});

export const GameConcludes = v.object({
    type: v.literal('GameConcludes'),
    pid: Id,
});

export type GameExpects = v.InferOutput<typeof GameExpects>;
export type GameStarted = v.InferOutput<typeof GameStarted>;

export interface PlayerResponds {
    next: Id;
    action: PlayerAction;
}

export const PlayerEliminated = v.object({
    type: v.literal('PlayerEliminated'),
    pid: Id,
});

export type PlayerEliminated = v.InferOutput<typeof PlayerEliminated>;

export const GameEvent = v.variant('type', [GameExpects, PlayerEliminated, GameConcludes]);
export type GameEvent = v.InferOutput<typeof GameEvent>;
