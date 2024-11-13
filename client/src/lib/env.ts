import { PUBLIC_ZZZ_WEBSOCKET_BASE_URL } from '$env/static/public';

if (!PUBLIC_ZZZ_WEBSOCKET_BASE_URL || !URL.canParse(PUBLIC_ZZZ_WEBSOCKET_BASE_URL))
    throw new Error('missing websocket url for game server');

export const ZZZ_WEBSOCKET_BASE_URL = PUBLIC_ZZZ_WEBSOCKET_BASE_URL;
