import { error } from '@sveltejs/kit';

export const prerender = false;

function validateNonNull<T>(data: T | null) {
    if (data === null) error(404, 'missing lobby ID');
    return data;
}

export function load({ url }) {
    return { lid: BigInt(validateNonNull(url.searchParams.get('lid'))) };
}
