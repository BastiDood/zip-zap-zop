<script lang="ts">
    import { DndContext, DragOverlay, type DragStartEvent, type UniqueIdentifier } from '@dnd-kit-svelte/core';
    import ActionButton from '$lib/components/zzz/ActionButton.svelte';
    import Draggable from '$lib/components/zzz/Draggable.svelte';
    import Droppable from '$lib/components/zzz/Droppable.svelte';
    import { assert } from '$lib/utils/assert';

    const players: [number, string][] = [
        [0, 'Player A'],
        [1, 'Player B'],
        [2, 'Player C'],
    ];
    let draggedButton = $state<UniqueIdentifier | null>(null);

    function handleDragStart({ active }: DragStartEvent) {
        assert(typeof active.id === 'string');
        draggedButton = active.id;
    }
</script>

<div class="flex flex-col items-center justify-center p-12">
    <div class="prose max-w-full">
        <h2>Help</h2>
        <p>
            Zip Zap Zop is a very simple game. During each turn, a player has three ways to respond: <strong>Zip</strong
            >,
            <strong>Zap</strong>, or <strong>Zop</strong> (in that order specifically!).
        </p>
        <ul>
            <li>Player A says <strong>Zip</strong> and points to Player C.</li>
            <li>Player C says <strong>Zap</strong> and points to Player B.</li>
            <li>Player B says <strong>Zop</strong> and points to Player C.</li>
            <li>Player C says <strong>Zip</strong> and points to Player A.</li>
            <li>And the cycle goes so on...</li>
        </ul>
        <p>A player is eliminated if:</p>
        <ol>
            <li>They took too long to respond, or</li>
            <li>They responded in violation of the strict Zip-Zap-Zop order.</li>
        </ol>
        <p>
            After each turn, the deadline decays exponentially (e.g., from 10 seconds to 5 seconds to 2.5 seconds and so
            on).
        </p>
        <p>The last player standing wins the game!</p>

        <h2>Controls</h2>
        <p>
            To play, drag your intended action (Zip, Zap, or Zop) and drop it on your targeted player. Remember to only
            act on your turn!
        </p>
        <DndContext
            onDragStart={handleDragStart}
            onDragEnd={() => (draggedButton = null)}
            onDragCancel={() => (draggedButton = null)}
        >
            <DragOverlay dropAnimation={null}>
                {#if typeof draggedButton === 'string'}
                    <ActionButton action={draggedButton} disabled={false} />
                {/if}
            </DragOverlay>
            <div class="space-y-4">
                <div class="flex touch-none select-none flex-row justify-center gap-2 p-2">
                    <Draggable id="Zip" disabled={false} />
                    <Draggable id="Zap" disabled={false} />
                    <Draggable id="Zop" disabled={false} />
                </div>
                <div class="flex justify-center gap-4">
                    {#each players as [pid, player]}
                        <Droppable id={pid}>
                            <p class="m-0 text-center">{player}</p>
                        </Droppable>
                    {/each}
                </div>
            </div>
        </DndContext>
        <div class="flex flex-col items-center md:items-start">
            <p>Ready to play?</p>
            <div>
                <a class="btn btn-primary text-primary-content" href="/">Yes!</a>
            </div>
        </div>
    </div>
</div>
