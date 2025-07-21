import { ref } from 'vue'
import { anonConfig, liveConfig } from 'chessground12/configs'
import { defineStore } from 'pinia'
import { ShuuroPosition } from 'shuuro-wasm'
import { type Color, type Key } from 'chessground12/types'
import { useWasmStore } from './useWasmStore'
import { useClockStore } from './useClockStore'
import { useShopStore } from './useShopStore'
import { useCgStore } from './useCgStore'
import { useAnalyzeStore } from './useAnalyzeStore'

import { useWs } from '../ws'
import { FenBtn, formatSfen } from '@/helpers/fen'
import { variantStr } from '@/helpers/variantDescription'
import { playAudio } from '@/helpers/audio'
import { ev2, rmEv2, type Event2 } from '@/helpers/customEvent'
import { newTitle } from '@/helpers/backend'
import { MessageType, type ConfirmSelection, type GameDraw, type GameEnd, type MovePiece, type PlacePiece, type PlayerSelection, type RedirectToPlacement, type ShuuroGame, type StartClock } from '@/helpers/rust_types'

export const useGameStore = defineStore('useGameStore', () => {
  const state = ref(emptyGame())
  const user = ref(defaultClientState())
  const wasmStore = useWasmStore()
  const clockStore = useClockStore()
  const shopStore = useShopStore()
  const cgStore = useCgStore()
  const analyzeStore = useAnalyzeStore()
  const ws = useWs()

  function setPlayer() {
    let index = state.value.players.findIndex((item) => {
      return item == ws.username
    })
    if (index != -1) {
      user.value.player.player = index
      user.value.player.isPlayer = true
      if (index == 1) cgStore.flipBoard()
    }
    if (ws.username.length > 3) {
      user.value.player.resolve(true)
      user.value.historyIndex = -1
    }
    newTitle(state.value.players[0] + ' vs ' + state.value.players[1])
  }

  const msg = new Map<MessageType, any>();


  let listener

  let s = {
    state,
    user,

    messageHandler() {
      msg.set(MessageType.NewPlayer, (_: any) => { setPlayer() });
      msg.set(MessageType.StartClock, (message: StartClock) => {
        state.value.players = message.players
        setPlayer()
        state.value.tc.last_click = message.click
        clockStore.setLastClock(message.click)
        clockStore.activateClock()
        shopStore.setHand('').then(() => { })
        ws.redirectToastOpen = false
      });
      msg.set(MessageType.Draw, (message: GameDraw) => {
        user.value.offeredDraw = message.draw_offer
      });
      msg.set(MessageType.GameEnd, (message: GameEnd) => {
        this.gameEnd(message)
      });
      msg.set(MessageType.GetHand, (message: PlayerSelection) => {
        shopStore.setHand(message.hand).then(() => { })
      });
      msg.set(MessageType.ConfirmSelection, (message: ConfirmSelection) => {
        if (!message.confirmed.includes(false)) {
          clockStore.pauseBoth()
        } else {
          let player = message.confirmed.indexOf(true)
          clockStore.pause(player)
        }
      });
      msg.set(MessageType.RedirectToGame, (message: RedirectToPlacement) => {
          this.redirectDeploy(message)
      });
      msg.set(MessageType.PlacePiece, (message: PlacePiece) => {
        this.serverPlace(message)
      });
      msg.set(MessageType.MovePiece, (message: MovePiece) => {
        this.serverMove(message)
      });

    },

    listen() {
      listener = ev2('wsMessage', this.onMessage)
    },

    onMessage(event: Event2) {
      // event.detail
      let detail = event.detail
      let fn = msg.get(detail.t);
      if (fn) {
        fn(detail);
      }

    },

    stopListening() {
      rmEv2(listener!)
    },

    loaded() {
      return user.value.server
    },

    player() {
      return user.value.player
    },

    playerColor(username?: string): Color | 'both' {
      let player = username ? this.findPlayer(username) : this.player().player
      switch (player) {
        case 0:
          return 'white'
        case 1:
          return 'black'
        default:
          if (analyzeStore.isActive()) {
            return 'both'
          }
          return 'none'
      }
    },

    findPlayer(username: string): number {
      const index = state.value.players.findIndex((item) => item == username)!
      return index
    },

    config() {
      if (this.player().isPlayer && state.value.status < 0) {
        if (this.clientStage() == state.value.current_stage) {
          return liveConfig
        }
      } else if (analyzeStore.isActive()) {
        return liveConfig
      }
      return anonConfig
    },

    setConfig() {
      // cg
    },

    clientStage(): Stage {
      return user.value.clientStage as Stage
    },

    offeredDraw() {
      return user.value.offeredDraw
    },

    history() {
      return state.value.history[this.clientStage()]
    },

    generateSfenHistory() {
      if (state.value.current_stage < 1) return
      let history: [string[], string[], string[]] = [[], [], []]
      let position = new ShuuroPosition(variantStr(state.value.variant))
      position.change_variant(state.value.variant)
      position.set_sfen(state.value.placement_start)
      state.value.placement_start != '' ? history[1].push(state.value.placement_start) : null
      for (let i = 0; i < state.value.history[1].length; i++) {
        let placed = position.place(state.value.history[1][i])
        if (placed != undefined) {
          history[1].push(placed)
        }
      }
      state.value.history[1] = history[1]
      position.free()
      user.value.historyIndex = history[1].length - 1
      user.value.generatedSfen = true

      if (state.value.current_stage < 2) return

      position = new ShuuroPosition(variantStr(state.value.variant))
      position.change_variant(state.value.variant)
      position.set_sfen(state.value.game_start)
      history[2].push(state.value.game_start)
      for (let i = 0; i < state.value.history[2].length; i++) {
        let move = position.make_move(state.value.history[2][i])
        if (move != undefined) {
          history[2].push(move)
        }
      }
      position.free()
      state.value.history = history
      user.value.historyIndex = history[user.value.clientStage].length - 1
      user.value.generatedSfen = true
      if (state.value.status > 0) {
        this.setSfen(wasmStore.state.position!)
      }
    },

    rejectDraw() {
      user.value.offeredDraw = false
    },

    watchCg() {
      cgStore.watch()
    },

    unMounted() {
      this.reset()
    },

    async fromServer(s: ShuuroGame) {
      user.value.clientStage = s.current_stage as Stage
      ws.SEND({ t: MessageType.ChangeRoom, d: `/game/${s._id}` })
      state.value = s
      user.value.server = true
      this.setTime(s)
      clockStore.fromServer(s)
      setPlayer()
      this.updateHeadTitle()
      user.value.historyIndex = -1
      clockStore.activateClock()
      await wasmStore.init()
      if (s.current_stage == 0) {
        shopStore.shopInfo()
      } else if ([1, 2].includes(s.current_stage)) {
        // this.setSfen(wasmStore.current()!);
        this.watchCg()
      }
      this.resSound()
      this.generateSfenHistory()
    },

    setTime(s: ShuuroGame) {
      state.value.min = s.min / 60000
      state.value.incr = s.incr / 1000
      state.value.last_clock = new Date(s.tc.last_click).toString()
    },

    currentSfen(): string {
      if (state.value.status > 0) {
        let sfen = state.value.history.at(this.clientStage())!.at(0)
        user.value.historyIndex = 0
        if (sfen) {
          return sfen
        }
      }
      let fen = state.value.sfen.split(' ')
      return `${fen[0]} ${fen[1]} ${fen[2]} ${fen[3]}`
    },

    addFirstMove(stage?: Stage) {
      stage = stage ? stage : this.clientStage()
      const firstItem = stage == 1 ? 0 : -1
      const item = state.value.history.at(1)!.at(firstItem)
      switch (stage) {
        case 1:
          if (item == undefined || item == '') {
            state.value.history[1].unshift(state.value.sfen)
          }
          break
        case 2:
          const fightItem = state.value.history.at(2)!.at(0)
          if (fightItem == undefined || fightItem == '') {
            state.value.history[2].unshift(item!)
          }
          break
      }
    },

    // start normal clock
    startClock() {
      const elapsed = clockStore.elapsed()
      const stm = state.value.side_to_move
      const other = clockStore.otherClock(stm)
      clockStore.setTime(other, state.value.tc.clocks[other])
      clockStore.start(stm, state.value.tc.clocks[stm] - elapsed)
      clockStore.pause(other)
    },



    resSound() {
      if (state.value.status <= 0) {
        this.audio('res')
      }
    },

    index() {
      return user.value.historyIndex
    },

    newClientStage(stage: Stage) {
      user.value.clientStage = stage
    },

    canPlay(): boolean {
      if (
        state.value.side_to_move == user.value.player.player &&
        state.value.status < 0 &&
        user.value.historyIndex == this.history().length - 1 &&
        user.value.generatedSfen
      ) {
        return true
      }
      return false
    },

    // legal moves for fight or analyze
    legal_moves(): Map<any, any> {
      const wasm = analyzeStore.isActive() ? wasmStore.analyze() : wasmStore.current()!
      const color = !analyzeStore.isActive() ? wasm.side_to_move() : wasm.side_to_move()
      const sfen = wasm.generate_sfen().split(' ')[0]
      if (!sfen.includes('k') || !sfen.includes('K')) {
        return new Map()
      } else if (state.value.status < 0 || analyzeStore.isActive()) {
        const moves = wasm.legal_moves(cgColor2(color))
        cgStore.new_legal_moves(moves)
        cgStore.setTurnColor(cgColor(color))
        return moves
      }
      return new Map()
    },

    wasmMove(game_move: string) {
      const wasm = wasmStore.current()!
      const beforeCount = wasm.pieces_count()
      const played = wasm.make_move(game_move)
      if (played == undefined) {
        return
      }

      const lastMove = wasm.last_move()
      const move = game_move.split('_')
      const newCount = wasm.pieces_count()

      cgStore.cg()!.move(move[0] as Key, move[1] as Key)
      if (newCount != beforeCount) {
        this.audio('capture')
      } else {
        this.audio('move')
      }
      if (lastMove.includes('=')) {
        cgStore.setPieces(cgStore.cg()!, wasm)
      }
      cgStore.setTurnColor()
      cgStore.setCheck()
      clockStore.switchClock()
      if (!analyzeStore.isActive()) this.addMove(2, lastMove)
      this.scrollToBottom()
      user.value.historyIndex += 1
      cgStore.cg()!.state.dom.redraw()
      this.legal_moves()
      this.playPremove()
      cgStore.others.last_move = game_move
    },

    playPremove() {
      if (cgStore.others.premoveData.active && this.canPlay()) {
        cgStore.cg()?.playPremove()
        cgStore.others.premoveData.active = false
      }
    },

    scrollToBottom(): void {
      const container = document.querySelector('#movelist')
      container!.scrollTop = container!.scrollHeight
    },

    addMove(h: 0 | 1 | 2, move: string) {
      state.value.history[h].push(move)
    },

    addMoves(h: 0 | 1 | 2, moves: string[]) {
      state.value.history[h] = moves
    },

    reset() {
      state.value = emptyGame()
      user.value = defaultClientState()
      cgStore.reset()
      shopStore.reset()
      wasmStore.reset()
      clockStore.reset()
      analyzeStore.reset()
    },

    tempPosition(sfen?: string) {
      let history = this.history()
      sfen = sfen == undefined ? history.at(user.value.historyIndex) : sfen
      if (sfen == undefined) return
      const position = new ShuuroPosition(variantStr(state.value.variant))
      position.change_variant(state.value.variant)
      let formatted = formatSfen(sfen!, true)
      position.set_sfen(formatted.sfen)
      cgStore.setPieces(cgStore.cg()!, position, true)
      cgStore.setTurnColor(position.side_to_move())
      cgStore.setCheck(cgStore.cg(), position)
      const sound = formatted.capture ? 'capture' : 'move'
      this.audio(sound)
      this.clientStage() == 1 ? cgStore.readPocket(cgStore.cg(), position) : null

      if (analyzeStore.isActive()) {
        wasmStore.state.position = position
        this.legal_moves()
      }

      !analyzeStore.isActive() ? position.free() : null
      if (formatted.game_move) {
        let isPlacement = formatted.game_move.includes('@')
        const parts = formatted.game_move.split(isPlacement ? '@' : '_')
        const from = parts[0]
        const to = parts[1]
        cgStore.cg()?.setLastMove(from, to)
      } else {
        cgStore.cg()!.state.lastMove! = []
      }
    },

    findFen(fenBtn: FenBtn) {
      const len = state.value.history[this.clientStage()].length
      const previous = user.value.historyIndex
      switch (fenBtn) {
        case FenBtn.First:
          user.value.historyIndex = 0
          break
        case FenBtn.Previous:
          user.value.historyIndex -= 1
          break
        case FenBtn.Next:
          user.value.historyIndex += 1
          break
        case FenBtn.Last:
          user.value.historyIndex = state.value.current_stage == 0 ? len - 2 : len - 1
          break
      }
      if (user.value.historyIndex <= 0) {
        user.value.historyIndex = 0
      } else if (user.value.historyIndex >= len - 1) {
        user.value.historyIndex = len - 1
        fenBtn = FenBtn.Last
      }
      if (fenBtn != FenBtn.Last) {
        cgStore.new_legal_moves(new Map())
      }
      if (user.value.clientStage == 0) return
      else if (previous == user.value.historyIndex) return
      this.tempPosition()
      cgStore.enableMovable(fenBtn)
    },

    isLiveSfen(sfen: string): boolean {
      if (state.value.status < 0) {
        let historySfen = state.value.history[state.value.current_stage].at(-1)
        if (historySfen == sfen) {
          return true
        }
      }
      return false
    },

    isIndexLive(stage: Stage): boolean {
      return user.value.historyIndex == state.value.history[stage].length - 1
    },

    serverPlace(msg: PlacePiece) {
      if (!this.isIndexLive(1)) this.findFen(FenBtn.Last)
      cgStore.wasmPlace(msg.sfen, true)
      clockStore.fromMove(msg.clocks)

      if (msg.next_stage) {
        user.value.generatedSfen = true
        state.value.current_stage = 2
        user.value.clientStage = 2
        state.value.tc.last_click = new Date().toString()
        clockStore.state.last_clock = new Date().toString()
        this.audio('res')
        this.addFirstMove(2)
        this.setSfen(wasmStore.current()!)

        cgStore.enablePremove()
        cgStore.state.cg!.state.events!.select = cgStore.selectSq
        cgStore.state.cg!.state.movable.events.after = cgStore.afterMove
        cgStore.state.cg!.state.movable.color = this.playerColor()
      }
      if (msg.first_move_error) {
        setTimeout(() => {
          this.audio('res')
          clockStore.pauseBoth()
          state.value.status = 1
          state.value.result = state.value.side_to_move
          cgStore.cg()!.set(anonConfig)
        }, 500)
      }
    },

    serverMove(msg: MovePiece) {
      if (!this.isIndexLive(2)) this.findFen(FenBtn.Last)
      this.newClientStage(2)
      this.wasmMove(msg.game_move)
      clockStore.fromMove(msg.clocks)
      if (msg.status > 0) {
        this.audio('res')
        clockStore.pauseBoth()
        state.value.status = msg.status
        state.value.result = msg.result
        this.scrollToBottom()
        cgStore.cg()?.set(anonConfig)
      }
    },

    gameOver(_: number) {
      // for (let i = 0; i < finished.length; i++) {
      //   if (outcome.startsWith(finished[i])) {
      //     return true;
      //   }
      // }
      // return false;
    },

    gameEnd(obj: GameEnd) {
      this.audio('res')
      clockStore.pauseBoth()
      state.value.status = obj.status
      state.value.result = obj.result
      user.value.offeredDraw = false
      this.scrollToBottom()
      if (this.clientStage() != 0) cgStore.cg()?.set(anonConfig)
      cgStore.new_legal_moves(new Map())
    },

    redirectDeploy(s: RedirectToPlacement) {
      clockStore.state.last_clock = new Date().toString()
      state.value.sfen = s.sfen
      state.value.side_to_move = 0
      clockStore.start(state.value.side_to_move)
      state.value.current_stage = 1
      user.value.clientStage = 1
      user.value.historyIndex = 0
      user.value.generatedSfen = true
      this.audio('res')
      this.addFirstMove(1)
    },

    setSfen(position: ShuuroPosition) {
      const sfen = this.currentSfen()
      // const sfen = this.currentSfen().replace("   ", " ");
      position.set_sfen(sfen)

      this.clientStage() == 1 ? cgStore.readPocket() : null
      cgStore.setPieces(cgStore.cg()!, position)
      cgStore.setPlinths(cgStore.cg()!, position)
      cgStore.setTurnColor()
      cgStore.setCheck()
      this.lastMoveIndex(this.clientStage())
      this.clientStage() == 2 ? this.legal_moves() : null
    },

    lastMoveIndex(stage: number) {
      if (state.value.status < 0) {
        user.value.historyIndex = state.value.history[stage].length - 1
      } else {
        user.value.historyIndex = 0
      }
    },

    audio(s: string) {
      playAudio(s, '100')
    },

    updateHeadTitle() { },
  }
  s.messageHandler();
  return s
})

function emptyGame(): ShuuroGame {
  return {
    _id: '',
    min: 0,
    incr: 0,
    players: ['', ''],
    status: 0,
    side_to_move: 0,
    last_clock: "",
    current_stage: 0,
    result: 2,
    variant: 0,
    credits: [800, 800],
    hands: ['', ''],
    sfen: '',
    history: [[], [], []],
    tc: {
      last_click: '',
      clocks: [0, 0],
    },
    sub_variant: 100,
    game_start: '',
    placement_start: '',
    clocks: [10_000, 10_000]
  }
}

function defaultClientState(): ClientState {
  let resolve = (_: boolean) => {
    return
  }
  let loaded: Promise<boolean> = new Promise((v) => {
    resolve = v
  })
  let state: ClientState = {
    offeredDraw: false,
    server: false,
    player: { isPlayer: false, player: 2, loaded, resolve },
    clientStage: 0,
    historyIndex: 0,
    generatedSfen: false,
  }
  return state
}

type ClientState = {
  offeredDraw: boolean
  server: boolean
  player: UserLive
  clientStage: Stage
  historyIndex: number
  generatedSfen: boolean
}

export type Stage = 0 | 1 | 2
export type Player = 0 | 1 | 2
export type PlayStr = 'white' | 'black' | 'none' | 'both'

function cgColor(color: string) {
  if (color == 'w') {
    return 'white'
  } else if (color == 'b') {
    return 'black'
  }
  ; ('none')
}
function cgColor2(color: string) {
  if (color == 'w') {
    return 0
  } else if (color == 'b') {
    return 1
  }
  return 2
}


export type UserLive = {
  isPlayer: boolean
  player: number
  loaded: Promise<boolean>
  resolve: (v: boolean) => void
}

