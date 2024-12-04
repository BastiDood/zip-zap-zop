import type { Config } from 'tailwindcss';
import DaisyUi from 'daisyui';
import typo from '@tailwindcss/typography';

export default {
    experimental: { optimizeUniversalDefaults: true },
    content: ['./src/**/*.{html,js,svelte,ts}'],
    plugins: [DaisyUi, typo],
    daisyui: { themes: ['dim'], logs: false },
} satisfies Config;
