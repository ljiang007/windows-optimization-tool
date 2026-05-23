<script setup>
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import NoticePanel from '../components/toolbox/NoticePanel.vue'
import SystemOptimizePanel from '../components/toolbox/SystemOptimizePanel.vue'
import UtilityToolsPanel from '../components/toolbox/UtilityToolsPanel.vue'

const message = useMessage()
const version = '1.0.0'

async function openWindowsActivationTool() {
  const loadingMsg = message.loading('正在打开 Windows激活工具...', { duration: 0 })
  try {
    const delay = ms => new Promise(resolve => setTimeout(resolve, ms))
    // 前端只传工具 key，真正的 exe 路径由 Rust 后端白名单决定。
    const resultMessage = await invoke('open_bundled_tool', {
      toolKey: 'windowsActivation',
    })
    // 确保 loading 至少显示 1 秒
    await delay(1000)
    loadingMsg.destroy()
    message.success(resultMessage)
  } catch (error) {
    loadingMsg.destroy()
    message.warning(String(error))
  }
}

async function handleToolClick(toolName) {
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
    <section class="toolbox-window">
      <NoticePanel />
      <SystemOptimizePanel @tool-click="handleToolClick" />
      <UtilityToolsPanel @tool-click="handleToolClick" />

      <footer class="app-footer">
        <span>版本号：{{ version }}</span>
      </footer>
    </section>
  </main>
</template>
