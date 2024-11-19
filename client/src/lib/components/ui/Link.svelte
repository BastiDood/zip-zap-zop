<script lang="ts">
    import type { HTMLAnchorAttributes } from 'svelte/elements';
    import type { Snippet } from 'svelte';

    interface Props extends Omit<HTMLAnchorAttributes, 'class'> {
        children: Snippet;
    }

    const { children, ...rest }: Props = $props();
</script>

<a {...rest} class="rounded-lg border-2 border-solid px-8 py-2 text-xl">{@render children()}</a>
