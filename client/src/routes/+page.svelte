<script>
    import Button from '$lib/components/Button.svelte';
    import Link from '$lib/components/Link.svelte';
    import { enhance } from '$app/forms';
</script>

<div class="flex flex-col items-center gap-2">
    <h1 class="p-8 text-6xl">Zip Zap Zop!</h1>
    <div class="grid grid-flow-col gap-4">
        <Link href="/create">Create Lobby</Link>
        <Link href="/help">Help</Link>
    </div>
    <form method="get" action="/join" use:enhance>
        <input type="number" name="lid" required placeholder="Lobby Name" />
        <Button type="submit">Join Lobby</Button>
    </form>
</div>
