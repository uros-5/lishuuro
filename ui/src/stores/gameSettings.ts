import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as v from 'valibot'
import router from '@/router'

export const useGameSettings = defineStore('gameSettings', () => {
  const state = defaultSettings()
  const board = ref(state.board)
  const piece = ref(state.piece)
  const zoom = ref(state.zoom)
  const sound = ref(state.sound)

  function updateStorage() {
    localStorage.setItem('board', board.value)
    localStorage.setItem('piece', piece.value)
    localStorage.setItem('zoom ', zoom.value.toString())
    localStorage.setItem('sound', sound.value.toString())
  }

  function updateBoard(value: string) {
    board.value = value
    updateStorage()
  }
  function updatePiece(value: string) {
    if (value == "custom") {
      router.push("/custom-piece")
      return
    }
    piece.value = value
    updateStorage()
  }

  return { board, piece, zoom, sound, updateStorage, updateBoard, updatePiece }
})

const schema = v.object({
  board: v.pipe(
    v.string(),
    v.values(['blue', 'brown', 'brown_sand', 'brown_yellow', 'gray', 'green']),
  ),
  piece: v.pipe(v.string()),
  zoom: v.pipe(v.number(), v.minValue(0), v.maxValue(100)),
  sound: v.pipe(v.number(), v.minValue(0), v.maxValue(100)),
})
type Schema = v.InferOutput<typeof schema>

function defaultSettings(): Schema {
  const board = localStorage.getItem('board')
  const piece = localStorage.getItem('piece')
  let zoom = Number(localStorage.getItem('zoom'))
  const sound = Number(localStorage.getItem('sound'))
  if (zoom == 0) {
    zoom = 100
  }
  const obj = v.safeParse(schema, {
    board,
    sound,
    zoom,
    piece,
  })
  if (obj.success) {
    return obj.output
  }
  return {
    piece: 'merida',
    board: 'gray',
    zoom: 100,
    sound: 100,
  }
}
