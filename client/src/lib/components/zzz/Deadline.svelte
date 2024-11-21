<script lang="ts">
    interface Props {
        deadline: Date;
    }

    const { deadline }: Props = $props();
    const end = $derived(deadline.valueOf() - performance.timeOrigin);

    interface Progress {
        min: DOMHighResTimeStamp;
        value: DOMHighResTimeStamp;
    }

    let id = null as number | null;
    let progress = $state<Progress>();

    function tick(value: DOMHighResTimeStamp) {
        // Update the current progress
        if (typeof progress === 'undefined') progress = { min: value, value };
        else progress.value = value;

        // Clamp down and re-render if necessary
        if (end <= progress.value) progress.value = end;
        else id = requestAnimationFrame(tick);
    }

    $effect(() => {
        id = requestAnimationFrame(tick);
        return () => {
            if (id !== null) cancelAnimationFrame(id);
        };
    });
</script>

{#if typeof progress === 'undefined'}
    <progress class="progress progress-accent"></progress>
{:else}
    {@const max = Math.max(0, end - progress.min)}
    {@const value = progress.value - progress.min}
    <progress {max} {value} class="progress progress-accent"></progress>
{/if}
