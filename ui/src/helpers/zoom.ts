import { ntw } from '@/not-tailwind'
import { useGameSettings } from '@/stores/gameSettings'
import type { Elements, MouchEvent } from 'chessground12/types'
import { eventPosition } from 'chessground12/util'

export function zoom(elements: Elements) {
  const el = document.createElement('cg-resize')
  elements.container.appendChild(el)

  const zoomVar = ntw.get('--zoom')
  const settings = useGameSettings()
  document.body.style.setProperty(zoomVar, `${settings.zoom}`)

  const startResize = (start: MouchEvent) => {
    start.preventDefault()

    const mousemoveEvent = start.type === 'touchstart' ? 'touchmove' : 'mousemove',
      mouseupEvent = start.type === 'touchstart' ? 'touchend' : 'mouseup',
      startPos = eventPosition(start)!
    let initialZoom = settings.zoom

    const resize = (move: MouchEvent) => {
      const pos = eventPosition(move)!,
        delta = pos[0] - startPos[0] + pos[1] - startPos[1]

      let zoom = Math.round(Math.min(100, Math.max(0, initialZoom + delta / 10)))

      document.body.style.setProperty(zoomVar, `${zoom}`)
      localStorage.setItem('zoom', `${zoom}`)

      document.body.classList.add('resizing')
    }

    document.addEventListener(mousemoveEvent, resize)

    document.addEventListener(
      mouseupEvent,
      () => {
        document.removeEventListener(mousemoveEvent, resize)
        document.body.classList.remove('resizing')
      },
      { once: true },
    )
  }

  el.addEventListener('touchstart', startResize, { passive: false })
  el.addEventListener('mousedown', startResize, { passive: false })
}
