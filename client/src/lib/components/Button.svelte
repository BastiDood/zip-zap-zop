<script lang="ts">
    import type { HTMLButtonAttributes } from 'svelte/elements';
    import type { Snippet } from 'svelte';

    interface Props extends Omit<HTMLButtonAttributes, 'class'> {
        children?: Snippet;
    }

    const { children, ...rest }: Props = $props();
</script>

<button {...rest} class="rounded-lg border-2 border-solid px-8 py-2 text-xl">{@render children?.()}</button>
