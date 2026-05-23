<script setup>
const emit = defineEmits(['tool-click'])

const tools = [
  '软件卸载',
  '安装WinRAR',
]
</script>

<template>
  <section class="tool-panel" aria-labelledby="utility-title">
    <div class="panel-title" id="utility-title">软件安装类</div>
    <div class="tool-grid compact-grid">
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
