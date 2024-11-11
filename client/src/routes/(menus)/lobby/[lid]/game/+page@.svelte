<script>
    const players = ['a', 'b', 'c', 'd', 'e'];
</script>

<div class="flex h-1/2 w-2/3 flex-col items-center justify-between gap-4">
    <div class="grid h-full w-full grid-flow-col">
        {#each players as player}
            <div class="text-center text-4xl hover:bg-slate-300">{player}</div>
        {/each}
    </div>
    <div>
        <button class="rounded-2xl border-2 p-4 text-xl">Zip</button>
        <button class="rounded-2xl border-2 p-4 text-xl">Zap</button>
        <button class="rounded-2xl border-2 p-4 text-xl">Zop</button>
    </div>
</div>
