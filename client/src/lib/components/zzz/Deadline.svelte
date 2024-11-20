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

    let progress = $state<Progress>();
    function tick(value: DOMHighResTimeStamp) {
        // Update the current progress
        if (typeof progress === 'undefined') progress = { min: value, value };
        else progress.value = value;

        // Clamp down and re-render if necessary
        if (end <= progress.value) progress.value = end;
        else requestAnimationFrame(tick);
    }

    $effect(() => {
        requestAnimationFrame(tick);
    });
</script>

{#if typeof progress === 'undefined'}
    <progress class="progress progress-warning"></progress>
{:else}
    {@const max = Math.max(0, end - progress.min)}
    {@const value = progress.value - progress.min}
    <progress {max} {value} class="progress progress-warning"></progress>
{/if}
