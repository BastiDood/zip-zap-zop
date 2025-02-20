<script lang="ts">
    import { State } from '$lib/zzz/state.svelte';
    import ZipZapZop from '$lib/components/zzz/ZipZapZop.svelte';
    import { validateString } from '$lib/utils/validate';

    let zzz = $state<State | null>(null);

    function joinLobby(form: HTMLFormElement) {
        const data = new FormData(form);
        const lid = Number.parseInt(validateString(data.get('lid')), 10);
        const player = validateString(data.get('player'));
        zzz = State.guest(lid, player);
    }
</script>

{#if zzz === null}
    <form
        onsubmit={event => {
            event.preventDefault();
            event.stopPropagation();
            joinLobby(event.currentTarget);
        }}
        class="card m-8 mx-auto max-w-lg shadow-xl"
    >
        <div class="card-body">
            <label class="form-control w-full">
                <div class="label"><span class="label-text">Lobby ID</span></div>
                <input type="number" required min="0" placeholder="0" name="lid" class="input input-bordered w-full" />
            </label>
            <label class="form-control w-full">
                <div class="label"><span class="label-text">Player Name</span></div>
                <input
                    type="text"
                    required
                    name="player"
                    maxlength="16"
                    placeholder="Lino"
                    class="input input-bordered w-full"
                />
            </label>
            <button type="submit" class="btn btn-primary mt-4">Join Lobby</button>
        </div>
    </form>
{:else}
    <main class="flex h-screen flex-col space-y-4 p-4"><ZipZapZop {zzz} /></main>
{/if}
