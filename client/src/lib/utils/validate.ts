export function validateString(data: unknown) {
    if (typeof data === 'string') return data;
    throw new Error('expected string');
}
