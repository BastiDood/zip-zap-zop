<script lang="ts">
    import { type UseDroppableArguments, useDroppable } from '@dnd-kit-svelte/core';
    import type { Snippet } from 'svelte';

    interface DroppableProps extends UseDroppableArguments {
        children: Snippet;
    }

    const { children, ...rest }: DroppableProps = $props();
    const droppable = useDroppable(rest);
</script>

<div
    class="rounded-xl border bg-base-300 px-4 py-2 text-neutral-content {droppable.isOver.current
        ? 'animate-pulse border-neutral-content'
        : 'border-base-300'}"
    bind:this={droppable.node.current}
>
    {@render children()}
</div>
