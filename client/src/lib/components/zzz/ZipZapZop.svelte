<script lang="ts">
    import Deadline from './Deadline.svelte';
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
        {@const disabled = zzz.pid === null}
        {@const target = zzz.players.get(zzz.expected.next) ?? zzz.player}
        {#if target !== null}
            <p><strong>{prevPlayerAction(zzz.expected.action)}</strong>! What's next, <strong>{target}</strong>?</p>
        {/if}
        {#if zzz.eliminated === null}
            <p>Nobody has been eliminated yet.</p>
        {:else}
            <p><strong>{zzz.eliminated}</strong> has been eliminated.</p>
        {/if}
        {#key zzz.expected.deadline}
            <Deadline deadline={zzz.expected.deadline} />
        {/key}
        <table>
            <thead>
                <tr>
                    <th>Name</th>
                    <th>Controls</th>
                </tr>
            </thead>
            <tbody>
                {#each zzz.players as [pid, player] (pid)}
                    <tr>
                        <td><strong>{player}</strong></td>
                        <td>
                            <button type="button" {disabled} onclick={() => zzz.respond(pid, PlayerAction.Zip)}
                                >Zip</button
                            >
                            <button type="button" {disabled} onclick={() => zzz.respond(pid, PlayerAction.Zap)}
                                >Zap</button
                            >
                            <button type="button" {disabled} onclick={() => zzz.respond(pid, PlayerAction.Zop)}
                                >Zop</button
                            >
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
{:else}
    {@const winner = zzz.players.get(zzz.winner) ?? zzz.player}
    {#if winner !== null}
        <p>Congratulations to <strong>{winner}</strong>!</p>
    {/if}
{/if}
