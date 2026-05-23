import { createApp } from 'vue'
import naive from 'naive-ui'
import App from './App.vue'
import './styles.css'

document.addEventListener('contextmenu', (event) => {
  event.preventDefault()
})

document.addEventListener('selectstart', (event) => {
  event.preventDefault()
})

document.addEventListener('dragstart', (event) => {
  event.preventDefault()
})

document.addEventListener('keydown', (event) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'a') {
    event.preventDefault()
  }
})

createApp(App).use(naive).mount('#app')
