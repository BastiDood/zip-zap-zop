<script lang="ts">
    import { cn } from '$lib/utils/cn';

    function buttonColorClasses(action: string) {
        switch (action) {
            case 'Zip':
                return 'btn-info';
            case 'Zap':
                return 'btn-success';
            case 'Zop':
                return 'btn-warning';
        }
    }

    const { class: externalClass = '', action, disabled } = $props();
</script>

<button
    {disabled}
    class={cn(externalClass, 'btn btn-circle btn-lg z-10 ring-offset-neutral', buttonColorClasses(action))}
>
    {action}
</button>
