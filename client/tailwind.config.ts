import type { Config } from 'tailwindcss';
import DaisyUi from 'daisyui';

export default {
    experimental: { optimizeUniversalDefaults: true },
    content: ['./src/**/*.{html,js,svelte,ts}'],
    plugins: [DaisyUi],
    daisyui: { themes: ['dim'] },
} satisfies Config;
