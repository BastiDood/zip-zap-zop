<script>
    import Button from '$lib/Button.svelte';
</script>

<div class="flex flex-col items-center">
    <h1 class="p-8 text-6xl">Zip Zap Zop!</h1>
    <div class="grid grid-flow-col gap-4">
        <a href="/lobbies"><Button content={'Play'} /></a>
        <a href="/help"><Button content={'Help'} /></a>
    </div>
</div>
