<script lang="ts">
    import { PlayerAction } from '$lib/models/game';
    import type { State } from '$lib/zzz/state.svelte';

    interface Props {
        zzz: State;
    }

    const { zzz }: Props = $props();

    function prevPlayerAction(action: PlayerAction) {
        switch (action) {
            case PlayerAction.Zip:
                return 'Zop';
            case PlayerAction.Zap:
                return 'Zip';
            case PlayerAction.Zop:
                return 'Zap';
        }
    }
</script>

<h1 class="text-3xl underline">
    {#if zzz.lid !== null}
        <strong>[{zzz.lid}]</strong>
    {/if}
    {#if zzz.lobby !== null}
        <span>{zzz.lobby}</span>
    {/if}
</h1>
{#if zzz.winner === null}
    {#if zzz.expected === null}
        <p>The game hasn't started yet.</p>
        <ul class="empty:hidden">
            {#each zzz.players as [pid, player] (pid)}
                <li><strong>[{pid}]</strong> {player}</li>
            {/each}
        </ul>
    {:else}
        {@const target = zzz.players.get(zzz.expected.pid) ?? zzz.player}
        {#if typeof target !== 'undefined'}
            <p><strong>{prevPlayerAction(zzz.expected.action)}</strong>! What's next, <strong>{target}</strong>?</p>
        {/if}
        {#if zzz.eliminated === null}
            <p>Nobody has been eliminated yet.</p>
        {:else}
            {@const eliminated = zzz.players.get(zzz.eliminated) ?? zzz.player}
            {#if typeof eliminated !== 'undefined'}
                <p><strong>{eliminated}</strong> has been eliminated.</p>
            {/if}
        {/if}
    {/if}
{:else}
    {@const winner = zzz.players.get(zzz.winner) ?? zzz.player}
    {#if typeof winner !== 'undefined'}
        <p>Congratulations to <strong>{winner}</strong>!</p>
    {/if}
{/if}
