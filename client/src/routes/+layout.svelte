<script>
    import './app.css';

    const { children } = $props();
</script>

<div class="flex h-screen items-center justify-center p-12">
    {@render children()}
</div>
