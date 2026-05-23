<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import NoticePanel from '../components/toolbox/NoticePanel.vue'
import SystemOptimizePanel from '../components/toolbox/SystemOptimizePanel.vue'
import UtilityToolsPanel from '../components/toolbox/UtilityToolsPanel.vue'

const message = useMessage()
const version = '1.0.0'
const loading = ref(false)

async function openWindowsActivationTool() {
  loading.value = true
  try {
    // Rust 端会等待工具进程关闭后才返回。
    await invoke('open_bundled_tool', {
      toolKey: 'windowsActivation',
    })
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function handleToolClick(toolName) {
  if (loading.value) return
  // 不同按钮后续可以在这里分流到不同的 Tauri/Rust 本地能力。
  if (toolName === 'Windows激活') {
    await openWindowsActivationTool()
    return
  }

  message.info(`${toolName} 功能演示入口，后续可在这里接入真实逻辑。`)
}
</script>

<template>
  <main class="toolbox-shell">
    <section class="toolbox-window" :class="{ 'is-loading': loading }">
      <NoticePanel />
      <SystemOptimizePanel @tool-click="handleToolClick" />
      <UtilityToolsPanel @tool-click="handleToolClick" />

      <footer class="app-footer">
        <span>版本号：{{ version }}</span>
      </footer>
    </section>

    <div v-if="loading" class="loading-overlay"></div>
  </main>
</template>

<style scoped>
.loading-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.3);
  cursor: not-allowed;
}
</style>
