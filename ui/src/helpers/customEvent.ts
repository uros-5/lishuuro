import type { MessageType } from "./rust_types"

export function ev2<K extends string, T>(
  type: K,
  listener: (event: CustomEvent<T>) => void,
  options?: boolean | AddEventListenerOptions,
): (event: Event) => void {
  const wrapped = (event: Event) => {
    listener(event as CustomEvent<T>)
  }
  addEventListener(type, wrapped, options)
  return wrapped
}
export function rmEv2(listener: (ev: Event) => void) {
  removeEventListener('wsMessage', listener)
}

export interface Event2 extends Event {
  detail: { t: MessageType; d: any }
}
