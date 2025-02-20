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

    function ringColorClasses(action: string) {
        switch (action) {
            case 'Zip':
                return 'ring-info';
            case 'Zap':
                return 'ring-success';
            case 'Zop':
                return 'ring-warning';
        }
    }
</script>

<div {style} bind:this={node.current} {...attributes.current} {...listeners.current}>
    <ActionButton
        {disabled}
        class={cn(isDragging.current && `btn-ghost ring-2 ring-offset-2 ${ringColorClasses(id)}`)}
        action={id}
    />
</div>
