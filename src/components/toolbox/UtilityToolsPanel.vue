<script setup>
const emit = defineEmits(['tool-click'])

const systemTools = [
  '软件卸载',
  '安装WinRAR',
  '杀蠕虫病毒',
  '微软常用运行库',
  '修复DX11',
  '技术员一键重装(vip0)',
  '天喵一键重装(1788)',
]

const entertainmentTools = [
  '电脑DIY',
  'YY绿色多开版',
  '汽水音乐',
  '谷歌浏览器',
]
</script>

<template>
  <section class="tool-panel" aria-labelledby="system-software-title">
    <div class="panel-title" id="system-software-title">系统软件类</div>
    <div class="tool-grid compact-grid">
      <button
        v-for="tool in systemTools"
        :key="tool"
        class="tool-button"
        type="button"
        @click="emit('tool-click', tool)"
      >
        {{ tool }}
      </button>
    </div>
  </section>

  <section class="tool-panel" aria-labelledby="entertainment-title">
    <div class="panel-title" id="entertainment-title">娱乐类</div>
    <div class="zhu">注：除电脑DIY外，其余娱乐类均为网站跳转，都为正规软件无毒无捆绑！</div>
    <div class="tool-grid compact-grid">
      <button
        v-for="tool in entertainmentTools"
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
