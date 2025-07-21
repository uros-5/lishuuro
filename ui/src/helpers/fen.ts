// export function from1to2(f: string): string {
//   const sfen = f.split("_");
//   const move = sfen[0].replace("@", "_");
//   const fen = sfen[1];
//   let hand = sfen[2];
//   hand = sfen[2] == " " ? "-" : sfen[2];
//   const stm = sfen[3];
//   const ply = sfen[4];
//   return `${fen} ${stm} ${hand} ${ply} ${move}`;
// }

export function formatSfen(fen: string, in_part = false): Sfen {
  // if (fen.includes("@")) {
  //   fen = from1to2(fen);
  // }
  const is_placement = fen.includes('@')
  const parts = fen.split(' ')
  const capture = fen.includes('x') ? true : false
  return {
    sfen: in_part ? `${parts[0]} ${parts[1]} ${parts[2]} 1` : fen,
    stm: parts[1],
    game_move: is_placement ? parts[4] : parts[5],
    capture,
  }
}

export function placementMove(move: string) {
  return move.split(' ')[4]
}

export type Sfen = {
  sfen: string
  stm: string
  game_move: string | undefined
  capture: boolean
}

export enum FenBtn {
  First,
  Next,
  Previous,
  Last,
}
