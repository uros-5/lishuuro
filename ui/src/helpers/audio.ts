import captureUrl from '@/assets/sounds/capture.ogg'
import resUrl from '@/assets/sounds/res.ogg'
import moveUrl from '@/assets/sounds/move.ogg'
import lowTimeUrl from '@/assets/sounds/low_time.ogg'
import buy from '@/assets/sounds/buy.ogg'

export function playAudio(sound: string, volume?: string) {
  let audio
  switch (sound) {
    case 'res':
      audio = resUrl
      break
    case 'move':
      audio = moveUrl
      break
    case 'capture':
      audio = captureUrl
      break
    case 'low_time':
      audio = lowTimeUrl
      break
    case 'buy':
      audio = buy
      break
  }
  const a = new Audio(audio)
  // volume ? (a.volume = Number(volume)) : null;
  a.play()
}
