<script lang="ts">
    const { action } = $props();

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

<div class="btn btn-circle btn-lg z-10 ring-offset-neutral {buttonColorClasses(action)}">
    {action}
</div>
