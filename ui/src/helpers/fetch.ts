import { createFetch } from '@vueuse/core'
import { backend } from './backend'

const myFetch = createFetch({
  baseUrl: backend() + '/',
  options: {
    timeout: 3000,
    async beforeFetch(ctx) {
      ctx.options.credentials = 'include'
    },
  },
})

export function GET(query: string) {
  return myFetch(query)
}

export function POST(query: string, data: any) {
  return myFetch(query).post(data)
}
