<script lang="ts">
  import { onMount, tick } from "svelte";
  import { call, on, applyLook, diffMark, esc, CH, type AppInfo, type Model, type Settings } from "./lib/api";
  import Bars from "./lib/Bars.svelte";

  let model = $state<Model>({ recording: false, rows: [], sel: 0 });
  let level = $state(0);
  let shake = $state(-1);
  let platform = $state("mac");
  let root: HTMLDivElement;
  const drafts = new Map<string, string>();
  const finals = new Map<string, string>();

  const waiting = $derived(model.rows.filter((r) => r.state === "wait" || r.state === "listen").length);
  // 路数越多单条越矮，长文本在条目内滚动，避免把后面的候选顶出屏幕
  const lines = $derived(model.rows.length <= 2 ? 4 : 3);
  const streaming = new Set<string>();

  function receive(m: Model) {
    if (m.recording && m.rows.every((r) => !r.text || r.state === "skip")) {
      drafts.clear();
      finals.clear();
    }
    for (const r of m.rows) {
      if (r.state === "final") {
        if (!finals.has(r.channel)) finals.set(r.channel, diffMark(drafts.get(r.channel) ?? "", r.text));
      } else {
        drafts.set(r.channel, r.text);
        finals.delete(r.channel);
      }
    }
    model = m;
    report();
  }

  async function report() {
    await tick();
    if (!root) return;
    for (const node of root.querySelectorAll(".text")) {
      if (!(node instanceof HTMLElement)) continue;
      const ch = node.dataset.ch ?? "";
      if (node.classList.contains("live")) {
        node.scrollTop = node.scrollHeight;
        streaming.add(ch);
      } else if (streaming.delete(ch)) node.scrollTop = 0;
    }
    // 窗口被工作区卡住时，面板可见高度小于内容高度，用列表 scrollHeight 量自然高度
    const list = root.querySelector(".rows");
    const box = root.getBoundingClientRect().height;
    const natural = list instanceof HTMLElement ? box - list.getBoundingClientRect().height + list.scrollHeight : box;
    // 6px 上下边距，再留 8px 给缩放取整
    call("pick_resize", { height: Math.ceil(natural) + 20 });
  }

  onMount(() => {
    call<AppInfo>("get_state").then((i) => {
      platform = i.platform;
      applyLook(i.platform, i.settings.theme);
    });
    // 面板仅在安全输入开启时取得焦点，此时由这里接收选择按键
    const onKey = (e: KeyboardEvent) => {
      if (e.repeat || e.metaKey || e.ctrlKey || e.altKey) return;
      e.preventDefault();
      call("pick_key", { key: e.key });
    };
    window.addEventListener("keydown", onKey);
    const un = [
      on<Model>("pick", receive),
      on<number>("level", (v) => (level = v)),
      on<Settings>("settings", (s) => applyLook(platform, s.theme)),
      on<number>("pick-shake", (i) => {
        shake = -1;
        tick().then(() => (shake = i));
      }),
    ];
    return () => {
      window.removeEventListener("keydown", onKey);
      un.forEach((p) => p.then((f) => f()));
    };
  });
</script>

<!-- 紧凑面板：无标题栏/页脚/图标。选中行左侧是音量条（录音中）→ 转圈（定稿中）→ 序号（完成）；
     其余行只显示一行、变淡。快捷键照旧：1-N 选择、↑↓ 切换、↵ 上屏、Esc 取消。 -->
<div class="pick" bind:this={root} style:--lines={lines}>
  {#each model.rows as r, i (r.channel)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="row"
      class:active={i === model.sel}
      class:shake={i === shake}
      title="{CH[r.channel].name}{r.ms != null ? ` · ${(r.ms / 1000).toFixed(2)}s` : ''} · {i + 1} 选择 · ↵ 上屏 · Esc 取消"
      onclick={() => call("pick_choose", { index: i })}
    >
      <span class="lead">
        {#if model.recording && i === model.sel}<Bars {level} size={14} />
        {:else if r.state === "wait" || (r.state === "listen" && !model.recording)}<span class="spinner"></span>
        {:else}<span class="num">{i + 1}</span>{/if}
      </span>
      <div
        class="text"
        class:live={r.state === "listen" || r.state === "wait"}
        class:muted={r.state === "skip" || r.state === "error" || !r.text}
        class:bad={r.state === "error"}
        data-ch={r.channel}
      >
        {#if r.state === "final"}
          {@html finals.get(r.channel) ?? esc(r.text)}
        {:else if r.text}
          {r.text}{#if r.state === "listen" || r.state === "wait"}<span class="cur"></span>{/if}
        {:else if model.recording && i === model.sel}<span class="hint">请说话…</span>
        {/if}
      </div>
      <span class="ch">{#if !model.recording && i === model.sel && !waiting}↵ {/if}{CH[r.channel].short}</span>
    </div>
  {/each}
</div>

<style>
  .pick {
    margin: 6px; width: calc(100% - 12px); max-height: calc(100vh - 12px); overflow-y: auto; scrollbar-width: none;
    border-radius: 12px; background: var(--pop); backdrop-filter: blur(36px) saturate(1.8);
    -webkit-backdrop-filter: blur(36px) saturate(1.8); box-shadow: 0 4px 16px rgba(20, 20, 50, 0.16); border: 1px solid var(--line);
    padding: 3px;
  }
  :global(.win) .pick { border-radius: 10px; }
  .row { display: flex; align-items: flex-start; gap: 8px; padding: 5px 8px; border-radius: 8px; cursor: pointer; }
  .row.active { background: color-mix(in srgb, var(--accent) 12%, transparent); }
  .row.shake { animation: shake 0.3s; }
  .lead { width: 18px; height: 21px; flex: none; display: grid; place-items: center; }
  .num { font-size: 10.5px; color: var(--fg3); font-variant-numeric: tabular-nums; }
  .row.active .num { color: var(--accent); font-weight: 700; }
  .lead :global(.spinner) { width: 10px; height: 10px; border-width: 1.5px; }
  .text {
    flex: 1; min-width: 0; font-size: 13.5px; line-height: 1.55; max-height: calc(1.55em * var(--lines, 4));
    overflow-y: auto; scrollbar-width: none; overflow-wrap: anywhere; -webkit-user-select: text; user-select: text;
  }
  .row:not(.active) .text { max-height: 1.55em; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; opacity: 0.55; }
  .text.muted { color: var(--fg3); }
  .text.bad { color: var(--err); }
  .hint { color: var(--fg3); }
  .ch { flex: none; font-size: 10.5px; color: var(--fg3); margin-top: 3px; white-space: nowrap; }
</style>
