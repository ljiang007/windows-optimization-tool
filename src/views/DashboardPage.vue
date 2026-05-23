<script setup>
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useMessage } from 'naive-ui'
import NoticePanel from '../components/toolbox/NoticePanel.vue'
import SystemOptimizePanel from '../components/toolbox/SystemOptimizePanel.vue'
import UtilityToolsPanel from '../components/toolbox/UtilityToolsPanel.vue'

const message = useMessage()
const version = '1.0.0'
const qqQrCodeUrl = '/qq-qrcode.png'
const loading = ref(false)
const showQqQrCode = ref(false)
const toolboxWindow = ref(null)
let resizeObserver
let resizeFrame = 0

function resizeWindowToContent() {
  if (resizeFrame) cancelAnimationFrame(resizeFrame)

  resizeFrame = requestAnimationFrame(async () => {
    const content = toolboxWindow.value
    if (!content) return

    const width = 540
    const verticalChrome = window.outerHeight - window.innerHeight
    const contentHeight = Math.ceil(content.getBoundingClientRect().height)
    const targetHeight = Math.max(360, contentHeight + 14 + verticalChrome)

    try {
      await getCurrentWindow().setSize(new LogicalSize(width, targetHeight))
    } catch {
      // 浏览器预览环境没有 Tauri window API，忽略即可。
    }
  })
}

onMounted(async () => {
  await nextTick()
  resizeWindowToContent()

  if (toolboxWindow.value) {
    resizeObserver = new ResizeObserver(resizeWindowToContent)
    resizeObserver.observe(toolboxWindow.value)
  }
})

onBeforeUnmount(() => {
  if (resizeFrame) cancelAnimationFrame(resizeFrame)
  resizeObserver?.disconnect()
})

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
  '杀蠕虫病毒': 'killWormVirus',
  '微软常用运行库': 'microsoftCommonRuntimeLibraries',
  '修复DX11': 'fixDx11',
  '技术员一键重装(vip0)': 'technicianReinstall',
  '天喵一键重装(1788)': 'tianmiaoReinstall',
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

async function openYyDownloadPage() {
  loading.value = true
  try {
    const result = await invoke('open_yy_download_page')
    message.success(result)
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function openQishuiMusicPage() {
  loading.value = true
  try {
    const result = await invoke('open_qishui_music_page')
    message.success(result)
  } catch (error) {
    message.warning(String(error))
  } finally {
    loading.value = false
  }
}

async function openGoogleChromePage() {
  loading.value = true
  try {
    const result = await invoke('open_google_chrome_page')
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

  if (toolName === 'YY绿色多开版') {
    await openYyDownloadPage()
    return
  }

  if (toolName === '汽水音乐') {
    await openQishuiMusicPage()
    return
  }

  if (toolName === '谷歌浏览器') {
    await openGoogleChromePage()
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
    <section ref="toolboxWindow" class="toolbox-window" :class="{ 'is-loading': loading }">
      <NoticePanel />
      <SystemOptimizePanel @tool-click="handleToolClick" />
      <UtilityToolsPanel @tool-click="handleToolClick" />

      <footer class="app-footer">
        <span>版本号：{{ version }}</span>
        <button type="button" class="contact-qq" :disabled="loading" @click="showQqQrCode = true">联系QQ</button>
      </footer>
    </section>

    <div v-if="showQqQrCode" class="qq-modal-backdrop" @click.self="showQqQrCode = false">
      <div class="qq-modal">
        <button type="button" class="qq-modal-close" aria-label="关闭" @click="showQqQrCode = false">×</button>
        <div class="qq-modal-title">联系QQ</div>
        <img class="qq-qrcode" :src="qqQrCodeUrl" alt="QQ二维码" draggable="false">
      </div>
    </div>

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

.qq-modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 10000;
  display: grid;
  place-items: center;
  background: rgba(13, 18, 24, 0.34);
}

.qq-modal {
  position: relative;
  width: 236px;
  padding: 16px 18px 18px;
  border: 1px solid rgba(174, 183, 193, 0.72);
  border-radius: 8px;
  background: #f7f8f7;
  box-shadow: 0 18px 50px rgba(31, 46, 64, 0.26);
  text-align: center;
}

.qq-modal-close {
  position: absolute;
  top: 6px;
  right: 8px;
  width: 24px;
  height: 24px;
  padding: 0;
  border: 0;
  color: #6c7582;
  background: transparent;
  cursor: pointer;
  font-size: 20px;
  line-height: 24px;
}

.qq-modal-title {
  margin-bottom: 12px;
  color: #18202a;
  font-size: 14px;
  font-weight: 800;
}

.qq-qrcode {
  display: block;
  width: 180px;
  height: 180px;
  margin: 0 auto;
  border: 1px solid #cfd6dd;
  background: #fff;
  object-fit: contain;
}
</style>
