import * as v from 'valibot';
import { Id } from './id';

export interface StartGame {
    count: bigint;
}

export const GameStarted = v.object({
    count: v.bigint(),
});

export type GameStarted = v.InferOutput<typeof GameStarted>;

const enum PlayerAction {
    Zip = 0,
    Zap,
    Zop,
}

export interface GameExpects {
    pid: Id;
    action: PlayerAction;
    deadline: Date;
}

export interface PlayerResponds {
    next: Id;
    action: PlayerAction;
}

export const PlayerEliminated = v.object({
    pid: Id,
});
