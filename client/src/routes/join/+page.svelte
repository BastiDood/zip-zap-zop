<script lang="ts">
    import Button from '$lib/components/Button.svelte';
    import { State } from '$lib/zzz/state.svelte';
    import { validateString } from '$lib/utils/validate';

    let state = $state<State | null>(null);

    function joinLobby(form: HTMLFormElement) {
        const data = new FormData(form);
        const lid = Number.parseInt(validateString(data.get('lid')), 10);
        const player = validateString(data.get('player'));
        state = State.guest(lid, player);
    }
</script>

{#if state === null}
    <form
        onsubmit={event => {
            event.preventDefault();
            event.stopPropagation();
            joinLobby(event.currentTarget);
        }}
    >
        <input type="number" required name="lid" placeholder="Lobby ID" />
        <input type="text" required name="player" placeholder="Player Name" />
        <Button type="submit">Join Lobby</Button>
    </form>
{:else}
    <p>Good</p>
{/if}
