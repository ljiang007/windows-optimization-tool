import { ref, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const DEBOUNCE_MS = 800

/**
 * 每行独立的爬取状态
 */
function createRowState() {
  return {
    loading: ref(false),
    error: ref(null),
    timer: null,
  }
}

/**
 * usePriceCrawler
 *
 * 传入 parts（ref 数组），为每一行监听 model 字段变化。
 * 防抖 800ms 后调用 Tauri 后端 crawl_prices 指令，
 * 结果自动回写 part.xianyu。
 *
 * @param {import('vue').Ref<Array>} parts - PcDiyWindowPage 的 parts ref
 * @returns {{ getRowState: (key: string) => { loading: Ref<boolean>, error: Ref<string|null> }, manualCrawl: (part: object) => void }}
 */
export function usePriceCrawler(parts) {
  // key -> { loading, error, timer }
  const stateMap = Object.fromEntries(
    parts.value.map(part => [part.key, createRowState()])
  )

  /**
   * 实际发起爬取，调用 Rust 后端指令 crawl_prices
   * 期望后端返回: { xianyu: number|null }
   */
  async function fetchPrices(part) {
    const state = stateMap[part.key]
    if (!part.model) {
      part.xianyu = ''
      state.error.value = null
      return
    }

    state.loading.value = true
    state.error.value = null

    try {
      const result = await invoke('crawl_prices', { keyword: part.model })
      part.xianyu = result.xianyu != null ? String(result.xianyu) : ''
    }
    catch (err) {
      state.error.value = typeof err === 'string' ? err : (err?.message ?? '爬取失败')
    }
    finally {
      state.loading.value = false
    }
  }

  /**
   * 防抖触发爬取
   */
  function scheduleCrawl(part) {
    const state = stateMap[part.key]
    clearTimeout(state.timer)
    state.timer = setTimeout(() => fetchPrices(part), DEBOUNCE_MS)
  }

  // 为每行的 model 字段注册 watcher
  parts.value.forEach((part) => {
    watch(
      () => part.model,
      (newVal, oldVal) => {
        if (newVal !== oldVal) {
          scheduleCrawl(part)
        }
      },
    )
  })

  /**
   * 获取某行的 loading / error 状态，供模板绑定
   */
  function getRowState(key) {
    return stateMap[key] ?? { loading: ref(false), error: ref(null) }
  }

  /**
   * 手动立即触发某行爬取（跳过防抖）
   */
  function manualCrawl(part) {
    const state = stateMap[part.key]
    clearTimeout(state.timer)
    fetchPrices(part)
  }

  return { getRowState, manualCrawl }
}
