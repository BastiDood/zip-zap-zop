<script lang="ts">
    import GitHub from '@iconify/icons-simple-icons/github';
    import Icon from '@iconify/svelte';
</script>

<header class="absolute right-0 top-0 p-4">
    <a href="https://github.com/BastiDood/zip-zap-zop" rel="external" target="_blank">
        <Icon icon={GitHub} width="40" />
    </a>
</header>
<div class="flex h-screen flex-col items-center justify-center">
    <h1 class="p-8 text-5xl md:text-6xl">Zip Zap Zop!</h1>
    <div class="grid grid-flow-col gap-4">
        <a class="btn btn-primary" href="/create/">Create Lobby</a>
        <a class="btn btn-secondary" href="/join/">Join Lobby</a>
        <a class="btn btn-outline" href="/help/">Help</a>
    </div>
</div>
