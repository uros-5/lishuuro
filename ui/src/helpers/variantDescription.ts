import placementStandard from '../assets/boards-live/placement-standard.png'
import selectionFairy from '../assets/boards-live/selection-fairy.png'
import selectionStandard from '../assets/boards-live/selection-standard.png'
import shuuroLiveFairy from '../assets/boards-live/shuuro-live-fairy.png'
import shuuroLive from '../assets/boards-live/shuuro-live.png'
import mini from '../assets/boards-live/mini.png'
import miniFairy from '../assets/boards-live/miniFairy.png'
import standardFairyLive from '../assets/boards-live/standard-fairy-live.png'
import standardLive from '../assets/boards-live/standard-live.png'
import standard from '../assets/boards-live/standard.png'

export type Description = {
  name: string
  points: number
  selection: boolean
  placement: boolean
  fairyPieces: boolean
  picture?: string[]
  category: BoardSize
  variant: Variant
  subVariant: SubVariant | 100
}

export enum Variant {
  Shuuro,
  ShuuroFairy,
  Standard,
  StandardFairy,
  ShuuroMini,
  ShuuroMiniFairy,
}

export enum SubVariant {
  Standard = 0,
  StandardFairy1 = 1,
  StandardFairy2 = 2,
  StandardPlacement = 3,
}

export enum BoardSize {
  Mini,
  Standard,
  Large,
}

export function variantStr(variant: Variant): string {
  switch (variant) {
    case Variant.Shuuro:
      return 'shuuro'
    case Variant.ShuuroFairy:
      return 'shuuroFairy'
    case Variant.Standard:
      return 'standard'
    case Variant.StandardFairy:
      return 'standardFairy'
    case Variant.ShuuroMini:
      return 'shuuroMini'
    case Variant.ShuuroMiniFairy:
      return 'shuuroMiniFairy'
    default:
      return 'shuuro'
  }
}

export const variants: Description[] = [
  {
    name: 'mini',
    points: 200,
    selection: true,
    placement: true,
    fairyPieces: false,
    picture: [mini],
    category: BoardSize.Mini,
    variant: Variant.ShuuroMini,
    subVariant: 100,
  },
  {
    name: 'fairy',
    points: 250,
    selection: true,
    placement: true,
    fairyPieces: true,
    picture: [selectionFairy, miniFairy],
    category: BoardSize.Mini,
    variant: Variant.ShuuroMiniFairy,
    subVariant: 100,
  },
  {
    name: 'shuuro',
    points: 800,
    selection: true,
    placement: true,
    fairyPieces: false,
    picture: [shuuroLive],
    category: BoardSize.Large,
    variant: Variant.Shuuro,

    subVariant: 100,
  },
  {
    name: 'fairy',
    points: 870,
    selection: true,
    placement: true,
    fairyPieces: false,
    picture: [selectionFairy, shuuroLiveFairy],
    category: BoardSize.Large,
    variant: Variant.ShuuroFairy,
    subVariant: 100,
  },
  {
    name: 'standard v1',
    points: 350,
    selection: true,
    placement: true,
    fairyPieces: false,
    picture: [standardLive],
    category: BoardSize.Standard,
    variant: Variant.Standard,
    subVariant: 100,
  },
  {
    name: 'fairy v1',
    points: 400,
    selection: true,
    placement: true,
    fairyPieces: true,
    picture: [selectionFairy, standardFairyLive],
    category: BoardSize.Standard,
    variant: Variant.StandardFairy,
    subVariant: 100,
  },
  {
    name: 'standard v2',
    points: 0,
    selection: false,
    placement: false,
    fairyPieces: false,
    picture: [standard],
    category: BoardSize.Standard,
    variant: Variant.Standard,
    subVariant: SubVariant.Standard,
  },
  {
    name: 'fairy v2',
    points: 0,
    selection: false,
    placement: false,
    fairyPieces: true,
    picture: [standardFairyLive],
    category: BoardSize.Standard,
    variant: Variant.StandardFairy,
    subVariant: SubVariant.StandardFairy1,
  },
  {
    name: 'fairy v3',
    points: 0,
    selection: false,
    placement: false,
    fairyPieces: true,
    picture: [standardFairyLive],
    category: BoardSize.Standard,
    variant: Variant.StandardFairy,
    subVariant: SubVariant.StandardFairy2,
  },
  {
    name: 'placement',
    points: 0,
    selection: false,
    placement: true,
    fairyPieces: false,
    picture: [selectionStandard, placementStandard],
    category: BoardSize.Standard,
    variant: Variant.Standard,
    subVariant: SubVariant.StandardPlacement,
  },
]
