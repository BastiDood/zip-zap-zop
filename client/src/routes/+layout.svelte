<script>
    import './app.css';
    import { onMount } from 'svelte';

    const { children } = $props();

    onMount(() => {
        // https://github.com/msgpack/msgpack-javascript/issues/260
        if (Object.hasOwn(Symbol, 'dispose')) return;
        Object.defineProperty(Symbol, 'dispose', {
            value: Symbol('zzz:dispose'),
            configurable: false,
            writable: false,
            enumerable: false,
        });
    });
</script>

{@render children()}
