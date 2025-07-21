import { wsUrl } from '@/helpers/backend'
import { GET } from '@/helpers/fetch'
import { useWebSocket } from '@vueuse/core'
import { defineStore } from 'pinia'
import Cookies from 'js-cookie'
import { ref } from 'vue'
import router from '@/router'
import { MessageType } from '@/helpers/rust_types'

type VueUser = { username: string; logged: boolean }
let newconnection = 0
let pingInterval = 0
export const useWs = defineStore('ws', () => {
  const username = ref('   ')
  const unsendMessages = ref([])
  const connected = ref(true)
  const reg = ref(false)
  const redirectToastOpen = ref(false)

  const checkCookie = () => {
    if (Cookies.get('username') == null) {
      updateAnonCookie()
    } else {
      updateAnonCookie()
      username.value = Cookies.get('username')!
      reg.value = JSON.parse(Cookies.get('logged')!)
    }
  }

  const updateAnonCookie = () => {
    GET('vue_user').then((res) => {
      const data = res.data.value as string
      setUser(JSON.parse(data) as VueUser)
    })
  }

  const setUser = (vueUser: VueUser) => {
    username.value = vueUser.username
    reg.value = vueUser.logged
    const prod = import.meta.env.PROD
    const d = new Date()

    let message = new CustomEvent('wsMessage', {
      detail: { t: MessageType.NewPlayer, d: '' },
    })
    dispatchEvent(message)

    d.setTime(d.getTime() + 60 * 60 * 24 * 365)
    Cookies.set('username', username.value, setOptions(d, prod))
    Cookies.set('logged', reg.value.toString(), setOptions(d, prod))
  }
  const ws = useWebSocket(wsUrl() + '/ws/', {
    autoReconnect: true,
    onConnected: (ws) => {
      clearInterval(pingInterval)
      newconnection += 1
      if (newconnection > 1) {
        let name = router.currentRoute.value.name
        SEND({ t: MessageType.ChangeRoom, d: '' })
        if (name == 'home' || name == 'tv') {
          SEND({ t: MessageType.ChangeRoom, d: name })
        } else if (name == 'game') {
          let id = router.currentRoute.value.params.id
          SEND({ t: MessageType.ChangeRoom, d: `/game/${id}` })
        }
      }

      checkCookie()
      unsendMessages.value.forEach((value) => {
        SEND(value)
      })
      unsendMessages.value = []
      pingInterval = window.setInterval(() => {
        SEND('')
      }, 40 * 1000)
      connected.value = true
    },
    onDisconnected: (ws2) => {
      connected.value = false
    },
    onMessage: (ws, event) => {
      const msg: { t: string; d: any } = JSON.parse(event.data)

      let message = new CustomEvent('wsMessage', {
        detail: msg,
      })
      dispatchEvent(message)
    },
  })

  function SEND(msg: any) {
    // return
    try {
      ws.send(JSON.stringify(msg))
    } catch (error) {
      unsendMessages.value.push(msg as never)
    }
  }
  return { SEND, setUser, username, reg, redirectToastOpen }
})

export function setOptions(expires: Date, secure: boolean): Cookies.CookieAttributes {
  return {
    expires,
    path: '',
    domain: '',
    secure,
    sameSite: 'lax',
  }
}
