<script setup>
import { NConfigProvider, NMessageProvider, darkTheme } from 'naive-ui'
import DashboardPage from './views/DashboardPage.vue'
import PcDiyWindowPage from './views/PcDiyWindowPage.vue'

const isPcDiyWindow = window.location.hash === '#/pc-diy'
</script>

<template>
  <NConfigProvider :theme="darkTheme">
    <NMessageProvider>
      <PcDiyWindowPage v-if="isPcDiyWindow" />
      <DashboardPage v-else />
    </NMessageProvider>
  </NConfigProvider>
</template>
