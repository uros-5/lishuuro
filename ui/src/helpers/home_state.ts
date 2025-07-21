import * as v from 'valibot'
import { BoardSize, Variant, type Description } from './variantDescription'

export const allowedDuration = [
  1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 25, 30, 35, 40, 45, 60, 75,
  90,
]

const timeSchema = v.pipe(v.number(), v.minValue(1), v.maxValue(allowedDuration.length))

function toNumber(key: string) {
  let value = localStorage.getItem(key)
  return value == null || value == undefined ? undefined : Number(value)
}

export function homeState() {
  let minutes = v.safeParse(timeSchema, toNumber('minutes'))
  let increment = v.safeParse(timeSchema, toNumber('increment'))
  let variant = v.safeParse(v.pipe(v.number(), v.minValue(0), v.maxValue(5)), toNumber('variant'))
  let subVariant = v.safeParse(
    v.pipe(v.number(), v.minValue(0), v.maxValue(101)),
    toNumber('subvariant'),
  )
  let category = v.safeParse(v.pipe(v.number(), v.minValue(0), v.maxValue(3)), toNumber('category'))
  let defaultState = {
    minutes: minutes.success ? minutes.output : 14,
    increment: increment.success ? increment.output : 13,
    variant: variant.success ? variant.output : Variant.ShuuroMini,
    subVariant: subVariant.success ? subVariant.output : 100,
    category: category.success ? category.output : BoardSize.Mini,
  }
  return defaultState
}

export function updateState(
  minutes: number,
  increment: number,
  variant: Description | undefined,
  board: number,
) {
  localStorage.setItem('minutes', minutes.toString())
  localStorage.setItem('increment', increment.toString())
  localStorage.setItem('variant', `${variant?.variant.toString()}`)
  localStorage.setItem('subvariant', `${variant?.subVariant}`)
  localStorage.setItem('category', `${board}`)
}
