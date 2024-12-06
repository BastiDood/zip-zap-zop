<script lang="ts">
    import Deadline from './Deadline.svelte';
    import { PlayerAction } from '$lib/models/game';
    import type { State } from '$lib/zzz/state.svelte';

    interface Props {
        zzz: State;
    }

    const { zzz }: Props = $props();

    let mousePosition = $state({ x: 0, y: 0 });
    let dragStartPos = $state({ x: 0, y: 0 });
    let nextAction: PlayerAction | null = $state(null);
    let hoveredPlayer: number | null = $state(null);

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

    function playerActionColorClasses(action: PlayerAction) {
        switch (action) {
            case PlayerAction.Zip:
                return ['text-info', 'fill-info'] as const;
            case PlayerAction.Zap:
                return ['text-success', 'fill-success'] as const;
            case PlayerAction.Zop:
                return ['text-warning', 'fill-warning'] as const;
        }
    }

    function positionLineStart(event: PointerEvent) {
        mousePosition = { x: event.clientX, y: event.clientY };
        const clickTarget = event.currentTarget;
        if (clickTarget instanceof HTMLButtonElement) {
            const btnBounds = clickTarget.getBoundingClientRect();
            dragStartPos = {
                x: btnBounds.left + btnBounds.width / 2,
                y: btnBounds.top + btnBounds.height / 2,
            };
        }
    }

    function handleDrop(pid: number) {
        if (nextAction !== null) {
            zzz.respond(pid, nextAction);
        }
        hoveredPlayer = null;
        nextAction = null;
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
        {#if nextAction !== null && !disabled}
            {@const [lineColor] = playerActionColorClasses(nextAction)}
            <div>
                <svg class="pointer-events-none absolute left-0 top-0 h-full w-full stroke-2 {lineColor}">
                    <line
                        x1={dragStartPos.x}
                        y1={dragStartPos.y}
                        x2={mousePosition.x}
                        y2={mousePosition.y}
                        stroke="currentColor"
                    />
                </svg>
            </div>
        {/if}
        <div class="flex touch-none select-none flex-row justify-center gap-2">
            <button
                type="button"
                draggable="true"
                {disabled}
                onpointerdown={positionLineStart}
                ondragstart={() => (nextAction = PlayerAction.Zip)}
                class="btn btn-circle btn-info btn-lg ring-offset-neutral {nextAction === PlayerAction.Zip
                    ? 'ring ring-info ring-offset-4'
                    : ''}">Zip</button
            >
            <button
                type="button"
                draggable="true"
                {disabled}
                onpointerdown={positionLineStart}
                ondragstart={() => (nextAction = PlayerAction.Zap)}
                class="btn btn-circle btn-success btn-lg ring-offset-neutral {nextAction === PlayerAction.Zap
                    ? 'ring ring-success ring-offset-4'
                    : ''}">Zap</button
            >
            <button
                type="button"
                draggable="true"
                {disabled}
                onpointerdown={positionLineStart}
                ondragstart={() => (nextAction = PlayerAction.Zop)}
                class="btn btn-circle btn-warning btn-lg ring-offset-neutral {nextAction === PlayerAction.Zop
                    ? 'ring ring-warning ring-offset-4'
                    : ''}">Zop</button
            >
        </div>
        <div class="grid grid-cols-3 gap-2 md:gap-4 lg:grid-cols-5">
            {#each zzz.players as [pid, player] (pid)}
                <div
                    role="gridcell"
                    tabindex="0"
                    class="rounded-xl border bg-base-300 px-4 py-2 text-neutral-content {pid === hoveredPlayer
                        ? 'animate-pulse border-neutral-content'
                        : 'border-base-300'}"
                    ondragover={e => {
                        e.preventDefault();
                        hoveredPlayer = pid;
                    }}
                    ondragleave={() => (hoveredPlayer = null)}
                    ondrop={() => handleDrop(pid)}
                >
                    <p class="w-full truncate text-center font-bold">{player}</p>
                </div>
            {/each}
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

<svelte:window
    ondrag={e => (mousePosition = { x: e.clientX, y: e.clientY })}
    ondragend={() => {
        nextAction = null;
    }}
/>
