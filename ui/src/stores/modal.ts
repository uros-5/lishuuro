import { defineStore } from 'pinia'
import { ref } from 'vue'

export type modalType = 'empty' | 'ai' | 'game-settings' | 'friend-request'

export const useModal = defineStore('modal', () => {
  const active = ref('empty' as modalType)

  function toggle(value?: modalType) {
    active.value = value == undefined ? 'empty' : value
  }

  return { active, toggle }
})
