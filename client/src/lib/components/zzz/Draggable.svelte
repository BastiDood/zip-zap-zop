<script lang="ts">
    import { CSS } from '@dnd-kit-svelte/utilities';
    import { useDraggable } from '@dnd-kit-svelte/core';

    const { id } = $props();
    const { transform, listeners, attributes, node } = useDraggable({ id });
    const style = $derived(transform.current ? `transform: ${CSS.Translate.toString(transform.current)}` : '');

    function buttonColorClasses(id: string) {
        switch (id) {
            case 'Zip':
                return 'btn-info';
            case 'Zap':
                return 'btn-success';
            case 'Zop':
                return 'btn-warning';
        }
    }
</script>

<div
    class="btn btn-circle btn-lg z-10 ring-offset-neutral {buttonColorClasses(id)}"
    {style}
    bind:this={node.current}
    {...attributes.current}
    {...listeners.current}
>
    {id}
</div>
