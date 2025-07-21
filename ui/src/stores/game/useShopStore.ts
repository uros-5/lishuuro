import { ref } from 'vue'
import { defineStore } from 'pinia'
import { useWasmStore } from './useWasmStore'
import { useClockStore } from './useClockStore'
import { useGameStore } from '.'
import { useWs } from '../ws'
import { MessageType } from '@/helpers/rust_types'

export const useShopStore = defineStore('useShopStore', () => {
  const wasmStore = useWasmStore()
  const clockStore = useClockStore()
  const gameStore = useGameStore()
  const state = ref(empty())
  const ws = useWs()
  return {
    state,

    shopInfo(): void {
      ws.SEND({ t: MessageType.GetHand, d: null } )
      ws.SEND({ t: MessageType.GetConfirmed, d: null })
    },

    setConfirmed(data: [boolean, boolean]) {
      state.value.confirmed = data
      if (!data.includes(false)) {
        return
      }

      data.includes(true) ? this.startClock() : null
    },

    async setHand(hand: string) {
      await wasmStore.state.init
      wasmStore.shop().set_hand(hand)
      state.value.pieceCounter = wasmStore.shop().shop_items(gameStore.playerColor())
      let moves: string[] = wasmStore.shop().history()
      moves = moves.filter((item) => {
        if (item.toLowerCase() == '+k') return false
        return true
      })
      gameStore.addMoves(0, moves)
      state.value.credit = wasmStore.shop().get_credit(gameStore.playerColor())
      if (gameStore.state.current_stage == 0) {
        gameStore.user.historyIndex = gameStore.state.history[0].length - 2
      } else {
        gameStore.user.historyIndex = 0
      }
    },

    buy(p: string, color: string) {
      p = color == 'white' ? p.toUpperCase() : p.toLowerCase()
      if (this.canShop()) {
        const game_move = `+${p}`
        state.value.pieceCounter = wasmStore.shop().buy(game_move)
        state.value.pieceCounter[0] = 1
        const new_credit = wasmStore.shop().get_credit(color[0])
        const _ = wasmStore.shop().get_piece(p)
        if (new_credit != state.value.credit) {
          gameStore.addMove(0, game_move)
          gameStore.scrollToBottom()
        }
        ws.SEND({ t: MessageType.SelectMove, d: game_move })
        gameStore.user.historyIndex = gameStore.state.history[0].length - 1
        state.value.credit = new_credit
        gameStore.audio('buy')
      }
    },

    confirm() {
      if (this.canShop()) {
        wasmStore.shop().confirm(gameStore.playerColor()[0])
        if (wasmStore.shop().is_confirmed(gameStore.playerColor()[0])) {
          ws.SEND({ t: MessageType.ConfirmSelection, d: 'cc' })
          clockStore.pause(gameStore.player().player)
          state.value.confirmed[gameStore.player().player] = true
        }
      }
    },

    startClock() {
      const elapsed = clockStore.elapsed()
      const confirmed = state.value.confirmed.findIndex((item) => item == true)
      if (confirmed == -1) {
        clockStore.startBoth(elapsed, gameStore.state.tc.clocks)
        return
      }
      clockStore.setTime(confirmed, gameStore.state.tc.clocks[confirmed] - elapsed)
      clockStore.pause(confirmed, false)
      const other = clockStore.otherClock(confirmed)
      clockStore.start(other)
    },

    amIConfirmed(): boolean {
      return state.value.confirmed[gameStore.player().player]
    },

    canPlay(): boolean {
      if (gameStore.state.side_to_move == gameStore.player().player && gameStore.state.status < 1) {
        return true
      }
      return false
    },

    canShop(): boolean {
      return (
        gameStore.player().isPlayer &&
        gameStore.state.current_stage == 0 &&
        gameStore.state.status < 0 &&
        !this.amIConfirmed() &&
        !gameStore.state.players.includes('')
      )
    },

    pieceCounter() {
      return state.value.pieceCounter
    },

    credit() {
      return state.value.credit
    },

    reset() {
      state.value = empty()
    },
  }
})

function empty(): State {
  return {
    confirmed: [false, false],
    pieceCounter: new Uint8Array(),
    credit: 800,
  }
}

type State = {
  confirmed: [boolean, boolean]
  pieceCounter: Uint8Array
  credit: number
}
