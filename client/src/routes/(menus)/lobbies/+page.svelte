<script lang="ts">
    import Link from '$lib/components/Link.svelte';
</script>

<h2 class="text-3xl">Join a lobby</h2>
<table>
    <tbody>
        <tr>
            <td> Lobby 1 </td>
            <td> 5 players </td>
            <td>
                <Link href="/lobby/1">Join</Link>
            </td>
        </tr>
        <tr>
            <td> Lobby 2 </td>
            <td> 15 players </td>
            <td>
                <Link href="/lobby/2">Join</Link>
            </td>
        </tr>
        <tr>
            <td> Lobby 3 </td>
            <td> 2 players </td>
            <td>
                <Link href="/lobby/3">Join</Link>
            </td>
        </tr>
    </tbody>
</table>
<Link href="/create">Create Lobby</Link>
