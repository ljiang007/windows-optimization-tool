<script setup>
import { computed, ref } from 'vue'

const parts = ref([
  { key: 'cpu', label: 'CPU', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'motherboard', label: '主板', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'memory', label: '内存条', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'ssd', label: '固态硬盘', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'gpu', label: '显卡', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'cooler', label: '散热', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'psu', label: '电源', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'case', label: '机箱', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'fan', label: '风扇', model: '', taobao: '', douyin: '', xianyu: '' },
  { key: 'monitor', label: '显示器', model: '', taobao: '', douyin: '', xianyu: '' },
])

const platformFields = [
  { key: 'taobao', label: '淘宝' },
  { key: 'douyin', label: '抖音' },
  { key: 'xianyu', label: '咸鱼' },
]

function parsePrice(value) {
  if (value === '' || value == null) return null

  const normalized = String(value).replace(/[^\d.]/g, '')
  const price = Number.parseFloat(normalized)
  return Number.isFinite(price) ? price : null
}

function formatPrice(value) {
  return value == null ? '--' : `¥${value.toFixed(2)}`
}

function getRowMetrics(part) {
  const offers = platformFields
    .map((platform) => ({
      platform: platform.label,
      value: parsePrice(part[platform.key]),
    }))
    .filter((offer) => offer.value != null)

  const bestOffer = offers.reduce((best, current) => {
    if (!best || current.value < best.value) return current
    return best
  }, null)

  const highestOffer = offers.reduce((highest, current) => {
    if (!highest || current.value > highest.value) return current
    return highest
  }, null)

  return {
    bestPrice: bestOffer?.value ?? null,
    bestPlatform: bestOffer?.platform ?? '待补价',
    spread:
      bestOffer && highestOffer && highestOffer.value > bestOffer.value
        ? highestOffer.value - bestOffer.value
        : null,
  }
}

const rows = computed(() =>
  parts.value.map((part) => {
    const metrics = getRowMetrics(part)

    return {
      key: part.key,
      label: part.label,
      part,
      ...metrics,
    }
  }),
)

const platformTotals = computed(() =>
  platformFields.map((platform) => {
    const prices = parts.value
      .map((part) => parsePrice(part[platform.key]))
      .filter((value) => value != null)

    return {
      label: platform.label,
      count: prices.length,
      total: prices.reduce((sum, value) => sum + value, 0),
    }
  }),
)

const totalPrice = computed(() =>
  rows.value.reduce((sum, row) => sum + (row.bestPrice ?? 0), 0),
)

const pricedCount = computed(() =>
  rows.value.filter((row) => row.bestPrice != null).length,
)

const missingParts = computed(() =>
  rows.value
    .filter((row) => !row.part.model.trim() || row.bestPrice == null)
    .map((row) => row.label),
)

const widestGap = computed(() =>
  rows.value
    .filter((row) => row.spread != null)
    .sort((left, right) => right.spread - left.spread)[0] ?? null,
)
</script>

<template>
  <main class="pc-diy-shell">
    <section class="pc-diy-window">
      <header class="worktop-bar">
        <div class="title-block">
          <p class="eyebrow">DIY PRICE DESK</p>
          <h1>电脑DIY比价台</h1>
        </div>
        <div class="platform-badges" aria-label="支持平台">
          <span v-for="platform in platformFields" :key="platform.key">{{ platform.label }}</span>
        </div>
      </header>

      <section class="summary-strip" aria-label="整机概览">
        <article class="stat-card total-card">
          <span class="stat-label">总价</span>
          <strong class="stat-value">{{ formatPrice(totalPrice) }}</strong>
        </article>
        <article class="stat-card">
          <span class="stat-label">已录配件</span>
          <strong class="stat-value">{{ pricedCount }}/{{ rows.length }}</strong>
        </article>
        <article class="stat-card">
          <span class="stat-label">待补配件</span>
          <strong class="stat-value">{{ missingParts.length }}</strong>
        </article>
        <article class="stat-card" v-for="platform in platformTotals" :key="platform.label">
          <span class="stat-label">{{ platform.label }}</span>
          <strong class="stat-value">{{ formatPrice(platform.total) }}</strong>
        </article>
        <article class="stat-card" :class="{ muted: !widestGap }">
          <span class="stat-label">最大差价</span>
          <strong class="stat-value">{{ widestGap ? formatPrice(widestGap.spread) : '--' }}</strong>
        </article>
        <article class="stat-card alert-card" :class="{ ready: missingParts.length === 0 }">
          <span class="stat-label">状态</span>
          <strong class="stat-value">{{ missingParts.length ? '待补价' : '已齐全' }}</strong>
        </article>
      </section>

      <section class="board-panel">
        <div class="table-shell">
          <table class="price-table">
            <thead>
              <tr>
                <th>配件</th>
                <th>型号 / 搜索关键词</th>
                <th>淘宝</th>
                <th>抖音</th>
                <th>咸鱼</th>
                <th>最低价</th>
                <th>来源</th>
                <th>差价</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in rows" :key="row.key">
                <td class="part-cell">
                  <strong>{{ row.label }}</strong>
                </td>
                <td>
                  <input
                    v-model.trim="row.part.model"
                    class="model-input"
                    type="text"
                    :placeholder="`输入${row.label}型号或搜索词`"
                  >
                </td>
                <td v-for="platform in platformFields" :key="`${row.key}-${platform.key}`">
                  <input
                    v-model.trim="row.part[platform.key]"
                    class="price-input"
                    type="text"
                    inputmode="decimal"
                    :placeholder="`${platform.label}价格`"
                  >
                </td>
                <td class="price-highlight">{{ formatPrice(row.bestPrice) }}</td>
                <td>
                  <span class="source-chip" :class="{ empty: row.bestPrice == null }">{{ row.bestPlatform }}</span>
                </td>
                <td class="spread-cell">{{ formatPrice(row.spread) }}</td>
              </tr>
            </tbody>
            <tfoot>
              <tr>
                <td colspan="5">总价</td>
                <td class="price-highlight">{{ formatPrice(totalPrice) }}</td>
                <td>最低价累计</td>
                <td>{{ widestGap ? formatPrice(widestGap.spread) : '--' }}</td>
              </tr>
            </tfoot>
          </table>
        </div>
      </section>
    </section>
  </main>
</template>

<style scoped>
.pc-diy-shell {
  height: 100vh;
  padding: 10px;
  overflow: hidden;
  background: #dfe4ea;
}

.pc-diy-window {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 8px;
  width: 100%;
  height: 100%;
  padding: 10px;
  border: 1px solid rgba(28, 40, 58, 0.16);
  border-radius: 18px;
  background: linear-gradient(180deg, #f8fafc 0%, #eef3f7 100%);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.92);
}

.worktop-bar {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: space-between;
  min-height: 64px;
  padding: 10px 14px;
  border: 1px solid rgba(27, 39, 57, 0.08);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.9);
}

.title-block {
  min-width: 0;
}

.eyebrow {
  margin: 0 0 4px;
  color: #b3472e;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.22em;
}

h1 {
  margin: 0;
  color: #172231;
  font-family: "Bahnschrift", "Segoe UI Semibold", "Microsoft YaHei UI", sans-serif;
  font-size: 24px;
  line-height: 1;
}

.platform-badges {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
  min-width: 0;
}

.platform-badges span,
.source-chip {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 28px;
  padding: 0 12px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.04em;
}

.platform-badges span {
  border: 1px solid rgba(28, 43, 68, 0.12);
  color: #1a2533;
  background: #f5f7fb;
}

.summary-strip {
  display: grid;
  grid-template-columns: 1.2fr repeat(6, minmax(0, 1fr));
  flex-wrap: wrap;
  gap: 8px;
}

.stat-card,
.board-panel {
  border: 1px solid rgba(25, 35, 52, 0.1);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.92);
}

.stat-card {
  display: flex;
  gap: 4px;
  align-items: flex-start;
  justify-content: center;
  flex-direction: column;
  min-width: 0;
  padding: 8px 12px;
}

.stat-label {
  color: #6a7786;
  font-size: 11px;
  font-weight: 700;
}

.stat-value {
  color: #15202d;
  font-size: 18px;
  font-weight: 800;
  line-height: 1.1;
  white-space: nowrap;
}

.total-card {
  background: linear-gradient(145deg, #e85e38 0%, #ce3e32 100%);
}

.total-card .stat-label,
.total-card .stat-value {
  color: #fff8f4;
}

.alert-card {
  background: rgba(255, 228, 195, 0.72);
}

.alert-card.ready {
  background: rgba(197, 244, 215, 0.82);
}

.stat-card.muted .stat-value {
  color: #6f7a88;
}

.table-shell {
  height: 100%;
  overflow: hidden;
  border: 1px solid rgba(27, 37, 54, 0.08);
  border-radius: 14px;
}

.price-table {
  width: 100%;
  height: 100%;
  border-collapse: collapse;
  table-layout: fixed;
  color: #15202d;
}

.price-table thead th {
  padding: 10px 8px;
  border-bottom: 1px solid rgba(21, 32, 45, 0.08);
  background: #edf1f5;
  text-align: left;
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.05em;
}

.price-table th:nth-child(1),
.price-table td:nth-child(1) {
  width: 9%;
}

.price-table th:nth-child(2),
.price-table td:nth-child(2) {
  width: 31%;
}

.price-table th:nth-child(3),
.price-table td:nth-child(3),
.price-table th:nth-child(4),
.price-table td:nth-child(4),
.price-table th:nth-child(5),
.price-table td:nth-child(5) {
  width: 10%;
}

.price-table th:nth-child(6),
.price-table td:nth-child(6) {
  width: 11%;
}

.price-table th:nth-child(7),
.price-table td:nth-child(7),
.price-table th:nth-child(8),
.price-table td:nth-child(8) {
  width: 10%;
}

.price-table td {
  padding: 6px 8px;
  border-bottom: 1px solid rgba(21, 32, 45, 0.06);
  background: rgba(255, 255, 255, 0.82);
  vertical-align: middle;
}

.price-table tbody tr:hover td {
  background: rgba(245, 248, 251, 0.98);
}

.price-table tfoot td {
  padding: 10px 8px;
  border-top: 1px solid rgba(21, 32, 45, 0.08);
  background: #eef3f8;
  font-weight: 800;
}

.part-cell strong {
  font-size: 13px;
}

.model-input,
.price-input {
  width: 100%;
  min-height: 30px;
  padding: 0 8px;
  border: 1px solid rgba(33, 49, 75, 0.12);
  border-radius: 8px;
  color: #132031;
  background: rgba(255, 255, 255, 0.96);
  outline: none;
  font-size: 12px;
  transition: border-color 140ms ease, box-shadow 140ms ease, transform 140ms ease;
}

.model-input:focus,
.price-input:focus {
  border-color: rgba(255, 110, 72, 0.8);
  box-shadow: 0 0 0 3px rgba(255, 110, 72, 0.12);
}

.price-highlight {
  color: #ce4d2b;
  font-size: 14px;
  font-weight: 800;
}

.source-chip {
  color: #0f3b68;
  background: rgba(176, 217, 255, 0.6);
}

.source-chip.empty {
  color: #6d7785;
  background: rgba(220, 226, 235, 0.8);
}

.spread-cell {
  color: #5f6c7a;
  font-size: 12px;
  font-weight: 700;
}

@media (max-width: 1180px) {
  .summary-strip {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

@media (max-width: 900px) {
  .pc-diy-shell {
    padding: 6px;
  }

  .pc-diy-window {
    padding: 8px;
  }

  h1 {
    font-size: 20px;
  }
}
</style>
