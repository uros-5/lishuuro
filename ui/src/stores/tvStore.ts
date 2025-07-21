import { defineStore } from 'pinia'
import { useWs } from './ws'
import { ref, type Ref } from 'vue'
import { ev2, rmEv2, type Event2 } from '@/helpers/customEvent'
import { ShuuroPosition } from 'shuuro-wasm'
import init from 'shuuro-wasm'
import { Variant, variantStr } from '@/helpers/variantDescription'
import { formatSfen } from '@/helpers/fen'
import { Geometry, type BoardDimensions } from 'chessground12/types'
import { anonConfig } from 'chessground12/configs'
import { fightCg } from '@/helpers/cg'
import type { Api } from 'chessground12/api'
import { setCheck } from 'chessground12/board'
import { MessageType, type AllGames, type NewTvGame, type NewTvMove, type RedirectToPlacement, type RemoveTvGame } from '@/helpers/rust_types'

export type TvGame = (RedirectToPlacement & {cg?: Api})

export const useTvStore = defineStore('tv', () => {
  const ws = useWs()
  const games = ref([]) as Ref<TvGame []>
  const msg = new Map<MessageType, any>();
  msg.set(MessageType.GetTv, (tv: AllGames) => games.value = tv.games);
  msg.set(MessageType.RemoveTVGame, (message: RemoveTvGame) => {
    let game = message.game
    games.value = games.value.filter((item) => item.id != game)
  });
  msg.set(MessageType.AddTvGame, (message: NewTvGame) => {
    games.value.push(message.game)
  });
  msg.set(MessageType.NewTvMove, (message: NewTvMove) => {
    let game = games.value.find((game) => game.id == message.game)
    if (game) {
      game.sfen = message.game_move
      tempPosition(game)
    }
  })
  let listener
  let s = {
    games,
    onMessage(event: Event2) {
      let detail = event.detail
      let fn = msg.get(detail.t);
      if (fn) {
        fn(detail);
      }

    },
    listen() {
      listener = ev2('wsMessage', this.onMessage)
    },
    stopListening() {
      rmEv2(listener!)
      ws.SEND({ t: MessageType.ChangeRoom, d: '' })
    },
    updateCg(id: string, cg?: Api) {
      let game = games.value.find((game) => game.id == id)
      if (game) {
        if (cg) {

          game.cg = cg
        }
        tempPosition(game)
      }
    },
    async init() {
      await init()
    },
  }
  return s
})

export function tvCg(element: HTMLElement, game: RedirectToPlacement) {
  let info = cgInfo(game.variant)
  const cg = fightCg(element, anonConfig, info[0], info[1])
  return cg
}

export function cgInfo(variant: number): [Geometry, BoardDimensions[]] {
  if (variant == Variant.Shuuro || variant == Variant.ShuuroFairy) {
    return [Geometry.dim12x12, [{ width: 12, height: 12 }]]
  } else if (variant == Variant.Standard || variant == Variant.StandardFairy) {
    return [Geometry.dim8x8, [{ width: 8, height: 8 }]]
  } else if (variant == Variant.ShuuroMini || variant == Variant.ShuuroMiniFairy) {
    return [Geometry.dim6x6, [{ width: 6, height: 6 }]]
  }

  return [Geometry.dim12x12, [{ width: 12, height: 12 }]]
}

function setPieces(cg: Api, sp: ShuuroPosition, force = true) {
  const pieces = sp.map_pieces()
  cg.setPieces(pieces)
  if (force) {
    cg.state.pieces = pieces
    cg.state.dom.redraw()
  }
}

function setPlinths(cg: Api, sp: ShuuroPosition, ignore?: boolean) {
  if (ignore == true) return
  const plinths = sp.map_plinths()
  cg.state.plinths = plinths
  cg.state.plinthsPlaced = false
  cg.setPlinths(plinths)
  cg.redrawAll()
  cg.state.plinthsPlaced = true
}

function setCheck2(cg: Api, sp: ShuuroPosition) {
  let check = sp!.is_check()
  setCheck(cg.state, check)
  cg.state.dom.redraw()
}

export function tempPosition(game: TvGame) {
  let sfen = game.sfen
  const position = new ShuuroPosition(variantStr(game.variant))
  position.change_variant(game.variant)
  let formatted = formatSfen(sfen, true)
  position.set_sfen(formatted.sfen)

  let turnColor = position.side_to_move() == 'w' ? 'white' : 'black'

  game.cg!.state.turnColor = turnColor as "white" | "black" | "none";
  setPieces(game.cg as Api, position, true)
  setPlinths(game.cg as Api, position)
  setCheck2(game.cg as Api, position)
  if (formatted.game_move) {
    let isPlacement = formatted.game_move.includes('@')
    const parts = formatted.game_move.split(isPlacement ? '@' : '_')
    const from = parts[0]
    const to = parts[1]
      ; (game.cg as Api).setLastMove(from, to)
  } else {
    ; (game.cg as Api).state.lastMove! = []
  }
}
