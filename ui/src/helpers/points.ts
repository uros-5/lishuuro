import { Variant } from './variantDescription'

type Piece = {
  piece: string
  points: number
  max: number
}

export const price = [0, 110, 70, 40, 40, 10, 130, 130, 70, 0]
export const limit = [1, 3, 6, 9, 9, 18, 3, 3, 4, 0]

export function updateCounter(variant: Variant, price: number[], limit: number[]) {
  switch (variant) {
    case Variant.Standard:
      price[5] = 10
      limit[5] = 12
      break
    case Variant.StandardFairy:
      price[5] = 10
      limit[5] = 12
      break
    case Variant.ShuuroMini:
      price[5] = 10
      limit[5] = 8
      break
    case Variant.ShuuroMiniFairy:
      price[5] = 10
      limit[5] = 8
      break
    default:
      undefined
  }
}

export const pieces = ['k', 'q', 'r', 'b', 'n', 'p', 'c', 'a', 'g']

export function isFairy(variant: Variant): boolean {
  return (
    variant == Variant.ShuuroFairy ||
    variant == Variant.ShuuroMiniFairy ||
    variant == Variant.StandardFairy
  )
}
