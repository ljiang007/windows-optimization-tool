<script setup>
const emit = defineEmits(['tool-click'])

const tools = [
  'Windows激活',
  'Windows更新设置',
  'Defender开关',
  '一键高性能',
  '打开声音设置',
  '设置系统级隐藏',
  '关闭UAC通知',
  'Win10右键菜单',
  '打开任务管理器',
  '开启/关闭防火墙',
]
</script>

<template>
  <section class="tool-panel" aria-labelledby="system-title">
    <div class="panel-title" id="system-title">系统优化类</div>
    <div class="tool-grid">
      <button
        v-for="tool in tools"
        :key="tool"
        class="tool-button"
        type="button"
        @click="emit('tool-click', tool)"
      >
        {{ tool }}
      </button>
    </div>
  </section>
</template>
