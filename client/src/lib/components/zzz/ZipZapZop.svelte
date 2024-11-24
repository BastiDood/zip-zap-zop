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
                return PlayerAction.Zop;
            case PlayerAction.Zap:
                return PlayerAction.Zip;
            case PlayerAction.Zop:
                return PlayerAction.Zap;
        }
    }

    function alertClasses(action: PlayerAction) {
        switch (action) {
            case PlayerAction.Zip:
                return ['alert-info', 'text-info-content'] as const;
            case PlayerAction.Zap:
                return ['alert-success', 'text-success-content'] as const;
            case PlayerAction.Zop:
                return ['alert-warning', 'text-warning-content'] as const;
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
            {@const prev = prevPlayerAction(zzz.expected.action)}
            {@const [alert, text] = alertClasses(prev)}
            <div role="alert" class="alert {alert} {text} grid-cols-1 shadow-sm">
                <h1 class="place-self-center text-2xl md:text-3xl">
                    <strong>{prev}</strong>! What's next, <strong>{target}</strong>?
                </h1>
            </div>
        {/if}
        {#key zzz.expected.deadline}
            <Deadline deadline={zzz.expected.deadline} />
        {/key}
        {#if zzz.eliminated === null}
            <div role="alert" class="alert skeleton shadow-sm">Nobody has been eliminated yet.</div>
        {:else}
            <div role="alert" class="alert alert-error text-error-content shadow-sm">
                <span><strong>{zzz.eliminated}</strong> has been eliminated.</span>
            </div>
        {/if}
        <div class="flex justify-center overflow-x-auto">
            <table class="table-auto">
                <thead class="border-b border-slate-700 text-left text-slate-500">
                    <tr class="*:p-2">
                        <th>Actions</th>
                        <th>Target</th>
                    </tr>
                </thead>
                <tbody class="empty:hidden">
                    {#each zzz.players as [pid, player] (pid)}
                        <tr class="border-b border-slate-700 *:p-2">
                            <td>
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zip)}
                                    class="btn btn-info btn-sm">Zip</button
                                >
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zap)}
                                    class="btn btn-success btn-sm">Zap</button
                                >
                                <button
                                    type="button"
                                    {disabled}
                                    onclick={() => zzz.respond(pid, PlayerAction.Zop)}
                                    class="btn btn-warning btn-sm">Zop</button
                                >
                            </td>
                            <td><strong>{player}</strong></td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
{:else}
    {@const winner = zzz.players.get(zzz.winner) ?? zzz.player}
    <div class="flex flex-col items-center gap-12">
        {#if winner !== null}
            <div class="flex flex-col gap-4 text-center">
                <h1 class="text-4xl md:text-5xl">Game over!</h1>
                <h2 class="text-2xl md:text-3xl">Congratulations to:</h2>
                <div role="alert" class="alert alert-success grid-cols-1 shadow-sm">
                    <span class="place-self-center"><strong>{winner}</strong>!</span>
                </div>
            </div>
        {/if}
        <a href="/" class="btn btn-primary">Go Back Home</a>
    </div>
{/if}
