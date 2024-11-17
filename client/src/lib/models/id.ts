import * as v from 'valibot';

export const Id = v.pipe(v.number(), v.safeInteger());
export type Id = v.InferOutput<typeof Id>;
