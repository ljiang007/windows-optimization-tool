<script setup>
const emit = defineEmits(['tool-click'])

const tools = [
  'Windows激活',
  'Windows更新设置',
  '一键高性能',
  '一键关闭关闭UAC通知/文件安全警告',
  '开启/关闭防火墙',
  'Defender开关',
]
</script>

<template>
  <section class="tool-panel" aria-labelledby="system-title">
    <div class="panel-title" id="system-title">系统优化类</div>
    <div class="zhu">注：最好依次按顺序点击，Defender开关执行完会重启电脑</div>
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
