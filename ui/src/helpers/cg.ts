import { Chessground } from 'chessground12'
import type { Api } from 'chessground12/api'
import type { Config } from 'chessground12/config'
import { userProfileConfig } from 'chessground12/configs'
import { dimensions, Geometry, type BoardDimensions } from 'chessground12/types'

export function userProfileCg(id: string, geo: Geometry, dim: BoardDimensions[]): Api {
  const elem = document.getElementById(id) as HTMLElement
  userProfileConfig.addDimensionsCssVars = true
  userProfileConfig.autoCastle = false
  const cg = Chessground(elem, userProfileConfig, 500, undefined)
  change_variant(cg, geo, dim)
  return cg
}

export function deployCg(
  elem: HTMLElement,
  config: Config,
  top: HTMLElement,
  bot: HTMLElement,
  geo: Geometry,
  dim: BoardDimensions[],
): Api {
  config.addDimensionsCssVars = true
  config.autoCastle = false
  const cg = Chessground(elem, config, 800, top, bot)
  change_variant(cg, geo, dim)
  return cg
}

export function fightCg(
  elem: HTMLElement,
  config: Config,
  geometry: Geometry,
  dim: BoardDimensions[],
): Api {
  config.addDimensionsCssVars = true
  config.autoCastle = false
  const cg = Chessground(elem, config) as Api
  change_variant(cg, geometry, dim)
  return cg
}

export function change_variant(cg: Api, geometry: Geometry, dim: BoardDimensions[]) {
  // cg.state.variant = "standard";
  cg.state.dimensions = dim[0]
  cg.state.geometry = geometry
}
