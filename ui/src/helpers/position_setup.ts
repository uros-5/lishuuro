import type { Api } from 'chessground12/api'

export function setup6(cg: Api) {
  if (cg.state.dimensions.width == 6) {
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'a2')
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'b2')
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'c2')
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'd2')
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'e2')
    cg.newPiece({ color: 'white', role: 'p-piece' }, 'f2')

    cg.newPiece({ color: 'white', role: 'r-piece' }, 'a1')
    cg.newPiece({ color: 'white', role: 'n-piece' }, 'b1')
    cg.newPiece({ color: 'white', role: 'q-piece' }, 'c1')
    cg.newPiece({ color: 'white', role: 'k-piece' }, 'd1')
    cg.newPiece({ color: 'white', role: 'b-piece' }, 'e1')
    cg.newPiece({ color: 'white', role: 'r-piece' }, 'f1')

    cg.newPiece({ color: 'black', role: 'p-piece' }, 'a5')
    cg.newPiece({ color: 'black', role: 'p-piece' }, 'b5')
    cg.newPiece({ color: 'black', role: 'p-piece' }, 'c5')
    cg.newPiece({ color: 'black', role: 'p-piece' }, 'd5')
    cg.newPiece({ color: 'black', role: 'p-piece' }, 'e5')
    cg.newPiece({ color: 'black', role: 'p-piece' }, 'f5')

    cg.newPiece({ color: 'black', role: 'r-piece' }, 'a6')
    cg.newPiece({ color: 'black', role: 'n-piece' }, 'b6')
    cg.newPiece({ color: 'black', role: 'q-piece' }, 'c6')
    cg.newPiece({ color: 'black', role: 'k-piece' }, 'd6')
    cg.newPiece({ color: 'black', role: 'b-piece' }, 'e6')
    cg.newPiece({ color: 'black', role: 'r-piece' }, 'f6')

    cg.newPiece({ color: 'none', role: 'l-piece' }, 'c3')
    cg.newPiece({ color: 'none', role: 'l-piece' }, 'e4')
  }
}
