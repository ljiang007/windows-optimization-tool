<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { emit, listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const qrImg = ref('')
const status = ref('loading')

let pollTimer = null
let timeoutTimer = null
let closeTimer = null
let unlistenQrReady = null
let unlistenLoginOk = null

async function pollQrCode() {
  const qr = await invoke('get_xianyu_qr').catch(() => null)
  if (qr) {
    qrImg.value = `data:image/png;base64,${qr}`
    status.value = 'ready'
    clearInterval(pollTimer)
    pollTimer = null
    if (timeoutTimer) {
      clearTimeout(timeoutTimer)
      timeoutTimer = null
    }
  }
}

function showQr(qr) {
  if (!qr) return
  qrImg.value = `data:image/png;base64,${qr}`
  status.value = 'ready'
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  if (timeoutTimer) {
    clearTimeout(timeoutTimer)
    timeoutTimer = null
  }
}

onMounted(async () => {
  await invoke('clear_xianyu_qr').catch(() => {})

  unlistenQrReady = await listen('xianyu-qr-ready', (event) => {
    showQr(event.payload)
  })

  unlistenLoginOk = await listen('xianyu-login-ok', async () => {
    status.value = 'done'
    if (closeTimer) clearTimeout(closeTimer)
    closeTimer = setTimeout(async () => {
      await getCurrentWebviewWindow().close().catch(() => {})
    }, 2000)
  })

  await pollQrCode()
  if (status.value !== 'ready' && status.value !== 'done') {
    pollTimer = setInterval(pollQrCode, 300)
    await emit('xianyu-login-ui-ready')
  }

  timeoutTimer = setTimeout(() => {
    if (status.value === 'loading') {
      status.value = 'timeout'
    }
  }, 100000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
  if (timeoutTimer) clearTimeout(timeoutTimer)
  if (closeTimer) clearTimeout(closeTimer)
  if (unlistenQrReady) unlistenQrReady()
  if (unlistenLoginOk) unlistenLoginOk()
})
</script>

<template>
  <div class="login-shell">
    <div class="login-card">
      <p class="eyebrow">XIANYU LOGIN</p>
      <h2>咸鱼扫码登录</h2>

      <div v-if="status === 'loading'" class="placeholder">
        <div class="spinner" />
        <p class="hint">二维码加载中…</p>
      </div>

      <div v-else-if="status === 'ready'" class="qr-wrap">
        <img :src="qrImg" class="qr-img" alt="咸鱼登录页面截图" />
        <p class="hint">请用手机扫码登录咸鱼</p>
        <p class="sub-hint">登录完成后本窗口将自动关闭</p>
      </div>

      <div v-else-if="status === 'timeout'" class="done-wrap">
        <div class="check warning">!</div>
        <p class="hint warning-text">二维码加载超时，请关闭窗口后重试</p>
      </div>

      <div v-else class="done-wrap">
        <div class="check">✓</div>
        <p class="hint success">登录成功，窗口即将关闭</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-shell {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100vh;
  background: #f0f2f5;
}

.login-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 100%;
  max-width: 680px;
  padding: 28px 24px 24px;
  border: 1px solid rgba(28, 40, 58, 0.14);
  border-radius: 18px;
  background: #fff;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
  margin: 0 16px;
}

.eyebrow {
  margin: 0;
  color: #b3472e;
  font-size: 10px;
  font-weight: 800;
  letter-spacing: 0.22em;
}

h2 {
  margin: 0;
  color: #172231;
  font-family: "Bahnschrift", "Segoe UI Semibold", "Microsoft YaHei UI", sans-serif;
  font-size: 20px;
  font-weight: 800;
}

.placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 0;
}

.spinner {
  width: 36px;
  height: 36px;
  border: 3px solid rgba(179, 71, 46, 0.2);
  border-top-color: #b3472e;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.qr-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  width: 100%;
}

.qr-img {
  width: auto;
  max-width: 340px;
  max-height: 340px;
  object-fit: contain;
  border: 1px solid rgba(28, 40, 58, 0.1);
  border-radius: 10px;
  display: block;
}

.hint {
  margin: 0;
  color: #5a6676;
  font-size: 13px;
  text-align: center;
}

.sub-hint {
  margin: 0;
  color: #9aa3ae;
  font-size: 11px;
  text-align: center;
}

.done-wrap {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 32px 0;
}

.check {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(76, 175, 80, 0.12);
  color: #4caf50;
  font-size: 24px;
  font-weight: 800;
}

.check.warning {
  background: rgba(255, 152, 0, 0.12);
  color: #ff9800;
}

.hint.success {
  color: #4caf50;
  font-size: 14px;
  font-weight: 700;
}

.hint.warning-text {
  color: #ff9800;
  font-size: 14px;
  font-weight: 700;
}
</style>
