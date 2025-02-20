<script lang="ts">
    import { type UseDroppableArguments, useDroppable } from '@dnd-kit-svelte/core';
    import type { Snippet } from 'svelte';
    import { cn } from '$lib/utils/cn';

    interface DroppableProps extends UseDroppableArguments {
        children: Snippet;
    }

    const { children, ...rest }: DroppableProps = $props();
    const droppable = useDroppable(rest);
</script>

<div
    class={cn('rounded-xl border border-base-300 bg-base-300 px-4 py-2 text-neutral-content', {
        'animate-pulse border-neutral-content': droppable.isOver.current,
    })}
    bind:this={droppable.node.current}
>
    {@render children()}
</div>
