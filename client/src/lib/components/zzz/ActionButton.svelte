<script lang="ts">
    import { cn } from '$lib/utils/cn';

    const { class: externalClass = '', action } = $props();
</script>

<div
    class={cn(externalClass, 'btn btn-circle btn-lg z-10 ring-offset-neutral', {
        'btn-info': action === 'Zip',
        'btn-success': action === 'Zap',
        'btn-warning': action === 'Zop',
    })}
>
    {action}
</div>
