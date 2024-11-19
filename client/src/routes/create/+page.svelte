<script lang="ts">
    import Button from '$lib/components/ui/Button.svelte';
    import Deadline from '$lib/components/zzz/Deadline.svelte';
    import { State } from '$lib/zzz/state.svelte';
    import ZipZapZop from '$lib/components/zzz/ZipZapZop.svelte';
    import { validateString } from '$lib/utils/validate';

    let zzz = $state<State | null>(null);
    let isPending = $state(true);

    function createLobby(form: HTMLFormElement) {
        const data = new FormData(form);
        const lobby = validateString(data.get('lobby'));
        const player = validateString(data.get('player'));
        zzz = State.host(lobby, player);
    }

    function startGame(zzz: State) {
        isPending = false;
        zzz.start();
    }
</script>

{#if zzz === null}
    <form
        onsubmit={event => {
            event.preventDefault();
            event.stopPropagation();
            createLobby(event.currentTarget);
        }}
    >
        <input type="text" required name="lobby" placeholder="Lobby Name" />
        <input type="text" required name="player" placeholder="Player Name" />
        <Button type="submit">Create Lobby</Button>
    </form>
{:else}
    {#if isPending}
        {@const disabled = zzz.lid === null || zzz.pid === null}
        <Button {disabled} onclick={startGame.bind(null, zzz)}>Start Game</Button>
    {/if}
    <ZipZapZop {zzz} />
{/if}
