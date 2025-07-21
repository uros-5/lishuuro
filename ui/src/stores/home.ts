import { homeState } from '@/helpers/home_state'
import { variants, type BoardSize, type Description } from '@/helpers/variantDescription'
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useHome = defineStore('home', () => {
  const storage = homeState()
  const minute = ref(storage.minutes)
  const increment = ref(storage.increment)
  const selectedCategory = ref(storage.category as BoardSize)
  const selectedVariant = ref(
    variants.find(
      (item) => item.variant == storage.variant && item.subVariant == storage.subVariant,
    ),
  )
  const viewVariant = ref(undefined as Description | undefined)
  const animateSelected = ref(false)
  const friendModal = ref(false)
  const aiModal = ref(false)
  const aiLevel = ref({ level: 0, selected: false })
  const friend = ref({ name: '', selected: false })
  const color = ref(2)
  const currentView = ref('' as '' | 'options' | 'variant')
  let players = ref(0)
  let games = ref(0)

  return {
    minute,
    increment,
    selectedCategory,
    selectedVariant,
    viewVariant,
    animateSelected,
    friendModal,
    aiModal,
    aiLevel,
    friend,
    color,
    currentView,
    players,
    games,
  }
})
