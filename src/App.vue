<script setup>
import { NConfigProvider, NMessageProvider, darkTheme } from 'naive-ui'
import DashboardPage from './views/DashboardPage.vue'
</script>

<template>
  <NConfigProvider :theme="darkTheme">
    <NMessageProvider>
      <DashboardPage />
    </NMessageProvider>
  </NConfigProvider>
</template>
