<script lang="ts">
    import Button from '$lib/components/Button.svelte';
    import { State } from '$lib/zzz/state.svelte';
    import { validateString } from '$lib/utils/validate';

    let state = $state<State | null>(null);

    function createLobby(form: HTMLFormElement) {
        const data = new FormData(form);
        const lobby = validateString(data.get('lobby'));
        const player = validateString(data.get('player'));
        state = State.host(lobby, player);
    }
</script>

{#if state === null}
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
    <p>Good</p>
{/if}
