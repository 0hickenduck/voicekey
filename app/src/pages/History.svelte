<script lang="ts">
  import { app, save } from "../lib/store.svelte";
  import { call, on, CH, type HistoryEntry } from "../lib/api";
  import ChIcon from "../lib/ChIcon.svelte";

  const s = $derived(app.s!);
  const LANG = { zh: "中文", ja: "日语", en: "英语", "": "" };
  let q = $state("");
  let list = $state<HistoryEntry[]>([]);
  let dir = $state("");
  let copied = $state(-1);
  let rev = 0;

  async function load() {
    const r = ++rev;
    const res = await call<{ dir: string; entries: HistoryEntry[] }>("history_list", { query: q });
    if (r !== rev) return;
    dir = res.dir;
    list = res.entries;
  }

  $effect(() => {
    q;
    const t = setTimeout(load, 150);
    return () => clearTimeout(t);
  });

  $effect(() => {
    const un = on<HistoryEntry>("history", () => load());
    return () => void un.then((f) => f());
  });

  async function copy(e: HistoryEntry, i: number) {
    try {
      await navigator.clipboard.writeText(e.text);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = e.text;
      document.body.append(ta);
      ta.select();
      document.execCommand("copy");
      ta.remove();
    }
    copied = i;
    setTimeout(() => copied === i && (copied = -1), 1200);
  }

  function fmt(ts: number) {
    const d = new Date(ts), now = new Date();
    const hm = d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
    if (d.toDateString() === now.toDateString()) return hm;
    return `${d.getFullYear() === now.getFullYear() ? "" : d.getFullYear() + "年"}${d.getMonth() + 1}月${d.getDate()}日 ${hm}`;
  }
</script>

<h2>历史</h2>
<div class="sub">上屏过的文字保存在本机，可搜索，点一条即复制。</div>
<div class="group">
  <div class="row">
    <div class="lbl">保存识别历史<small>{dir ? `${dir} 下的 history.jsonl，一行一条` : "一行一条，保存在数据目录"}</small></div>
    <button class="btn" disabled={!dir} onclick={() => call("open_url", { url: dir })}>打开文件夹</button>
    <button class="sw" class:on={s.history} onclick={() => save({ history: !s.history })} aria-label="保存识别历史"></button>
  </div>
</div>
<h3>记录</h3>
<input class="search" placeholder="搜索文字" bind:value={q} />
{#if list.length}
  <div class="group">
    {#each list as e, i}
      <button class="row item" onclick={() => copy(e, i)} title="点击复制">
        <ChIcon ch={e.channel} size="xs" />
        <span class="lbl">
          <span class="txt">{e.text}</span>
          <small>{fmt(e.ts)} · {CH[e.channel]?.short ?? e.channel}{e.mix ? ` · ${e.mix.split("+").map((l) => (LANG as Record<string, string>)[l] ?? l).join("+")}` : e.lang ? ` · ${LANG[e.lang]}` : ""}{e.alts?.length ? ` · 另有 ${e.alts.length} 条候选` : ""}</small>
        </span>
        <span class="hint" class:ok={copied === i}>{copied === i ? "已复制" : "复制"}</span>
      </button>
    {/each}
  </div>
  {#if list.length >= 500}<div class="note">只显示最近 500 条，更早的可搜索或打开文件查看。</div>{/if}
{:else}
  <div class="note">{q ? "没有匹配的记录" : s.history ? "还没有记录，说一句话后会出现在这里。" : "历史已关闭。"}</div>
{/if}

<style>
  .search { width: 100%; border: 1px solid var(--line); background: var(--card); border-radius: var(--rs); padding: 7px 12px; margin-bottom: 10px; outline: none; }
  .search:focus { border-color: var(--accent); }
  .item { width: 100%; border: 0; background: none; text-align: left; cursor: pointer; align-items: flex-start; }
  .item + .item { border-top: 1px solid var(--line); }
  .item:hover { background: var(--card2); }
  .item :global(.ic) { margin-top: 2px; }
  .txt { display: block; white-space: pre-wrap; word-break: break-word; }
  .hint { font-size: 11px; color: var(--fg3); flex: none; visibility: hidden; }
  .item:hover .hint, .hint.ok { visibility: visible; }
</style>
