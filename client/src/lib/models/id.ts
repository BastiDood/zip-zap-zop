import * as v from 'valibot';

export const Id = v.bigint();
export type Id = v.InferOutput<typeof Id>;
