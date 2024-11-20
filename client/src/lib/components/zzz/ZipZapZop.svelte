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

<div class="flex overflow-hidden rounded-lg text-3xl shadow-xl">
    {#if zzz.lid !== null}
        <div class="flex-none bg-primary px-4 py-2 text-primary-content">{zzz.lid}</div>
    {/if}
    {#if zzz.lobby !== null}
        <div class="flex-1 bg-neutral px-4 py-2 text-neutral-content">{zzz.lobby}</div>
    {/if}
</div>
{#if zzz.winner === null}
    {#if zzz.expected === null}
        <div role="alert" class="alert skeleton shadow-sm">Waiting for the host to start the game...</div>
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>ID</th>
                        <th>Name</th>
                    </tr>
                </thead>
                <tbody class="empty:hidden">
                    {#each zzz.players as [pid, player] (pid)}
                        <tr>
                            <td>{pid}</td>
                            <td>{player}</td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {:else}
        {@const disabled = zzz.pid === null}
        {@const target = zzz.players.get(zzz.expected.next) ?? zzz.player}
        {#if target !== null}
            <div role="alert" class="alert alert-info text-info-content shadow-sm">
                <span
                    ><strong>{prevPlayerAction(zzz.expected.action)}</strong>! What's next,
                    <strong>{target}</strong>?</span
                >
            </div>
        {/if}
        {#if zzz.eliminated === null}
            <div role="alert" class="alert skeleton shadow-sm">Nobody has been eliminated yet.</div>
        {:else}
            <div role="alert" class="alert alert-error text-error-content shadow-sm">
                <span><strong>{zzz.eliminated}</strong> has been eliminated.</span>
            </div>
        {/if}
        {#key zzz.expected.deadline}
            <Deadline deadline={zzz.expected.deadline} />
        {/key}
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>Name</th>
                        <th>Controls</th>
                    </tr>
                </thead>
                <tbody class="empty:hidden">
                    {#each zzz.players as [pid, player] (pid)}
                        <tr>
                            <td><strong>{player}</strong></td>
                            <td>
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zip)}
                                    class="btn btn-primary btn-xs">Zip</button
                                >
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zap)}
                                    class="btn btn-secondary btn-xs">Zap</button
                                >
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zop)}
                                    class="btn btn-accent btn-xs">Zop</button
                                >
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
{:else}
    {@const winner = zzz.players.get(zzz.winner) ?? zzz.player}
    {#if winner !== null}
        <div role="alert" class="alert alert-success shadow-sm">
            <span>Congratulations to <strong>{winner}</strong>!</span>
        </div>
    {/if}
    <a href="/" class="btn btn-primary">Go Back Home</a>
{/if}
