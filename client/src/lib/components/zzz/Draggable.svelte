<script lang="ts">
    import ActionButton from './ActionButton.svelte';
    import { CSS } from '@dnd-kit-svelte/utilities';
    import { cn } from '$lib/utils/cn';
    import { useDraggable } from '@dnd-kit-svelte/core';

    const { id, disabled } = $props();
    const { transform, listeners, attributes, node, isDragging } = useDraggable({ id, disabled: () => disabled });
    const style = $derived(
        transform.current && !isDragging.current ? `transform: ${CSS.Translate.toString(transform.current)}` : '',
    );
</script>

<div {style} bind:this={node.current} {...attributes.current} {...listeners.current}>
    <ActionButton
        {disabled}
        class={cn(
            { 'btn-ghost ring-2 ring-offset-2': isDragging.current },
            isDragging.current && {
                'ring-info': id === 'Zip',
                'ring-success': id === 'Zap',
                'ring-warning': id === 'Zop',
            },
        )}
        action={id}
    />
</div>
