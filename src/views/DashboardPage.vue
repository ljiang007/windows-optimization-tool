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

async function openBundledTool(toolKey) {
  loading.value = true
  try {
    // Rust 端会等待工具进程关闭后才返回。
    await invoke('open_bundled_tool', { toolKey })
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

const toolKeyMap = {
  'Windows激活': 'windowsActivation',
  'Windows更新设置': 'windowsUpdateSettings',
  '彻底关闭实时防护/杀毒功能': 'defenderSwitch',
  '软件卸载': 'softwareUninstall',
  '安装WinRAR': 'installWinrar',
}

async function disableUacAndFileWarning() {
  loading.value = true
  try {
    const result = await invoke('disable_uac_and_file_warning')
    message.success(result)
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function setHighPerformancePowerPlan() {
  loading.value = true
  try {
    const result = await invoke('set_high_performance_power_plan')
    message.success(result)
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function permanentlyDisableFirewallByRegistry() {
  loading.value = true
  try {
    const result = await invoke('permanently_disable_firewall_by_registry')
    message.success(result)
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function handleToolClick(toolName) {
  if (loading.value) return

  if (toolName === '一键高性能') {
    await setHighPerformancePowerPlan()
    return
  }

  if (toolName === '一键关闭关闭UAC通知/文件安全警告') {
    await disableUacAndFileWarning()
    return
  }

  if (toolName === '彻底禁用防火墙') {
    await permanentlyDisableFirewallByRegistry()
    return
  }

  const toolKey = toolKeyMap[toolName]
  if (toolKey) {
    await openBundledTool(toolKey)
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
  cursor: not-allowed;
}
</style>
