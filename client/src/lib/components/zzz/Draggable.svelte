<script lang="ts">
    import ActionButton from './ActionButton.svelte';
    import { CSS } from '@dnd-kit-svelte/utilities';
    import { useDraggable } from '@dnd-kit-svelte/core';

    const { id } = $props();
    const { transform, listeners, attributes, node, isDragging } = useDraggable({ id });
    const style = $derived(
        transform.current && !isDragging ? `transform: ${CSS.Translate.toString(transform.current)}` : '',
    );
</script>

<div {style} bind:this={node.current} {...attributes.current} {...listeners.current}>
    <ActionButton action={id} />
</div>
