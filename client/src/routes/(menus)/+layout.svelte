<script>
    const { children } = $props();
</script>

<div class="flex h-full w-full flex-col gap-8 sm:px-4 md:px-20">
    <div class="flex flex-col md:flex-row md:items-center">
        <a href="/"><button class="text-lg underline underline-offset-2">Back</button></a>
        <h1 class="w-full text-center text-4xl">Zip Zap Zop</h1>
    </div>
    <div class="flex flex-col gap-4">
        {@render children()}
    </div>
</div>
