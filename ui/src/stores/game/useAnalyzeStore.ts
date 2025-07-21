import { ref } from 'vue'
import { defineStore } from 'pinia'
import type { Key } from 'chessground12/types'
import { useGameStore } from '.'
import { useWasmStore } from './useWasmStore'
import { useCgStore } from './useCgStore'
import { FenBtn } from '@/helpers/fen'

export const useAnalyzeStore = defineStore('useAnalyzeStore', () => {
  const state = ref(empty())
  const gameStore = useGameStore()
  const wasmStore = useWasmStore()
  const cgStore = useCgStore()

  return {
    state,
    isActive(): boolean {
      return state.value.active
    },

    moves(): string[] {
      return state.value.moves
    },

    canAnalyze() {
      if (gameStore.state.current_stage == 2 && gameStore.state.status > -1) {
        return true
      }
      return false
    },

    toggle() {
      state.value.active = !state.value.active
    },

    addAnalyzeMove(move: string) {
      if (state.value.index == this.moves().length - 1) {
        state.value.moves.push(move)
        this.newIndex()
      } else {
        state.value.moves = state.value.moves.slice(0, state.value.index + 1)
        state.value.moves.push(move)
        this.newIndex()
      }
    },

    newIndex(index?: number) {
      if (index != undefined) state.value.index = index
      else {
        state.value.index = this.moves().length - 1
      }
    },

    reset() {
      state.value = empty()
    },

    findFen(fenBtn: FenBtn) {
      const len = this.moves().length
      switch (fenBtn) {
        case FenBtn.First:
          state.value.index = 0
          break
        case FenBtn.Previous:
          state.value.index -= 1
          break
        case FenBtn.Next:
          state.value.index += 1
          break
        case FenBtn.Last:
          state.value.index = len - 1
          break
      }
      if (state.value.index <= 0) {
        state.value.index = 0
      } else if (state.value.index >= len) {
        state.value.index = len - 1
      }
      this.selectSfen()
    },
    selectSfen() {
      gameStore.tempPosition(this.moves().at(state.value.index))
      cgStore.enableMovable(FenBtn.Next)
    },
    afterMove(orig: Key, dest: Key) {
      const gameMove = `${orig}_${dest}`
      wasmStore.analyze().make_move(gameMove)
    },
    deleteMoves() {
      state.value.moves = []
    },
  }
})

function empty() {
  return { active: false, moves: [] as string[], index: 0 }
}
