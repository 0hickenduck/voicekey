<script lang="ts">
  import { onMount } from "svelte";
  // 五根竖条的小音量计，替代占地方的横向波形：说话时随音量起伏，静音时轻微呼吸
  let { level = 0, idle = false, size = 16 }: { level?: number; idle?: boolean; size?: number } = $props();
  const N = 5;
  const weight = [0.55, 0.85, 1, 0.8, 0.5];
  let bars: HTMLSpanElement[] = $state([]);
  let shown = 0;

  onMount(() => {
    let raf = 0;
    const draw = () => {
      shown = level > shown ? shown * 0.3 + level * 0.7 : shown * 0.85 + level * 0.15;
      const t = performance.now() / 1000;
      for (let i = 0; i < N; i++) {
        const wob = 0.65 + 0.35 * Math.sin(t * (5 + i * 1.7) + i * 1.3);
        const v = idle ? 0.22 + 0.12 * (Math.sin(t * 2.4 + i * 0.7) * 0.5 + 0.5) : 0.18 + 0.82 * shown * weight[i] * wob;
        if (bars[i]) bars[i].style.transform = `scaleY(${Math.min(1, v).toFixed(3)})`;
      }
      raf = requestAnimationFrame(draw);
    };
    draw();
    return () => cancelAnimationFrame(raf);
  });
</script>

<span class="bars" style:height="{size}px" aria-hidden="true">
  {#each Array(N) as _, i}<span bind:this={bars[i]}></span>{/each}
</span>

<style>
  .bars { display: inline-flex; align-items: center; gap: 2px; flex: none; }
  .bars span { width: 2.5px; height: 100%; border-radius: 2px; background: var(--accent); transform: scaleY(0.2); transition: transform 40ms linear; }
</style>
