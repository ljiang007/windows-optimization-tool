<script setup>
import { NConfigProvider, NMessageProvider, darkTheme } from 'naive-ui'
import DashboardPage from './views/DashboardPage.vue'
import PcDiyWindowPage from './views/PcDiyWindowPage.vue'
import XianyuLoginPage from './views/XianyuLoginPage.vue'

const isPcDiyWindow = window.location.hash === '#/pc-diy'
const isXianyuLogin = window.location.hash === '#/xianyu-login'
</script>

<template>
  <NConfigProvider :theme="null">
    <NMessageProvider>
      <XianyuLoginPage v-if="isXianyuLogin" />
      <PcDiyWindowPage v-else-if="isPcDiyWindow" />
      <DashboardPage v-else />
    </NMessageProvider>
  </NConfigProvider>
</template>
