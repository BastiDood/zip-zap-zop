<script lang="ts">
    import Button from '$lib/Button.svelte';

    // const { lobbyName, playerCount }: { lobbyName: string; playerCount: string } = $props();
</script>

<!-- {#snippet row()}
    <tr>
        <td>{lobbyName}</td>
        <td>{playerCount}</td>
    </tr>
{/snippet} -->

<h2 class="text-3xl">Join a lobby</h2>

<table>
    <tbody>
        <tr>
            <td> Lobby 1 </td>
            <td> 5 players </td>
            <td>
                <a href="/lobby/1"><Button content={'Join'} /></a>
            </td>
        </tr>
        <tr>
            <td> Lobby 2 </td>
            <td> 15 players </td>
            <td>
                <a href="/lobby/2"><Button content={'Join'} /></a>
            </td>
        </tr>
        <tr>
            <td> Lobby 3 </td>
            <td> 2 players </td>
            <td>
                <a href="/lobby/3"><Button content={'Join'} /></a>
            </td>
        </tr>
    </tbody>
</table>

<a href="/create"><Button content={'+ Create a lobby'} /></a>
