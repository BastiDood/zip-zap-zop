<script lang="ts">
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
        class="card m-8 mx-auto max-w-lg shadow-xl"
    >
        <div class="card-body">
            <label class="form-control w-full">
                <div class="label"><span class="label-text">Lobby Name</span></div>
                <input type="text" required name="lobby" placeholder="My Lobby" class="input input-bordered w-full" />
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
            <button type="submit" class="btn btn-primary mt-4">Create Lobby</button>
        </div>
    </form>
{:else}
    <main class="flex max-h-screen flex-col space-y-4 p-4">
        <ZipZapZop {zzz} />
        {#if isPending}
            {@const disabled = zzz.lid === null || zzz.pid === null}
            <div>
                <button type="button" {disabled} onclick={startGame.bind(null, zzz)} class="btn btn-success"
                    >Start Game</button
                >
            </div>
        {/if}
    </main>
{/if}
