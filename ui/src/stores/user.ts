import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUser = defineStore('user', () => {
  const user = ref(true)
  const live = ref(false)
  return { user, live }
})
