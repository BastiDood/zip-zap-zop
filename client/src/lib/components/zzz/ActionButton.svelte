<script lang="ts">
    import { cn } from '$lib/utils/cn';

    interface Props {
        class?: string;
        action: string;
        disabled: boolean;
    }
    const { class: className, action, disabled }: Props = $props();

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
</script>

<button {disabled} class={cn(className, 'btn btn-circle btn-lg z-10 ring-offset-neutral', buttonColorClasses(action))}>
    {action}
</button>
