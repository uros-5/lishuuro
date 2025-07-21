import type { Api } from 'chessground12/api'
import { setCheck } from 'chessground12/board'
import { p, p2 } from 'chessground12/configs'
import type { ShuuroPosition } from 'shuuro-wasm'
import { ref, watch, type WatchStopHandle } from 'vue'
import { readPockets } from 'chessground12/pocket'
import type { Key, MoveMetadata, Piece, Color } from 'chessground12/types'
import { defineStore } from 'pinia'
import { useGameStore, type Stage } from '.'
import { useWasmStore } from './useWasmStore'
import { useAnalyzeStore } from './useAnalyzeStore'
import { useClockStore } from './useClockStore'
import { FenBtn } from '@/helpers/fen'
import { deployCg, fightCg } from '@/helpers/cg'
import { variantStr } from '@/helpers/variantDescription'
import { zoom } from '@/helpers/zoom'
import { useWs } from '../ws'
import { cgInfo } from '../tvStore'
import { MessageType } from '@/helpers/rust_types'

export const useCgStore = defineStore('useCgStore', () => {
  const state = ref(empty())
  const others = ref(emptyOthers())
  const wasmStore = useWasmStore()
  const analyzeStore = useAnalyzeStore()
  const gameStore = useGameStore()
  const clockStore = useClockStore()
  const ws = useWs()
  let watcher: WatchStopHandle
  return {
    state,
    others,
    cg(): Api | undefined {
      return state.value.cg
    },

    watch() {
      watcher = watch(
        state,
        (n, o) => {
          this.cgWatcher(n, o)
        },
        { deep: true },
      )
    },

    deployCg() {
      if (state.value.cg) {
        state.value.cg.destroy()
      }
      let geo = cgInfo(gameStore.state.variant)
      const cg = deployCg(
        state.value.element,
        gameStore.config(),
        state.value.top,
        state.value.bot,
        geo[0],
        geo[1],
      )
      state.value.cg = cg
      this.changePocketRoles()
      state.value.cg.state.events.insert = (el) => {
        zoom(el)
      }
      gameStore.setSfen(wasmStore.current()!)
      gameStore.player().player == 1 ? this.flipBoard(cg) : null
      cg.state.plinthsPlaced = true

      cg.state.events.pocketSelect = this.pocketSelect
      cg.state.events.dropNewPiece = this.afterPlace
      cg.state.movable.color = gameStore.playerColor() as Color
    },

    fightCg() {
      if (state.value.cg) {
        state.value.cg.destroy()
      }
      let info = cgInfo(gameStore.state.variant)
      const cg = fightCg(state.value.element, gameStore.config(), info[0], info[1])
      state.value.cg = cg
      gameStore.setSfen(wasmStore.current()!)
      gameStore.player().player == 1 ? this.flipBoard(cg) : null
      cg.state.plinthsPlaced = true
      this.enablePremove()

      state.value.cg.state.events.insert = (el) => {
        zoom(el)
      }

      state.value.cg.state.events.select = this.selectSq
      state.value.cg.state.movable.events.after = this.afterMove
      cg.state.movable.color = gameStore.playerColor()
    },

    cgWatcher(newstate: State, _oldstate: State) {},

    getCg(cg?: Api): Api | undefined {
      return !cg ? this.cg() : cg
    },

    getPosition(pos?: ShuuroPosition): ShuuroPosition | undefined {
      return !pos ? wasmStore.current() : pos
    },

    flipBoard(cg?: Api) {
      const selection = gameStore.state.current_stage == 0 || gameStore.user.clientStage == 0
      cg = !cg ? this.cg() : cg
      if (!selection && cg) {
        if (cg.state.orientation != 'black') {
          cg.toggleOrientation()
          cg.state.plinthsPlaced = true
        }
      }
    },

    setPieces(cg?: Api, sp?: ShuuroPosition, force = true) {
      sp = this.getPosition(sp)
      cg = this.getCg(cg)
      const pieces = sp?.map_pieces()
      if (!pieces) return

      cg?.setPieces(pieces)
      if (force) {
        cg!.state.pieces = pieces
        cg!.state.dom.redraw()
      }
    },

    setPlinths(cg?: Api, sp?: ShuuroPosition, ignore?: boolean) {
      if (ignore == true) return
      sp = this.getPosition(sp)
      cg = this.getCg(cg)
      const plinths = sp!.map_plinths()
      cg!.state.plinths = plinths
      cg!.state.plinthsPlaced = false
      cg!.setPlinths(plinths)
      cg!.redrawAll()
      cg!.state.plinthsPlaced = true
    },

    setCheck(cg?: Api, sp?: ShuuroPosition) {
      sp = this.getPosition(sp)
      cg = this.getCg(cg)
      if (cg) {
        let check = sp!.is_check()
        setCheck(cg.state, check)
        cg.state.dom.redraw()
      }
    },

    changePocketRoles() {
      if (variantStr(gameStore.state.variant).endsWith('Fairy')) {
        state.value.cg!.state.pocketRoles = p2
        // state.value.cg!.state.dom.redraw();
        return
      }
      state.value.cg!.state.pocketRoles = p
    },

    readPocket(cg?: Api, sp?: ShuuroPosition) {
      sp = this.getPosition(sp)
      cg = this.getCg(cg)
      const hand = sp!.count_hand_pieces()
      // if(hand == "" || hand == "-") return
      cg!.state.pockets = readPockets(`[${hand}]`, state.value.cg!.state.pocketRoles!)
      cg!.state.dom.redrawNow()
    },

    pocketSelect(piece: Piece) {
      if (!gameStore.canPlay()) {
        return
      }
      if (gameStore.user.historyIndex == gameStore.history().length - 1) {
        const ch = wasmStore.wasmPiece(piece)
        const moves = wasmStore.current()!.place_moves(ch)
        this.new_legal_moves(moves)
      }
    },

    afterPlace(piece: Piece, key: Key) {
      if (!gameStore.canPlay()) {
        return
      }
      const p = wasmStore.wasmPiece(piece)
      const gameMove = `${p}@${key}`
      ws.SEND({ t: MessageType.PlacePiece, d: gameMove })
      this.wasmPlace(gameMove, false)
      others.value.last_move = gameMove
    },

    wasmPlace(gameMove: string, isServer: boolean) {
      if (isServer && gameMove == others.value.last_move) return
      const placed = wasmStore.current()!.place(gameMove)
      if (!placed) return
      this.addPlaceMove(placed)
      this.placementStm()
      if (isServer) {
      }
      this.setCheck()
    },

    wasmMove(game_move: string) {
      const wasm = wasmStore.current()!
      const played = wasm.make_move(game_move)
      if (played == undefined) return

      const lastMove = wasm.last_move()
      if (lastMove.includes('=')) {
        this.setPieces(state.value.cg!, wasm)
      }
      this.setTurnColor()
      this.setCheck()
      clockStore.switchClock()
      if (!analyzeStore.isActive()) {
        gameStore.addMove(gameStore.state.current_stage as Stage, lastMove)
      } else {
        analyzeStore.addAnalyzeMove(lastMove)
      }
      gameStore.scrollToBottom()
      gameStore.lastMoveIndex(gameStore.state.current_stage)
      state.value.cg!.state.dom.redraw()
      const lm = gameStore.legal_moves()
      this.new_legal_moves(lm)
      this.playPremove()
    },

    playPremove() {
      if (others.value.premoveData.active && gameStore.canPlay()) {
        state.value.cg!.playPremove()
        others.value.premoveData.active = false
      }
    },

    sendMove(s: string) {
      if (!analyzeStore.isActive()) {
        ws.SEND({ t: MessageType.MovePiece, d: s })
      }
    },

    addPlaceMove(placed: string) {
      const wasm = wasmStore.current()!
      const last_move = wasm.last_move()
      gameStore.addMove(1, last_move)
      gameStore.scrollToBottom()
      gameStore.lastMoveIndex(1)
      this.readPocket()
      gameStore.state.sfen = wasm.generate_sfen()
      gameStore.audio('move')
    },

    placementStm() {
      const wasm = wasmStore.current()!
      clockStore.pause(gameStore.state.side_to_move, false)
      this.setPieces(state.value.cg!, wasm)
      this.setTurnColor()
      clockStore.switchClock()
    },

    afterMove(orig: Key, dest: Key, _metadata: MoveMetadata) {
      const gameMove = `${orig}_${dest}`
      this.wasmMove(gameMove)
      gameStore.audio('move')
      if (_metadata.captured!) {
        gameStore.audio('capture')
      }
      ws.SEND({ t: MessageType.MovePiece, d: gameMove })
    },

    enablePremove() {
      if (gameStore.player().isPlayer && gameStore.state.status < 1) {
        state.value.cg!.state.premovable.events = {
          set: (orig, dest) => {},
          unset: () => {},
        }
        state.value.cg!.state.premovable.enabled = true
        state.value.cg!.state.premovable!.events!.set = (orig, dest, _) => {
          others.value.premoveData.orig = orig
          others.value.premoveData.dest = dest
          others.value.premoveData.active = true
        }
        state.value.cg!.state.premovable!.events!.unset! = () => {
          others.value.premoveData.active = false
        }
      }
    },

    // useless method
    selectSq(_key: Key) {
      if (gameStore.canPlay()) {
        if (state.value.cg!.state.movable.dests?.size == 0) {
          gameStore.legal_moves()
        }
      } else if (analyzeStore.isActive()) {
        // const dests = state.value.cg!.state.movable.dests;
        // if (dests?.size == 0 || dests == undefined) {
        //   const lm = gameStore.legal_moves();
        //   this.new_legal_moves(lm);
        // }
      }
    },

    new_legal_moves(lm: Map<any, any>) {
      if (state.value.cg) {
        state.value.cg.state.movable.dests?.clear()
        state.value.cg.state.movable.dests = lm
      }
    },

    setMovable(movable: boolean, cg?: Api) {
      cg = this.getCg(cg)
      cg!.state.movable.showDests = movable
      cg!.state.draggable.enabled = movable
      cg!.state.movable.color = analyzeStore.isActive() ? 'both' : gameStore.playerColor()
      movable ? (cg!.state.movable.dests = new Map()) : null
    },

    setDraggable(draggable: boolean, cg?: Api) {
      cg = this.getCg(cg)
      cg!.state.draggable.enabled = draggable
      cg!.state.dropmode.active = draggable
    },

    enableMovable(fenBtn: FenBtn) {
      const enable = (value: boolean) => {
        this.setMovable(value)
        this.setDraggable(value)
      }
      if (analyzeStore.isActive()) {
        enable(true)
        this.setTurnColor()
        return
      }
      const checks = [
        fenBtn == FenBtn.Last,
        gameStore.state.status < 0,
        gameStore.player().isPlayer,
        gameStore.state.current_stage == gameStore.clientStage(),
      ]
      !checks.includes(false) ? enable(true) : enable(false)
    },

    setTurnColor(turnColor?: string) {
      let pos = wasmStore.current()!
      if (turnColor == undefined) {
        turnColor = pos.side_to_move()
      }
      turnColor = turnColor.startsWith('w') ? 'white' : 'black'
      if (!analyzeStore.isActive()) {
        gameStore.state.side_to_move = turnColor == 'white' ? 0 : 1
      }
      state.value.cg!.state.turnColor = turnColor as Color
      if (analyzeStore.isActive()) {
        state.value.cg!.state.movable.color = turnColor as Color
      }
    },

    newPiece(to: string, cg?: Api) {},

    reset() {
      state.value = empty()
      others.value = emptyOthers()
      // watcher();
    },
  }
})

function empty(): State {
  return {
    cg: undefined,
    element: {} as any,
    top: {} as any,
    bot: {} as any,
    tvCgs: [],
    profileGames: [],
  }
}

function emptyOthers() {
  return {
    premoveData: {
      orig: '',
      dest: '',
      active: false,
    },
    stage: 0,
    last_move: '',
  }
}

type State = {
  cg: Api | undefined
  element: HTMLElement
  top: HTMLElement
  bot: HTMLElement
  tvCgs: Api[]
  profileGames: Api[]
}

export enum CgElement {
  Main = 0,
  Top,
  Bot,
  None,
}

type premoveData = {
  orig: string
  dest: string
  active: boolean
}
