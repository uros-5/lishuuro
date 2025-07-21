import { defineStore } from 'pinia'
import { ref } from 'vue'

import { ntw } from '@/not-tailwind'

export const useMaxWidthStore = defineStore('maxwidth', () => {
  const header = ref<HTMLElement>()
  const clock = ref<HTMLElement>()
  const firstItem = ref<HTMLElement>()
  const placement = ref<HTMLElement>()

  const headerHeight = ntw.get('--header-height')
  const clockHeight = ntw.get('--clock-height')
  const placementHeight = ntw.get('--placement-height')

  function resize() {
    if (header.value) {
      document.body.style.setProperty(
        headerHeight,
        `${header.value.getBoundingClientRect().height}px`,
      )
    }
    if (clock.value) {
      document.body.style.setProperty(
        clockHeight,
        `${clock.value.getBoundingClientRect().height}px`,
      )
    }
    if (placement.value) {
      document.body.style.setProperty(
        placementHeight,
        `${placement.value.getBoundingClientRect().height}px`,
      )
    }
    // if(firstItem.value) {
    //   document.body.style.setPropertyy('--empty-item-height',`${firstItem.value.getBoundingClientRect().height}px`)
    // }
  }
  return { header, clock, firstItem, placement, resize }
})
