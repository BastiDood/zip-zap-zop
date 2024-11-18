<script>
    import Link from '$lib/components/Link.svelte';
</script>

<div class="flex flex-col items-center gap-2">
    <h1 class="p-8 text-6xl">Zip Zap Zop!</h1>
    <div class="grid grid-flow-col gap-4">
        <Link href="/create/">Create Lobby</Link>
        <Link href="/join/">Join Lobby</Link>
        <Link href="/help/">Help</Link>
    </div>
</div>
